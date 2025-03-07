mod cjk_bigram_tokenizer;

use crate::types::model::{Links, Path};
use crate::types::setting::SearchFuzziness;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::{extract_text_from_doc, get_state};
use cjk_bigram_tokenizer::CJKBigramTokenizer;
use diacritics::remove_diacritics;
use eyre::OptionExt;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::any::TypeId;
use std::fs;
use tantivy::collector::TopDocs;
use tantivy::directory::{ManagedDirectory, MmapDirectory};
use tantivy::query::QueryParser;
use tantivy::tokenizer::{Language, LowerCaser, Stemmer};
use tantivy::tokenizer::{TextAnalyzer, TokenizerManager};
use tantivy::{doc, schema::*, DocAddress, IndexReader};
use tantivy::{Index, IndexWriter};
use tauri::async_runtime::{Mutex, RwLock};
use tauri::test::MockRuntime;
use tauri::{AppHandle, Manager, Runtime};
use unicode_normalization::UnicodeNormalization;

pub struct IndexTarget<'a> {
    pub id: UUIDv7Base64URL,
    pub pot_id: UUIDv7Base64URL,
    pub doc_type: &'a str,
    pub doc: &'a str,
    pub path: &'a Path,
    pub links: &'a Links,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(sqlx::FromRow)]
pub struct DeleteTarget {
    pub id: UUIDv7Base64URL,
    pub pot_id: UUIDv7Base64URL,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Serialize, Deserialize, Type)]
pub struct SearchResult {
    pub id: UUIDv7Base64URL,
    pub doc_type: String,
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum OrderBy {
    CreatedAt(Order),
    UpdatedAt(Order),
    Relevance,
}

#[derive(Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Order {
    Desc,
    Asc,
}

pub struct Fields {
    id: Field,
    doc_type: Field,
    text: Field,
    path: Field,
    links: Field,
    hidden: Field,
    created_at: Field,
    updated_at: Field,
}

pub struct SearchIndex {
    fields: Fields,
    reader: IndexReader,
    writer: Mutex<IndexWriter>,
    parser: RwLock<QueryParser>,
    fuzziness: RwLock<SearchFuzziness>,
}

pub async fn load_index<R: Runtime>(
    app_handle: &AppHandle<R>,
    pot_id: UUIDv7Base64URL,
    search_fuzziness: SearchFuzziness,
) -> eyre::Result<SearchIndex> {
    let levenshtein_distance = match search_fuzziness {
        SearchFuzziness::Exact => 0,
        SearchFuzziness::Fuzzy => 1,
        SearchFuzziness::Fuzziest => 2,
    };

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("doc_type", STRING | STORED);
    schema_builder.add_text_field(
        "text",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("cjkbigram")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );
    schema_builder.add_i64_field("created_at", INDEXED | FAST);
    schema_builder.add_i64_field("updated_at", INDEXED | FAST);
    schema_builder.add_facet_field("path", FacetOptions::default().set_stored());
    schema_builder.add_facet_field("links", FacetOptions::default());
    schema_builder.add_bool_field("hidden", NumericOptions::default());

    let schema = schema_builder.build();

    let fields = Fields {
        id: schema.get_field("id")?,
        doc_type: schema.get_field("doc_type")?,
        text: schema.get_field("text")?,
        path: schema.get_field("path")?,
        links: schema.get_field("links")?,
        hidden: schema.get_field("hidden")?,
        created_at: schema.get_field("created_at")?,
        updated_at: schema.get_field("updated_at")?,
    };

    let index: Index = if TypeId::of::<R>() == TypeId::of::<MockRuntime>() {
        Ok(Index::create_in_ram(schema))
    } else {
        let mut path = app_handle.path().app_data_dir()?;

        path.push("search_engine");
        path.push(pot_id.to_string());
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        let dir = ManagedDirectory::wrap(Box::new(MmapDirectory::open(&path)?))?;
        Index::open_or_create(dir, schema)
    }?;

    let reader = index.reader()?;
    let writer = index.writer(100_000_000)?;

    let tokenizer = TextAnalyzer::builder(CJKBigramTokenizer::new())
        .filter(Stemmer::new(Language::English))
        .filter(LowerCaser)
        .build();
    index.tokenizers().register("cjkbigram", tokenizer);

    let tokenizer_manager_for_query = TokenizerManager::new();
    let tokenizer_for_query = TextAnalyzer::builder(CJKBigramTokenizer::new().for_query())
        .filter(Stemmer::new(Language::English))
        .filter(LowerCaser)
        .build();
    tokenizer_manager_for_query.register("cjkbigram", tokenizer_for_query);

    let mut parser = QueryParser::new(
        index.schema(),
        vec![fields.text],
        tokenizer_manager_for_query,
    );
    parser.set_field_fuzzy(fields.text, true, levenshtein_distance, true);
    parser.set_conjunction_by_default();

    Ok(SearchIndex {
        fields,
        reader,
        writer: Mutex::new(writer),
        parser: RwLock::new(parser),
        fuzziness: RwLock::new(search_fuzziness),
    })
}

