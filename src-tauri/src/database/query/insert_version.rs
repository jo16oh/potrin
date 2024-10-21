use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::{Sqlite, Transaction};

pub async fn insert_version<'a>(
    tx: &mut Transaction<'a, Sqlite>,
    pot_id: &Base64,
    outline_ids: Vec<&Base64>,
    card_ids: Vec<&Base64>,
) -> anyhow::Result<Base64> {
    let version_id: Base64 = uuidv7::create_raw().to_vec().into();

    sqlx::query!(
        r#"
            INSERT INTO versions (id, pot_id)
            VALUES (?, ?);
        "#,
        version_id,
        pot_id
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| anyhow!(e))?;

    let query = format!(
        r#"
            INSERT INTO outline_y_updates_versions (version_id, y_update_id)
            SELECT ? AS version_id, id AS y_update_id
            FROM outline_y_updates 
            WHERE outline_id IN ({});
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query::<_>(&query);

    query_builder = query_builder.bind(&version_id);
    for id in outline_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .execute(&mut **tx)
        .await
        .map_err(|e| anyhow!(e))?;

    let query = format!(
        r#"
            INSERT INTO card_y_updates_versions (version_id, y_update_id)
            SELECT ? AS version_id, id AS y_update_id
            FROM card_y_updates 
            WHERE card_id IN ({});
        "#,
        card_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query::<_>(&query);

    query_builder = query_builder.bind(&version_id);
    for id in card_ids {
        query_builder = query_builder.bind(id)
    }

    query_builder
        .execute(&mut **tx)
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(version_id)
}
