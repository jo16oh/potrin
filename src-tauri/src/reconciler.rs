use crate::{
    database::query::{delete, fetch, upsert},
    events::{Origin, OutlineChange, ParagraphChange, Target},
    search_engine::{add_index, remove_index, DeleteTarget, IndexTarget},
    types::{
        error::PotrinError,
        model::{Ancestor, Link, Oplog, OutlineForIndex, ParagraphForIndex, YUpdate},
        util::UUIDv7Base64URL,
    },
    utils::get_state,
};
use eyre::OptionExt;
use itertools::{Either, Itertools};
use serde::Deserialize;
use sqlx::SqlitePool;
use std::collections::{HashMap, VecDeque};
use tauri::{
    async_runtime::{self, channel, Sender},
    AppHandle, Manager, Runtime,
};
use tauri_specta::Event;

#[derive(Deserialize)]
struct Status {
    deleted: bool,
}

#[derive(Deserialize)]
struct DeleteLogStatus {
    pot_id: String,
}

pub struct Reconciler {
    sender: Sender<DatabaseChange>,
}

impl Reconciler {
    pub async fn send(&self, target: DatabaseChange) -> eyre::Result<()> {
        self.sender.send(target).await.map_err(|e| e.into())
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

pub async fn init<R: Runtime>(app_handle: &AppHandle<R>) -> eyre::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let change = fetch::oplog_rowids_all(pool)
        .await
        .map(|ids| DatabaseChange::new(ids, Origin::Init))?;
    reconcile::<R>(app_handle, change).await?;

    let (sender, mut receiver) = channel::<DatabaseChange>(100);

    app_handle.manage::<Reconciler>(Reconciler { sender });

    async_runtime::spawn({
        let app_handle = app_handle.clone();
        async move {
            while let Some(change) = receiver.recv().await {
                let _ = reconcile(&app_handle, change)
                    .await
                    .map_err(PotrinError::from)
                    .inspect_err(|e| eprintln!("{}", e));
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
                    .filter(|(_, deleted)| *deleted)
                    .map(|(log, _)| log.primary_key)
                    .collect();

                if !inserted_ids.is_empty() {
                    update_path(pool, &inserted_ids).await?;

                    let inserted_outlines =
                        fetch::outlines_for_index_by_id(pool, &inserted_ids).await?;

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
                    update_path(pool, &updated_ids).await?;

                    let updated_outlines =
                        fetch::outlines_for_index_by_id(pool, &updated_ids).await?;

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
                    update_path(pool, &deleted_ids).await?;

                    let delete_targets = fetch::outline_delete_targets(pool, &deleted_ids).await?;
                    remove_index(app_handle, delete_targets).await?;

                    OutlineChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
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
                    .filter(|(_, deleted)| *deleted)
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

async fn update_path(pool: &SqlitePool, outline_ids: &[UUIDv7Base64URL]) -> eyre::Result<()> {
    let outlines = fetch::outline_trees(pool, outline_ids, None).await?;
    let ancestors: HashMap<UUIDv7Base64URL, Ancestor> = fetch::ancestors(
        pool,
        &outlines
            .iter()
            .map(|o| o.id)
            .collect::<Vec<UUIDv7Base64URL>>(),
    )
    .await?
    .into_iter()
    .map(|a| (a.id, a))
    .collect();

    let path = outlines
        .into_iter()
        .map(|outline| {
            let mut path: VecDeque<Link> = VecDeque::new();
            let mut current_id = outline.parent_id;

            while let Some(id) = current_id.as_ref() {
                if let Some(ancestor) = ancestors.get(id) {
                    current_id = ancestor.parent_id;
                    path.push_front(Link {
                        id: ancestor.id,
                        text: ancestor.text.clone(),
                        hidden: ancestor.hidden,
                    });
                } else {
                    break;
                }
            }

            Ok((outline.id, serde_sqlite_jsonb::to_vec(&path)?))
        })
        .collect::<eyre::Result<Vec<(UUIDv7Base64URL, Vec<u8>)>>>()?;

    upsert::path(pool, &path).await?;

    Ok(())
}
