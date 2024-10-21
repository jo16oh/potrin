use crate::types::model::YUpdate;
use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::SqliteExecutor;

pub async fn insert_outline_y_updates<'a, E>(
    conn: E,
    outline_id: &Base64,
    y_updates: &[YUpdate],
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    if y_updates.is_empty() {
        return Err(anyhow!("no y-update provided"));
    }

    let query = format!(
        r#"
            INSERT INTO outline_y_updates (id, outline_id, data)
            VALUES {};
        "#,
        y_updates
            .iter()
            .map(|_| "(?, ?, ?)".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for update in y_updates {
        query_builder = query_builder.bind(&update.id);
        query_builder = query_builder.bind(outline_id);
        query_builder = query_builder.bind(&update.data);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn insert_card_y_updates<'a, E>(
    conn: E,
    card_id: &Base64,
    y_updates: &[YUpdate],
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    if y_updates.is_empty() {
        return Err(anyhow!("no y-update provided"));
    }

    let query = format!(
        r#"
            INSERT INTO card_y_updates (id, card_id, data)
            VALUES {};
        "#,
        y_updates
            .iter()
            .map(|_| "(?, ?, ?)".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for update in y_updates {
        query_builder = query_builder.bind(&update.id);
        query_builder = query_builder.bind(card_id);
        query_builder = query_builder.bind(&update.data);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn insert_outline_y_updates_many<'a, E>(
    conn: E,
    updates: Vec<(&Base64, Vec<u8>)>,
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let query = format!(
        r#"
            INSERT INTO outline_y_updates (id, outline_id, data)
            VALUES {};
        "#,
        updates
            .iter()
            .map(|_| "(?, ?, ?)".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for (id, data) in updates {
        query_builder = query_builder.bind(uuidv7::create_raw().to_vec());
        query_builder = query_builder.bind(id);
        query_builder = query_builder.bind(data);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn insert_card_y_updates_many<'a, E>(
    conn: E,
    updates: Vec<(&Base64, Vec<u8>)>,
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let query = format!(
        r#"
        INSERT INTO card_y_updates (id, card_id, data)
        VALUES {};
    "#,
        updates
            .iter()
            .map(|_| "(?, ?, ?)".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for (id, data) in updates {
        query_builder = query_builder.bind(uuidv7::create_raw().to_vec());
        query_builder = query_builder.bind(id);
        query_builder = query_builder.bind(data);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn delete_outline_y_updates<'a, E>(
    conn: E,
    y_update_ids: Vec<&Base64>,
) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let query = format!(
        r#"
            DELETE FROM outline_y_updates
            WHERE id IN ({});
        "#,
        y_update_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for id in y_update_ids {
        query_builder = query_builder.bind(id);
    }

    query_builder.execute(conn).await?;

    Ok(())
}

pub async fn delete_card_y_updates<'a, E>(conn: E, y_update_ids: Vec<&Base64>) -> anyhow::Result<()>
where
    E: SqliteExecutor<'a>,
{
    let query = format!(
        r#"
            DELETE FROM card_y_updates
            WHERE id IN ({});
        "#,
        y_update_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for id in y_update_ids {
        query_builder = query_builder.bind(id);
    }

    query_builder.execute(conn).await?;

    Ok(())
}
