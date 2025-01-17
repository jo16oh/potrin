use sqlx::{Sqlite, Transaction};

use crate::types::{
    model::{Links, Quote},
    util::UUIDv7Base64URL,
};

pub async fn outline_links<'a>(
    conn: &mut Transaction<'a, Sqlite>,
    outline_id: UUIDv7Base64URL,
    links: &Links,
) -> eyre::Result<()> {
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

pub async fn paragraph_links<'a>(
    conn: &mut Transaction<'a, Sqlite>,
    paragraph_id: UUIDv7Base64URL,
    links: &Links,
) -> eyre::Result<()> {
    let query = format!(
        r#"
            DELETE FROM paragraph_links
            WHERE id_from = ? AND id_to NOT IN ({});
        "#,
        links
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    query_builder = query_builder.bind(paragraph_id);

    for (id_to, _) in links.iter() {
        query_builder = query_builder.bind(id_to);
    }

    query_builder.execute(&mut **conn).await?;

    if !links.is_empty() {
        let query = format!(
            r#"
                INSERT INTO paragraph_links (id_from, id_to)
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
            query_builder = query_builder.bind(paragraph_id);
            query_builder = query_builder.bind(id_to);
        }

        query_builder.execute(&mut **conn).await?;
    }

    Ok(())
}

pub async fn quote<'a>(
    conn: &mut Transaction<'a, Sqlite>,
    paragraph_id: UUIDv7Base64URL,
    quote: &Option<Quote>,
) -> eyre::Result<()> {
    if let Some(quote) = quote {
        sqlx::query!(
            r#"
                INSERT INTO quotes (paragraph_id, quote_id, version_id)
                VALUES (?, ?, ?)
                ON CONFLICT
                DO UPDATE
                SET
                    quote_id = excluded.quote_id,
                    version_id = excluded.version_id;
            "#,
            paragraph_id,
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
                WHERE paragraph_id = ?;
            "#,
            paragraph_id
        )
        .execute(&mut **conn)
        .await?;
    }

    Ok(())
}