pub async fn add_index<R: Runtime>(
    app_handle: &AppHandle<R>,
    index_targets: Vec<IndexTarget<'_>>,
) -> eyre::Result<()> {
    let windows = app_handle.webview_windows();
    let targets_map = index_targets.into_iter().into_group_map_by(|t| t.pot_id);

    for (pot_id, targets) in targets_map.into_iter() {
        if let Some(win) = windows.get(&pot_id.to_string()) {
            let index = get_state::<R, SearchIndex>(win)?;
            process_targets(index, targets).await?;
        } else {
            let index = load_index(app_handle, pot_id, SearchFuzziness::default()).await?;
            process_targets(&index, targets).await?;
        };
    }

    Ok(())
}

async fn process_targets(
    index: &SearchIndex,
    index_targets: Vec<IndexTarget<'_>>,
) -> eyre::Result<()> {
    let mut writer = index.writer.lock().await;

    for item in index_targets {
        let term = Term::from_field_text(index.fields.id, &item.id.to_string());
        writer.delete_term(term);

        if item.doc.chars().all(|c| c.is_whitespace()) {
            continue;
        }

        let text = extract_text_from_doc(item.doc)
            .map(|text| remove_diacritics(text.nfc().collect::<String>().as_str()))?;

        let mut document = doc!(
            index.fields.id => item.id.to_string(),
            index.fields.doc_type => item.doc_type,
            index.fields.text => text,
            index.fields.created_at => item.created_at,
            index.fields.updated_at => item.updated_at,
        );

        {
            let path: Vec<&str> = item.path.iter().map(|e| e.text.as_str()).collect();
            document.add_facet(index.fields.path, Facet::from_path(path));
        }

        for (_, path) in item.links.iter() {
            let mut texts: Vec<&str> = Vec::new();
            let mut hidden: bool = false;

            for link in path.iter() {
                texts.push(&link.text);
                hidden = hidden || link.hidden;
            }

            document.add_facet(index.fields.links, Facet::from_path(texts));
            document.add_bool(index.fields.hidden, hidden);
        }

        writer.add_document(document)?;
    }

    writer.commit()?;

    Ok(())
}

pub async fn remove_index<R: Runtime>(
    app_handle: &AppHandle<R>,
    targets: &[DeleteTarget],
) -> eyre::Result<()> {
    let windows = app_handle.webview_windows();
    let targets_map = targets.iter().into_group_map_by(|t| t.pot_id);

    for (pot_id, targets) in targets_map.into_iter() {
        if let Some(win) = windows.get(&pot_id.to_string()) {
            let index = get_state::<R, SearchIndex>(win)?;
            process_targets(targets, index).await?;
        } else {
            let index = load_index(app_handle, pot_id, SearchFuzziness::default()).await?;
            process_targets(targets, &index).await?;
        };
    }

    async fn process_targets(targets: Vec<&DeleteTarget>, index: &SearchIndex) -> eyre::Result<()> {
        let mut writer = index.writer.lock().await;

        for DeleteTarget { id, .. } in targets {
            let term = Term::from_field_text(index.fields.id, &id.to_string());
            writer.delete_term(term);
        }

        writer.commit()?;

        Ok(())
    }

    Ok(())
}

