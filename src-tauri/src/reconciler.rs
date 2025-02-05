use crate::{
    database::query::{delete, fetch, upsert},
    events::{Origin, OutlineChange, ParagraphChange, Target},
    search_engine::{add_index, remove_index, DeleteTarget, IndexTarget},
    types::{
        error::PotrinError,
        model::{
            Ancestor, Descendant, Link, Oplog, OutlineForIndex, ParagraphForIndex, Path, YUpdate,
        },
        util::UUIDv7Base64URL,
    },
    utils::get_state,
};
use eyre::{eyre, OptionExt};
use itertools::{Either, Itertools};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::{HashMap, VecDeque};
use tauri::{
    async_runtime::{self, channel, Sender},
    AppHandle, Manager, Runtime,
};
use tauri_specta::Event;
use tokio::sync::oneshot;

#[derive(Deserialize)]
struct Status {
    deleted: bool,
}

#[derive(Deserialize)]
struct DeleteLogStatus {
    pot_id: String,
}

pub struct Reconciler {
    sender: Sender<Message>,
}

impl Reconciler {
    pub async fn send(&self, change: DatabaseChange) -> eyre::Result<()> {
        self.sender
            .send(Message::DatabaseChange(change))
            .await
            .map_err(|e| e.into())
    }

    /// Returns a Future that resolves when all tasks that were pending at the time this function was called have finished.
    #[allow(unused)]
    pub async fn wait(&self) -> eyre::Result<()> {
        let (tx, rx) = oneshot::channel::<()>();
        self.sender.send(Message::Wait(tx)).await?;
        rx.await.map_err(|e| e.into())
    }
}

pub struct DatabaseChange {
    pub oplog_rowids: Vec<i64>,
    pub origin: Origin,
}

impl DatabaseChange {
    pub fn new(rowids: Vec<i64>, origin: Origin) -> Self {
        Self {
            oplog_rowids: rowids,
            origin,
        }
    }
}

enum Message {
    #[allow(unused)]
    Wait(oneshot::Sender<()>),
    DatabaseChange(DatabaseChange),
}

pub async fn init<R: Runtime>(app_handle: &AppHandle<R>) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let change = fetch::oplog_rowids_all(pool)
        .await
        .map(|ids| DatabaseChange::new(ids, Origin::Init))?;
    reconcile::<R>(app_handle, change).await?;

    let (sender, mut receiver) = channel::<Message>(100);

    app_handle.manage::<Reconciler>(Reconciler { sender });

    async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    Message::Wait(sender) => {
                        let _ = sender
                            .send(())
                            .map_err(|_| eyre!("failed to send message"))
                            .map_err(PotrinError::from)
                            .inspect_err(|e| eprintln!("{:?}", e));
                    }
                    Message::DatabaseChange(change) => {
                        let _ = reconcile(&app_handle, change)
                            .await
                            .map_err(PotrinError::from)
                            .inspect_err(|e| eprintln!("{:?}", e));
                    }
                }
            }
        }
    });

    Ok(())
}

async fn reconcile<R: Runtime>(
    app_handle: &AppHandle<R>,
    change: DatabaseChange,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let logs = fetch::oplogs_by_rowid(pool, &change.oplog_rowids).await?;
    let mut y_updates_map = fetch::y_updates_by_id(
        pool,
        &logs
            .iter()
            .filter(|log| log.tablename == "y_updates")
            .map(|log| log.primary_key)
            .collect::<Vec<UUIDv7Base64URL>>(),
    )
    .await?
    .into_iter()
    .into_group_map_by(|update| update.y_doc_id);

    let rowids = logs.iter().map(|e| e.rowid).collect::<Vec<i64>>();
    let tablename_logs_map = logs.iter().into_group_map_by(|l| l.tablename.as_str());

    for (tablename, logs) in tablename_logs_map.into_iter() {
        match tablename {
            "outlines" => {
                process_outline_changes(app_handle, logs, &mut y_updates_map, &change.origin)
                    .await?
            }
            "paragraphs" => {
                process_paragraph_changes(app_handle, logs, &mut y_updates_map, &change.origin)
                    .await?
            }
            _ => {}
        }
    }

    delete::oplogs(pool, &rowids).await?;

    Ok(())
}

