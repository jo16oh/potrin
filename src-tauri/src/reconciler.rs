use crate::{
    database::query::{delete, fetch, upsert},
    events::{CardChange, Origin, OutlineChange, Target},
    search_engine::{add_index, remove_index, DeleteTarget, IndexTarget},
    types::{
        model::{Ancestor, CardForIndex, Link, Oplog, OutlineForIndex, YUpdate},
        util::UUIDv7Base64,
    },
    utils::{extract_text_from_doc, get_state},
};
use anyhow::Context;
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
    is_deleted: bool,
}

#[derive(Deserialize)]
struct DeleteLogStatus {
    pot_id: String,
}

pub struct Reconciler {
    sender: Sender<DatabaseChange>,
}

impl Reconciler {
    pub async fn send(&self, target: DatabaseChange) -> anyhow::Result<()> {
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

pub async fn init<R: Runtime>(app_handle: &AppHandle<R>) -> anyhow::Result<()> {
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
                let r = reconcile(&app_handle, change).await;

                if let Err(e) = r {
                    println!("{}", e);
                }
            }
        }
    });

    Ok(())
}

async fn reconcile<R: Runtime>(
    app_handle: &AppHandle<R>,
    change: DatabaseChange,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let logs = fetch::oplogs_by_rowid(pool, &change.oplog_rowids).await?;
    let mut y_updates_map = fetch::y_updates_by_id(
        pool,
        &logs
            .iter()
            .filter(|log| log.tablename == "y_updates")
            .map(|log| log.primary_key)
            .collect::<Vec<UUIDv7Base64>>(),
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
            "cards" => {
                process_card_changes(app_handle, logs, &mut y_updates_map, &change.origin).await?
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
    y_updates_map: &mut HashMap<UUIDv7Base64, Vec<YUpdate>>,
    origin: &Origin,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let operation_logs_map = outline_logs
        .iter()
        .into_group_map_by(|l| l.operation.as_str());

    for (operation, logs) in operation_logs_map.into_iter() {
        match operation {
            "insert" => {
                // omit logically deleted rows from indexing
                let inserted_ids: Vec<UUIDv7Base64> = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().context("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.is_deleted))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
                    .into_iter()
                    .filter(|(_, is_deleted)| *is_deleted)
                    .map(|(log, _)| log.primary_key)
                    .collect();

                if !inserted_ids.is_empty() {
                    update_breadcrumbs(pool, &inserted_ids).await?;

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
                                    breadcrumbs: &e.breadcrumbs,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<anyhow::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    OutlineChange::insert(
                        inserted_outlines
                            .into_iter()
                            .map(|o| {
                                let y_updates = y_updates_map
                                    .remove(&o.id)
                                    .context("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(o, y_updates))
                            })
                            .collect::<anyhow::Result<Vec<Target<OutlineForIndex>>>>()?,
                        origin.clone(),
                    );
                }
            }
            "update" => {
                let (updated_ids, deleted_ids): (Vec<UUIDv7Base64>, Vec<UUIDv7Base64>) = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().context("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.is_deleted))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
                    .into_iter()
                    .partition_map(|(log, is_deleted)| {
                        if is_deleted {
                            Either::Left(&log.primary_key)
                        } else {
                            Either::Right(&log.primary_key)
                        }
                    });

                if !updated_ids.is_empty() {
                    update_breadcrumbs(pool, &updated_ids).await?;

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
                                    breadcrumbs: &e.breadcrumbs,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<anyhow::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    OutlineChange::update(
                        updated_outlines
                            .into_iter()
                            .map(|o| {
                                let y_updates = y_updates_map
                                    .remove(&o.id)
                                    .context("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(o, y_updates))
                            })
                            .collect::<anyhow::Result<Vec<Target<OutlineForIndex>>>>()?,
                        origin.clone(),
                    )
                    .emit(app_handle)?;
                }

                if !deleted_ids.is_empty() {
                    update_breadcrumbs(pool, &deleted_ids).await?;

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
                        let status = log.status.as_ref().context("status is not set")?;
                        let decoded_status: DeleteLogStatus =
                            serde_sqlite_jsonb::from_slice(status)?;
                        let pot_id: UUIDv7Base64 =
                            hex::decode(decoded_status.pot_id)?.try_into()?;

                        Ok(DeleteTarget {
                            id: log.primary_key,
                            pot_id,
                        })
                    })
                    .collect::<anyhow::Result<Vec<DeleteTarget>>>()?;

                remove_index(app_handle, delete_targets).await?;

                OutlineChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn process_card_changes<R: Runtime>(
    app_handle: &AppHandle<R>,
    card_logs: Vec<&Oplog>,
    y_updates_map: &mut HashMap<UUIDv7Base64, Vec<YUpdate>>,
    origin: &Origin,
) -> anyhow::Result<()> {
    let pool = get_state::<R, SqlitePool>(app_handle).unwrap();

    let operation_logs_map = card_logs.iter().into_group_map_by(|l| l.operation.as_str());

    for (operation, logs) in operation_logs_map.into_iter() {
        match operation {
            "insert" => {
                // omit logically deleted rows from indexing
                let inserted_ids: Vec<UUIDv7Base64> = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().context("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.is_deleted))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
                    .into_iter()
                    .filter(|(_, is_deleted)| *is_deleted)
                    .map(|(log, _)| log.primary_key)
                    .collect();

                if !inserted_ids.is_empty() {
                    let inserted_cards = fetch::cards_for_index_by_id(pool, &inserted_ids).await?;

                    add_index(
                        app_handle,
                        inserted_cards
                            .iter()
                            .map(|e| {
                                Ok(IndexTarget {
                                    id: e.id,
                                    pot_id: e.pot_id,
                                    doc_type: "card",
                                    doc: &e.doc,
                                    breadcrumbs: &e.breadcrumbs,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<anyhow::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    CardChange::insert(
                        inserted_cards
                            .into_iter()
                            .map(|c| {
                                let y_updates = y_updates_map
                                    .remove(&c.id)
                                    .context("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(c, y_updates))
                            })
                            .collect::<anyhow::Result<Vec<Target<CardForIndex>>>>()?,
                        origin.clone(),
                    );
                }
            }
            "update" => {
                let (updated_ids, deleted_ids): (Vec<UUIDv7Base64>, Vec<UUIDv7Base64>) = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().context("status is not set")?;
                        let decoded_status: Status = serde_sqlite_jsonb::from_slice(status)?;
                        Ok((log, decoded_status.is_deleted))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
                    .into_iter()
                    .partition_map(|(log, is_deleted)| {
                        if is_deleted {
                            Either::Left(&log.primary_key)
                        } else {
                            Either::Right(&log.primary_key)
                        }
                    });

                if !updated_ids.is_empty() {
                    let updated_cards = fetch::cards_for_index_by_id(pool, &updated_ids).await?;

                    add_index(
                        app_handle,
                        updated_cards
                            .iter()
                            .map(|e| {
                                Ok(IndexTarget {
                                    id: e.id,
                                    pot_id: e.pot_id,
                                    doc_type: "card",
                                    doc: &e.doc,
                                    breadcrumbs: &e.breadcrumbs,
                                    links: &e.links,
                                    created_at: e.created_at,
                                    updated_at: e.updated_at,
                                })
                            })
                            .collect::<anyhow::Result<Vec<IndexTarget>>>()?,
                    )
                    .await?;

                    CardChange::update(
                        updated_cards
                            .into_iter()
                            .map(|c| {
                                let y_updates = y_updates_map
                                    .remove(&c.id)
                                    .context("")?
                                    .into_iter()
                                    .map(|y| y.data)
                                    .collect();
                                Ok(Target::new(c, y_updates))
                            })
                            .collect::<anyhow::Result<Vec<Target<CardForIndex>>>>()?,
                        origin.clone(),
                    )
                    .emit(app_handle)?;
                }

                if !deleted_ids.is_empty() {
                    let delete_targets = fetch::card_delete_targets(pool, &deleted_ids).await?;
                    remove_index(app_handle, delete_targets).await?;

                    CardChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
                }
            }
            "delete" => {
                let deleted_ids = logs.iter().map(|l| l.primary_key).collect();
                let delete_targets = logs
                    .into_iter()
                    .map(|log| {
                        let status = log.status.as_ref().context("status is not set")?;
                        let decoded_status: DeleteLogStatus =
                            serde_sqlite_jsonb::from_slice(status)?;
                        let pot_id: UUIDv7Base64 =
                            hex::decode(decoded_status.pot_id)?.try_into()?;

                        Ok(DeleteTarget {
                            id: log.primary_key,
                            pot_id,
                        })
                    })
                    .collect::<anyhow::Result<Vec<DeleteTarget>>>()?;

                remove_index(app_handle, delete_targets).await?;

                OutlineChange::delete(deleted_ids, origin.clone()).emit(app_handle)?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn update_breadcrumbs(pool: &SqlitePool, outline_ids: &[UUIDv7Base64]) -> anyhow::Result<()> {
    let outlines = fetch::outline_trees(pool, outline_ids, None).await?;
    let ancestors: HashMap<UUIDv7Base64, Ancestor> = fetch::ancestors(
        pool,
        &outlines.iter().map(|o| o.id).collect::<Vec<UUIDv7Base64>>(),
    )
    .await?
    .into_iter()
    .map(|a| (a.id, a))
    .collect();

    let breadcrumbs = outlines
        .into_iter()
        .map(|outline| {
            let mut breadcrumbs: VecDeque<Link> = VecDeque::new();
            let mut current_id = outline.parent_id;

            while let Some(id) = current_id.as_ref() {
                if let Some(breadcrumb) = ancestors.get(id) {
                    current_id = breadcrumb.parent_id;
                    breadcrumbs.push_front(Link {
                        id: breadcrumb.id,
                        text: extract_text_from_doc(&breadcrumb.doc)?,
                    });
                } else {
                    break;
                }
            }

            Ok((outline.id, serde_sqlite_jsonb::to_vec(&breadcrumbs)?))
        })
        .collect::<anyhow::Result<Vec<(UUIDv7Base64, Vec<u8>)>>>()?;

    upsert::breadcrumbs(pool, &breadcrumbs).await?;

    Ok(())
}