pub async fn search(
    index: &SearchIndex,
    query: &str,
    order_by: OrderBy,
    limit: u8,
    search_fuzziness: SearchFuzziness,
) -> eyre::Result<Vec<SearchResult>> {
    if search_fuzziness != *index.fuzziness.read().await {
        let mut query_parser = index.parser.write().await;

        let levenshtein_distance = search_fuzziness.levenshtein_distance();
        query_parser.set_field_fuzzy(index.fields.text, true, levenshtein_distance, true);
        query_parser.set_conjunction_by_default();

        let mut f = index.fuzziness.write().await;
        *f = search_fuzziness;
    }

    let query_parser = index.parser.read().await;

    let query = remove_diacritics(query.nfc().collect::<String>().as_str());
    let parsed_query = query_parser.parse_query(&query)?;
    let searcher = index.reader.searcher();

    let doc_addresses: Vec<DocAddress> = match order_by {
        OrderBy::CreatedAt(order) => {
            let order = match order {
                Order::Desc => tantivy::Order::Desc,
                Order::Asc => tantivy::Order::Asc,
            };

            let collector =
                TopDocs::with_limit(limit as usize).order_by_fast_field::<i64>("created_at", order);

            searcher
                .search(&parsed_query, &collector)?
                .into_iter()
                .map(|(_, doc_address)| doc_address)
                .collect()
        }
        OrderBy::UpdatedAt(order) => {
            let order = match order {
                Order::Desc => tantivy::Order::Desc,
                Order::Asc => tantivy::Order::Asc,
            };

            let collector =
                TopDocs::with_limit(limit as usize).order_by_fast_field::<i64>("updated_at", order);

            searcher
                .search(&parsed_query, &collector)?
                .into_iter()
                .map(|(_, doc_address)| doc_address)
                .collect()
        }
        OrderBy::Relevance => {
            let collector = TopDocs::with_limit(limit as usize);

            searcher
                .search(&parsed_query, &collector)?
                .into_iter()
                .map(|(_, doc_address)| doc_address)
                .collect()
        }
    };

    let mut results: Vec<SearchResult> = vec![];

    for doc_address in doc_addresses {
        let retreived_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        let id_value = retreived_doc
            .get_first(index.fields.id)
            .ok_or_eyre("id field of the search result is not defined!")?
            .as_str()
            .ok_or_eyre("id field of the search result is not defined!")?;

        let type_value = retreived_doc
            .get_first(index.fields.doc_type)
            .ok_or_eyre("type field of the search result is not defined!")?
            .as_str()
            .ok_or_eyre("type field of the search result is not defined!")?;

        results.push(SearchResult {
            id: id_value.try_into()?,
            doc_type: type_value.to_string(),
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_in_mock_app;
    use chrono::Utc;
    use tauri::test::MockRuntime;

    #[test]
    fn test() {
        run_in_mock_app!(test_impl);
    }

    async fn test_impl(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pot_id = UUIDv7Base64URL::new();

        let one = UUIDv7Base64URL::new();
        let two = UUIDv7Base64URL::new();
        let three = UUIDv7Base64URL::new();
        let four = UUIDv7Base64URL::new();
        let links = Links::new();
        let path = Path::new();
        let input = vec![
            IndexTarget {
                id: one,
                pot_id,
                doc_type: "paragraph",
                doc: r#"{ "text": "content brûlée connection" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
            IndexTarget {
                id: two,
                pot_id,
                doc_type: "outline",
                doc: r#"{ "text": "東京国際空港（とうきょうこくさいくうこう、英語: Tokyo International Airport）は、東京都大田区にある日本最大の空港。通称は羽田空港（はねだくうこう、英語: Haneda Airport）であり、単に「羽田」と呼ばれる場合もある。空港コードはHND。" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
            IndexTarget {
                id: three,
                pot_id,
                doc_type: "outline",
                doc: r#"{ "text": "股份有限公司" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
            IndexTarget {
                id: four,
                pot_id,
                doc_type: "paragraph",
                doc: r#"{ "text": "デカすぎで草" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
        ];

        let index = load_index(app_handle, pot_id, SearchFuzziness::Exact)
            .await
            .unwrap();

        process_targets(&index, input).await.unwrap();
        index.reader.reload().unwrap();

        // prefix search
        assert_eq!(
            search(&index, "c", OrderBy::Relevance, 100, SearchFuzziness::Exact)
                .await
                .unwrap(),
            vec![SearchResult {
                id: one,
                doc_type: String::from("paragraph")
            },]
        );

        // remove diacritics
        assert_eq!(
            search(
                &index,
                "brulee",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: one,
                doc_type: String::from("paragraph")
            },]
        );

        // NFC normalization
        assert_eq!(
            search(
                &index,
                "brûlée".nfd().collect::<String>().as_str(),
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: one,
                doc_type: String::from("paragraph")
            },]
        );

        // english stemming
        assert_eq!(
            search(
                &index,
                "connected",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: one,
                doc_type: String::from("paragraph")
            },]
        );

        // fuzzy search
        assert_eq!(
            search(
                &index,
                "cantnt",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Fuzziest
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: one,
                doc_type: String::from("paragraph")
            },]
        );

        // japanese bigram
        assert_eq!(
            search(
                &index,
                "はねだ",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: two,
                doc_type: String::from("outline")
            },]
        );

        // english and japanese compound
        assert_eq!(
            search(
                &index,
                "羽田Airport",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: two,
                doc_type: String::from("outline")
            },]
        );

        // lowercase
        assert_eq!(
            search(
                &index,
                "hnd",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: two,
                doc_type: String::from("outline")
            },]
        );

        // chinese bigram
        assert_eq!(
            search(
                &index,
                "份有",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: three,
                doc_type: String::from("outline")
            },]
        );

        // search one character word on the end of the sentence
        assert_eq!(
            search(
                &index,
                "草",
                OrderBy::Relevance,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: four,
                doc_type: String::from("paragraph")
            },]
        );

        Ok(())
    }
}
