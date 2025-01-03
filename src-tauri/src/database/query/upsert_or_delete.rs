use sqlx::{Sqlite, Transaction};

use crate::types::{
    model::{Links, Quote},
    util::UUIDv7Base64URL,
};

pub async fn outline_links<'a>(
    conn: &mut Transaction<'a, Sqlite>,
    outline_id: UUIDv7Base64URL,
    links: &Links,
) -> anyhow::Result<()> {
    let query = format!(
        r#"
            DELETE FROM outline_links
            WHERE id_from = ? AND id_to NOT IN ({});
        "#,
        links
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    query_builder = query_builder.bind(outline_id);

    for (id_to, _) in links.iter() {
        query_builder = query_builder.bind(id_to);
    }

    query_builder.execute(&mut **conn).await?;

    if !links.is_empty() {
        let query = format!(
            r#"
                INSERT INTO outline_links (id_from, id_to)
                VALUES {}
                ON CONFLICT
                DO NOTHING;
            "#,
            links
                .iter()
                .map(|_| "(?, ?)".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query(&query);

        for (id_to, _) in links.iter() {
            query_builder = query_builder.bind(outline_id);
            query_builder = query_builder.bind(id_to);
        }

        query_builder.execute(&mut **conn).await?;
    }

    Ok(())
}

pub async fn card_links<'a>(
    conn: &mut Transaction<'a, Sqlite>,
    card_id: UUIDv7Base64URL,
    links: &Links,
) -> anyhow::Result<()> {
    let query = format!(
        r#"
            DELETE FROM card_links
            WHERE id_from = ? AND id_to NOT IN ({});
        "#,
        links
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    query_builder = query_builder.bind(card_id);

    for (id_to, _) in links.iter() {
        query_builder = query_builder.bind(id_to);
    }

    query_builder.execute(&mut **conn).await?;

    if !links.is_empty() {
        let query = format!(
            r#"
                INSERT INTO card_links (id_from, id_to)
                VALUES {}
                ON CONFLICT
                DO NOTHING;
            "#,
            links
                .iter()
                .map(|_| "(?, ?)".to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let mut query_builder = sqlx::query(&query);

        for (id_to, _) in links.iter() {
            query_builder = query_builder.bind(card_id);
            query_builder = query_builder.bind(id_to);
        }

        query_builder.execute(&mut **conn).await?;
    }

    Ok(())
}

pub async fn quote<'a>(
    conn: &mut Transaction<'a, Sqlite>,
    card_id: UUIDv7Base64URL,
    quote: &Option<Quote>,
) -> anyhow::Result<()> {
    if let Some(quote) = quote {
        sqlx::query!(
            r#"
                INSERT INTO quotes (card_id, quote_id, version_id)
                VALUES (?, ?, ?)
                ON CONFLICT 
                DO UPDATE
                SET 
                    quote_id = excluded.quote_id,
                    version_id = excluded.version_id;
            "#,
            card_id,
            quote.id,
            quote.version_id
        )
        .execute(&mut **conn)
        .await?;
    } else {
        sqlx::query!(
            r#"
                DELETE 
                FROM quotes
                WHERE card_id = ?;
            "#,
            card_id
        )
        .execute(&mut **conn)
        .await?;
    }

    Ok(())
}
