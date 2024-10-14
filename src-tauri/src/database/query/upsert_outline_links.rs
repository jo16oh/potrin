use crate::types::util::Base64;
use sqlx::Sqlite;
use sqlx::Transaction;

pub async fn upsert_or_delete_outline_links<'a>(
    conn: &mut Transaction<'a, Sqlite>,
    outline_id: &Base64,
    links: &[Base64],
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

    for link in links {
        query_builder = query_builder.bind(link);
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

        for link in links {
            query_builder = query_builder.bind(outline_id);
            query_builder = query_builder.bind(link);
        }

        query_builder.execute(&mut **conn).await?;
    }

    Ok(())
}