async fn process_outline_changes<R: Runtime>(
    app_handle: &AppHandle<R>,
    outline_logs: Vec<&Oplog>,
    y_updates_map: &mut HashMap<UUIDv7Base64URL, Vec<YUpdate>>,
    origin: &Origin,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let operation_logs_map = outline_logs
        .iter()
        .into_group_map_by(|l| l.operation.as_str());

    for (operation, logs) in operation_logs_map.into_iter() {
        match operation {
            "insert" => {
                // omit logically deleted rows from indexing
                let inserted_ids: Vec<UUIDv7Base64URL> = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().ok_or_eyre("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.deleted))
                    })
                    .collect::<eyre::Result<Vec<_>>>()?
                    .into_iter()
                    .filter(|(_, deleted)| !*deleted)
                    .map(|(log, _)| log.primary_key)
                    .collect();

                if !inserted_ids.is_empty() {
                    let inserted_outlines =
                        fetch::outlines_for_index_by_id(pool, &inserted_ids).await?;

                    upsert_path(pool, &inserted_ids).await?;

                    add_index(
                        app_handle,
                        inserted_outlines
                            .iter()
                            .map(|e| {
                                Ok(IndexTarget {
                                    id: e.id,
                                    pot_id: e.pot_id,
                                    doc_type: "outline",
                                    doc: &e.doc,
                                    path: &e.path,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<eyre::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    OutlineChange::insert(
                        inserted_outlines
                            .into_iter()
                            .map(|o| {
                                let y_updates = y_updates_map
                                    .remove(&o.id)
                                    .ok_or_eyre("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(o, y_updates))
                            })
                            .collect::<eyre::Result<Vec<Target<OutlineForIndex>>>>()?,
                        origin.clone(),
                    );
                }
            }
            "update" => {
                let (updated_ids, deleted_ids): (Vec<UUIDv7Base64URL>, Vec<UUIDv7Base64URL>) = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().ok_or_eyre("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.deleted))
                    })
                    .collect::<eyre::Result<Vec<_>>>()?
                    .into_iter()
                    .partition_map(|(log, deleted)| {
                        if deleted {
                            Either::Left(&log.primary_key)
                        } else {
                            Either::Right(&log.primary_key)
                        }
                    });

                if !updated_ids.is_empty() {
                    let updated_outlines =
                        fetch::outlines_for_index_by_id(pool, &updated_ids).await?;

                    upsert_path(pool, &updated_ids).await?;

                    add_index(
                        app_handle,
                        updated_outlines
                            .iter()
                            .map(|e| {
                                Ok(IndexTarget {
                                    id: e.id,
                                    pot_id: e.pot_id,
                                    doc_type: "outline",
                                    doc: &e.doc,
                                    path: &e.path,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<eyre::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    OutlineChange::update(
                        updated_outlines
                            .into_iter()
                            .map(|o| {
                                let y_updates = y_updates_map
                                    .remove(&o.id)
                                    .ok_or_eyre("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(o, y_updates))
                            })
                            .collect::<eyre::Result<Vec<Target<OutlineForIndex>>>>()?,
                        origin.clone(),
                    )
                    .emit(app_handle)?;
                }

                if !deleted_ids.is_empty() {
                    let delete_targets = fetch::outline_delete_targets(pool, &deleted_ids).await?;
                    upsert_path(pool, &deleted_ids).await?;
                    remove_index(app_handle, delete_targets).await?;

                    OutlineChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
                }
            }
            "delete" => {
                let deleted_ids = logs
                    .iter()
                    .map(|l| l.primary_key)
                    .collect::<Vec<UUIDv7Base64URL>>();

                let delete_targets = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().ok_or_eyre("status is not set")?;
                        let decoded_status: DeleteLogStatus =
                            serde_sqlite_jsonb::from_slice(status)?;
                        let pot_id: UUIDv7Base64URL =
                            hex::decode(decoded_status.pot_id)?.try_into()?;

                        Ok(DeleteTarget {
                            id: log.primary_key,
                            pot_id,
                        })
                    })
                    .collect::<eyre::Result<Vec<DeleteTarget>>>()?;

                upsert_path(pool, &deleted_ids).await?;
                remove_index(app_handle, delete_targets).await?;

                OutlineChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn process_paragraph_changes<R: Runtime>(
    app_handle: &AppHandle<R>,
    paragraph_logs: Vec<&Oplog>,
    y_updates_map: &mut HashMap<UUIDv7Base64URL, Vec<YUpdate>>,
    origin: &Origin,
) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let operation_logs_map = paragraph_logs
        .iter()
        .into_group_map_by(|l| l.operation.as_str());

    for (operation, logs) in operation_logs_map.into_iter() {
        match operation {
            "insert" => {
                // omit logically deleted rows from indexing
                let inserted_ids: Vec<UUIDv7Base64URL> = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().ok_or_eyre("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.deleted))
                    })
                    .collect::<eyre::Result<Vec<_>>>()?
                    .into_iter()
                    .filter(|(_, deleted)| !*deleted)
                    .map(|(log, _)| log.primary_key)
                    .collect();

                if !inserted_ids.is_empty() {
                    let inserted_paragraphs =
                        fetch::paragraphs_for_index_by_id(pool, &inserted_ids).await?;

                    add_index(
                        app_handle,
                        inserted_paragraphs
                            .iter()
                            .map(|e| {
                                Ok(IndexTarget {
                                    id: e.id,
                                    pot_id: e.pot_id,
                                    doc_type: "paragraph",
                                    doc: &e.doc,
                                    path: &e.path,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<eyre::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    ParagraphChange::insert(
                        inserted_paragraphs
                            .into_iter()
                            .map(|c| {
                                let y_updates = y_updates_map
                                    .remove(&c.id)
                                    .ok_or_eyre("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(c, y_updates))
                            })
                            .collect::<eyre::Result<Vec<Target<ParagraphForIndex>>>>()?,
                        origin.clone(),
                    );
                }
            }
            "update" => {
                let (updated_ids, deleted_ids): (Vec<UUIDv7Base64URL>, Vec<UUIDv7Base64URL>) = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().ok_or_eyre("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.deleted))
                    })
                    .collect::<eyre::Result<Vec<_>>>()?
                    .into_iter()
                    .partition_map(|(log, deleted)| {
                        if deleted {
                            Either::Left(&log.primary_key)
                        } else {
                            Either::Right(&log.primary_key)
                        }
                    });

                if !updated_ids.is_empty() {
                    let updated_paragraphs =
                        fetch::paragraphs_for_index_by_id(pool, &updated_ids).await?;

                    add_index(
                        app_handle,
                        updated_paragraphs
                            .iter()
                            .map(|e| {
                                Ok(IndexTarget {
                                    id: e.id,
                                    pot_id: e.pot_id,
                                    doc_type: "paragraph",
                                    doc: &e.doc,
                                    path: &e.path,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<eyre::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    ParagraphChange::update(
                        updated_paragraphs
                            .into_iter()
                            .map(|c| {
                                let y_updates = y_updates_map
                                    .remove(&c.id)
                                    .ok_or_eyre("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(c, y_updates))
                            })
                            .collect::<eyre::Result<Vec<Target<ParagraphForIndex>>>>()?,
                        origin.clone(),
                    )
                    .emit(app_handle)?;
                }

                if !deleted_ids.is_empty() {
                    let delete_targets =
                        fetch::paragraph_delete_targets(pool, &deleted_ids).await?;
                    remove_index(app_handle, delete_targets).await?;

                    ParagraphChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
                }
            }
            "delete" => {
                let deleted_ids = logs.iter().map(|l| l.primary_key).collect();
                let delete_targets = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().ok_or_eyre("status is not set")?;
                        let decoded_status: DeleteLogStatus =
                            serde_sqlite_jsonb::from_slice(status)?;
                        let pot_id: UUIDv7Base64URL =
                            hex::decode(decoded_status.pot_id)?.try_into()?;

                        Ok(DeleteTarget {
                            id: log.primary_key,
                            pot_id,
                        })
                    })
                    .collect::<eyre::Result<Vec<DeleteTarget>>>()?;

                remove_index(app_handle, delete_targets).await?;

                OutlineChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn upsert_path(
    pool: &SqlitePool,
    changed_outline_ids: &[UUIDv7Base64URL],
) -> eyre::Result<()> {
    let updated_paths_map = {
        let self_and_its_ancestors: HashMap<UUIDv7Base64URL, Ancestor> =
            fetch::self_and_its_ancestors(pool, changed_outline_ids)
                .await?
                .into_iter()
                .map(|a| (a.id, a))
                .collect();

        changed_outline_ids
            .iter()
            .map(|id| {
                let mut path: VecDeque<Link> = VecDeque::new();
                let mut current_id = Some(id);

                while let Some(id) = current_id.as_ref() {
                    if let Some(ancestor) = self_and_its_ancestors.get(id) {
                        current_id = ancestor.parent_id.as_ref();
                        path.push_front(Link {
                            id: ancestor.id,
                            text: ancestor.text.clone(),
                            hidden: ancestor.hidden,
                        });
                    } else {
                        break;
                    }
                }

                (*id, Path::from(path))
            })
            .collect::<HashMap<UUIDv7Base64URL, Path>>()
    };

    let mut parent_children_map = fetch::descendants(pool, changed_outline_ids)
        .await?
        .into_iter()
        .map(|d| (d.parent_id, d))
        .into_group_map();

    let mut results = vec![];

    for (id, path) in updated_paths_map.into_iter() {
        results.push((id, serde_sqlite_jsonb::to_vec(&path)?));
        update_descendants_paths_recursively(&mut results, id, &path, &mut parent_children_map)?;
    }

    fn update_descendants_paths_recursively(
        results: &mut Vec<(UUIDv7Base64URL, Vec<u8>)>,
        parent_id: UUIDv7Base64URL,
        path: &Path,
        descendants_map: &mut HashMap<UUIDv7Base64URL, Vec<Descendant>>,
    ) -> eyre::Result<()> {
        let mut children_ids = vec![];

        if let Some(children) = descendants_map.get_mut(&parent_id) {
            for c in children {
                if let Some(ref mut p) = c.path {
                    p.replace_ancestors_with(path);
                    results.push((c.id, serde_sqlite_jsonb::to_vec(&p)?));
                    children_ids.push(c.id);
                }
            }
        }

        for id in children_ids {
            update_descendants_paths_recursively(results, id, path, descendants_map)?;
        }

        Ok(())
    }

    upsert::path(pool, &results).await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        commands::{fetch_path::fetch_path, upsert_outline::test::upsert_outline},
        database::test::create_mock_pot,
        run_in_mock_app,
        types::model::Outline,
    };
    use tauri::test::MockRuntime;

    #[test]
    fn test_josnb_array() {
        let empty_vec: Vec<Link> = vec![];
        let encoded = serde_sqlite_jsonb::to_vec(&empty_vec).unwrap();
        let decoded: Vec<Link> = serde_sqlite_jsonb::from_slice(&encoded).unwrap();

        assert_eq!(decoded.len(), 0);
    }

    #[test]
    fn test_upsert_path() {
        run_in_mock_app!(test_upsert_path_impl)
    }

    async fn test_upsert_path_impl(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pot = create_mock_pot(app_handle).await;

        let o1 = Outline::new(None);
        let o2 = Outline::new(Some(o1.id));
        let o3 = Outline::new(Some(o2.id));
        upsert_outline(app_handle, pot.id, &o1, vec![])
            .await
            .unwrap();
        upsert_outline(app_handle, pot.id, &o2, vec![])
            .await
            .unwrap();
        upsert_outline(app_handle, pot.id, &o3, vec![])
            .await
            .unwrap();

        let reconciler = get_state::<MockRuntime, Reconciler>(app_handle)?;
        reconciler.wait().await?;

        let path = fetch_path(app_handle.clone(), o1.id).await.unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(path.inner()[0].id, o1.id);

        let path = fetch_path(app_handle.clone(), o2.id).await.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path.inner()[0].id, o1.id);
        assert_eq!(path.inner()[1].id, o2.id);

        let path = fetch_path(app_handle.clone(), o3.id).await.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path.inner()[0].id, o1.id);
        assert_eq!(path.inner()[1].id, o2.id);
        assert_eq!(path.inner()[2].id, o3.id);

        eyre::Ok(())
    }
}

// use yrs::{
//     merge_updates_v2, updates::decoder::Decode, Doc, Map, MapRef, Transact, Update, XmlFragment,
//     XmlFragmentRef,
// };
// use crate::types::util::BytesBase64URL;

// async fn save_pending_y_updates(pool: &SqlitePool) -> eyre::Result<()> {
//     let type_updates_map = fetch::pending_y_updates(pool)
//         .await?
//         .into_iter()
//         .map(|u| (u.doc_type, (u.y_doc_id, u.data, u.timestamp)))
//         .into_group_map()
//         .into_iter()
//         .map(|(doc_type, updates)| {
//             let merged_updates = updates
//                 .into_iter()
//                 .map(|(y_doc_id, data, timestamp)| (y_doc_id, (data, timestamp)))
//                 .into_group_map()
//                 .into_iter()
//                 .map(|(y_doc_id, updates)| {
//                     let (y_updates, timestamps): (Vec<BytesBase64URL>, Vec<i64>) =
//                         updates.into_iter().unzip();
//
//                     let merged_y_update: BytesBase64URL = merge_updates_v2(y_updates)?.into();
//                     let timestamp = timestamps
//                         .into_iter()
//                         .max()
//                         .ok_or_eyre("failed to find created_at")?;
//
//                     eyre::Ok(YUpdate::new(y_doc_id, merged_y_update, None, timestamp))
//                 })
//                 .collect::<eyre::Result<Vec<YUpdate>>>()?;
//
//             Ok((doc_type, merged_updates))
//         })
//         .collect::<eyre::Result<Vec<(String, Vec<YUpdate>)>>>()?;
//
//     for (doc_type, updates) in type_updates_map.into_iter() {
//         match &doc_type {
//             "outline" => {
//                 for update in updates {
//                     let mut updates: Vec<Vec<u8>> =
//                         fetch::y_updates_by_doc_id(pool, update.y_doc_id)
//                             .await?
//                             .into_iter()
//                             .map(|u| u.into())
//                             .collect();
//
//                     updates.push(update.data);
//
//                     let outline = materialize_outline(&updates);
//                 }
//             }
//             "paragraph" => {}
//             _ => {}
//         }
//     }
//
//     Ok(())
// }
//
// async fn materialize_outline(updates: &[Vec<u8>]) -> eyre::Result<Outline> {
//     let mut ydoc = Doc::new();
//     let mut txn = ydoc.transact_mut();
//
//     for update in updates {
//         txn.apply_update(Update::decode_v2(update)?);
//     }
//
//     txn.commit();
//
//     let map = ydoc.get_or_insert_map("potrin");
//
//     let parent_id = map.get(&txn, "parentId").ok_or_eyre("")?.cast::<String>()?;
//     let fractional_index = map
//         .get(&txn, "fractional_index")
//         .ok_or_eyre("")?
//         .cast::<String>()?;
//     let doc = map
//         .get(&txn, "doc")
//         .ok_or_eyre("")?
//         .cast::<XmlFragmentRef>()?;
//     let links = map.get(&txn, "links").ok_or_eyre("")?.cast::<MapRef>()?;
//     let hidden = map.get(&txn, "hidden").ok_or_eyre("")?.cast::<bool>()?;
//     let collapsed = map.get(&txn, "collapsed").ok_or_eyre("")?.cast::<bool>()?;
//     let deleted = map.get(&txn, "deleted").ok_or_eyre("")?.cast::<bool>()?;
//
//     Ok()
// }
//
// async fn materialize_paragraph(updates: &[Vec<u8>]) -> Paragraph {}
