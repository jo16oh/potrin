use crate::types::model::Link;
use crate::types::util::Base64;
use anyhow::anyhow;
use sqlx::SqlitePool;

pub async fn fetch_links(
    pool: &SqlitePool,
    outline_ids: &[&Base64],
    card_ids: &[&Base64],
) -> anyhow::Result<Vec<Link>> {
    let query = format!(
        r#"
            WITH RECURSIVE links AS (
                SELECT outlines.id, outlines.parent_id, outlines.text 
                FROM outlines 
                INNER JOIN outline_links ON outlines.id = outline_links.id_to
                WHERE outline_links.id_from IN ({})
                UNION
                SELECT outlines.id, outlines.parent_id, outlines.text
                FROM outlines
                INNER JOIN card_links ON outlines.id = card_links.id_to
                WHERE card_links.id_from IN ({})
                UNION 
                SELECT outlines.id, outlines.parent_id, outlines.text 
                FROM outlines
                INNER JOIN links ON outlines.id = links.parent_id 
            )
            SELECT * FROM links;
        "#,
        outline_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", "),
        card_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", "),
    );

    let mut query_builder = sqlx::query_as::<_, Link>(&query);

    for id in outline_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    query_builder.fetch_all(pool).await.map_err(|e| anyhow!(e))
}
