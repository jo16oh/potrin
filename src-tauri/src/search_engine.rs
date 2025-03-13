mod cjk_bigram_tokenizer;

use crate::state::SearchIndices;
use crate::types::model::{Links, Path};
use crate::types::setting::SearchFuzziness;
use crate::types::util::UUIDv7Base64URL;
use crate::utils::{extract_text_from_doc, get_rw_state};
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
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::tokenizer::{Language, LowerCaser, Stemmer};
use tantivy::tokenizer::{TextAnalyzer, TokenizerManager};
use tantivy::{doc, schema::*, DocAddress, IndexReader};
use tantivy::{Index, IndexWriter};
use tauri::async_runtime::{Mutex, RwLock};
use tauri::test::MockRuntime;
use tauri::{AppHandle, Manager, Runtime};
use unicode_normalization::UnicodeNormalization;

#[derive(Clone)]
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, Type)]
#[serde(rename_all = "camelCase")]
pub enum OrderBy {
    CreatedAt(Order),
    UpdatedAt(Order),
    #[default]
    Relevance,
}

#[derive(Serialize, Deserialize, Clone, Debug, Type)]
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
    let targets_map = index_targets.into_iter().into_group_map_by(|t| t.pot_id);

    for (pot_id, targets) in targets_map.into_iter() {
        let search_indices_lock = get_rw_state::<R, SearchIndices>(app_handle)?;
        let search_indices = search_indices_lock.read().await;

        if let Some(index) = search_indices.get(&pot_id) {
            process_targets(index, targets).await?;
        } else {
            let index = load_index(app_handle, pot_id, SearchFuzziness::default()).await?;
            process_targets(&index, targets).await?;
        }
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
            let mut path: Vec<String> = item.path.iter().map(|e| e.id.to_string()).collect();

            if item.doc_type == "outline" {
                path.pop();
            }

            if !path.is_empty() {
                document.add_facet(index.fields.path, Facet::from_path(path));
            }
        }

        for (_, path) in item.links.iter() {
            let mut texts: Vec<String> = Vec::new();
            let mut hidden: bool = false;

            for link in path.iter() {
                texts.push(link.id.to_string());
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
    let targets_map = targets.iter().into_group_map_by(|t| t.pot_id);

    for (pot_id, targets) in targets_map.into_iter() {
        let search_indices_lock = get_rw_state::<R, SearchIndices>(app_handle)?;
        let search_indices = search_indices_lock.read().await;

        if let Some(index) = search_indices.get(&pot_id) {
            process_targets(index, targets).await?;
        } else {
            let index = load_index(app_handle, pot_id, SearchFuzziness::default()).await?;
            process_targets(&index, targets).await?;
        }
    }

    async fn process_targets(index: &SearchIndex, targets: Vec<&DeleteTarget>) -> eyre::Result<()> {
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
    scope: Option<Vec<UUIDv7Base64URL>>,
    order_by: OrderBy,
    offset: u32,
    limit: u32,
    search_fuzziness: SearchFuzziness,
) -> eyre::Result<Vec<SearchResult>> {
    if search_fuzziness != *index.fuzziness.read().await {
        let mut query_parser = index.parser.write().await;

        let levenshtein_distance = search_fuzziness.levenshtein_distance();
        query_parser.set_field_fuzzy(index.fields.text, true, levenshtein_distance, true);

        let mut f = index.fuzziness.write().await;
        *f = search_fuzziness;
    }

    let searcher = index.reader.searcher();

    let query = {
        let query_parser = index.parser.read().await;
        let query = remove_diacritics(query.nfc().collect::<String>().as_str());
        let (parsed_query, _) = query_parser.parse_query_lenient(&query);

        if let Some(path) = scope {
            let path: Vec<String> = path.iter().map(|p| p.to_string()).collect();

            let facet_query = TermQuery::new(
                Term::from_facet(index.fields.path, &Facet::from_path(path)),
                IndexRecordOption::Basic,
            );

            Box::new(BooleanQuery::new(vec![
                (Occur::Must, Box::new(facet_query)),
                (Occur::Must, parsed_query),
            ]))
        } else {
            parsed_query
        }
    };

    let doc_addresses: Vec<DocAddress> = match order_by {
        OrderBy::CreatedAt(order) => {
            let order = match order {
                Order::Desc => tantivy::Order::Desc,
                Order::Asc => tantivy::Order::Asc,
            };

            let collector = TopDocs::with_limit(limit as usize)
                .and_offset(offset as usize)
                .order_by_fast_field::<i64>("created_at", order);

            searcher
                .search(&query, &collector)?
                .into_iter()
                .map(|(_, doc_address)| doc_address)
                .collect()
        }
        OrderBy::UpdatedAt(order) => {
            let order = match order {
                Order::Desc => tantivy::Order::Desc,
                Order::Asc => tantivy::Order::Asc,
            };

            let collector = TopDocs::with_limit(limit as usize)
                .and_offset(offset as usize)
                .order_by_fast_field::<i64>("updated_at", order);

            searcher
                .search(&query, &collector)?
                .into_iter()
                .map(|(_, doc_address)| doc_address)
                .collect()
        }
        OrderBy::Relevance => {
            let collector = TopDocs::with_limit(limit as usize).and_offset(offset as usize);

            searcher
                .search(&query, &collector)?
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
    use crate::{test::run_in_mock_app, types::model::Link};
    use chrono::Utc;
    use tauri::test::MockRuntime;

    #[test]
    fn test() {
        run_in_mock_app!(test_impl);
    }

    async fn test_impl(app_handle: &AppHandle<MockRuntime>) -> eyre::Result<()> {
        let pot_id = UUIDv7Base64URL::new();

        let links = Links::new();
        let path = Path::new();

        let links_for_path = vec![
            Link {
                id: UUIDv7Base64URL::new(),
                text: "".to_string(),
                hidden: false,
            },
            Link {
                id: UUIDv7Base64URL::new(),
                text: "".to_string(),
                hidden: false,
            },
        ];

        let path1 = Path::from(vec![links_for_path[0].clone()]);
        let path2 = Path::from(links_for_path.clone());

        let input = vec![
            IndexTarget {
                id: UUIDv7Base64URL::new(),
                pot_id,
                doc_type: "paragraph",
                doc: r#"{ "text": "content brûlée connection" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
            IndexTarget {
                id: UUIDv7Base64URL::new(),
                pot_id,
                doc_type: "outline",
                doc: r#"{ "text": "東京国際空港（とうきょうこくさいくうこう、英語: Tokyo International Airport）は、東京都大田区にある日本最大の空港。通称は羽田空港（はねだくうこう、英語: Haneda Airport）であり、単に「羽田」と呼ばれる場合もある。空港コードはHND。" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
            IndexTarget {
                id: UUIDv7Base64URL::new(),
                pot_id,
                doc_type: "outline",
                doc: r#"{ "text": "股份有限公司" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
            IndexTarget {
                id: UUIDv7Base64URL::new(),
                pot_id,
                doc_type: "paragraph",
                doc: r#"{ "text": "デカすぎで草" }"#,
                path: &path,
                links: &links,
                created_at: Utc::now().timestamp_millis(),
                updated_at: Utc::now().timestamp_millis(),
            },
            IndexTarget {
                id: UUIDv7Base64URL::new(),
                pot_id,
                doc_type: "paragraph",
                doc: r#"{ "text": "path" }"#,
                path: &path1,
                links: &links,
                created_at: 1,
                updated_at: 1,
            },
            IndexTarget {
                id: UUIDv7Base64URL::new(),
                pot_id,
                doc_type: "paragraph",
                doc: r#"{ "text": "path" }"#,
                path: &path2,
                links: &links,
                created_at: 2,
                updated_at: 2,
            },
        ];

        let index = load_index(app_handle, pot_id, SearchFuzziness::Exact)
            .await
            .unwrap();

        process_targets(&index, input.clone()).await.unwrap();
        index.reader.reload().unwrap();

        // prefix search
        assert_eq!(
            search(
                &index,
                "c",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[0].id,
                doc_type: String::from("paragraph")
            },]
        );

        // remove diacritics
        assert_eq!(
            search(
                &index,
                "brulee",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[0].id,
                doc_type: String::from("paragraph")
            },]
        );

        // NFC normalization
        assert_eq!(
            search(
                &index,
                "brûlée".nfd().collect::<String>().as_str(),
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[0].id,
                doc_type: String::from("paragraph")
            },]
        );

        // english stemming
        assert_eq!(
            search(
                &index,
                "connected",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[0].id,
                doc_type: String::from("paragraph")
            },]
        );

        // fuzzy search
        assert_eq!(
            search(
                &index,
                "cantnt",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Fuzziest
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[0].id,
                doc_type: String::from("paragraph")
            },]
        );

        // japanese bigram
        assert_eq!(
            search(
                &index,
                "はねだ",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[1].id,
                doc_type: String::from("outline")
            },]
        );

        // english and japanese compound
        assert_eq!(
            search(
                &index,
                "羽田Airport",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[1].id,
                doc_type: String::from("outline")
            },]
        );

        // lowercase
        assert_eq!(
            search(
                &index,
                "hnd",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[1].id,
                doc_type: String::from("outline")
            },]
        );

        // chinese bigram
        assert_eq!(
            search(
                &index,
                "份有",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[2].id,
                doc_type: String::from("outline")
            },]
        );

        // search one character word on the end of the sentence
        assert_eq!(
            search(
                &index,
                "草",
                None,
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[3].id,
                doc_type: String::from("paragraph")
            },]
        );

        // search inside certain path
        assert_eq!(
            search(
                &index,
                "path",
                Some(links_for_path.iter().map(|e| e.id).collect::<Vec<_>>()[0..1].to_vec()),
                OrderBy::CreatedAt(Order::Asc),
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![
                SearchResult {
                    id: input[4].id,
                    doc_type: String::from("paragraph")
                },
                SearchResult {
                    id: input[5].id,
                    doc_type: String::from("paragraph")
                },
            ]
        );

        // search inside certain path
        assert_eq!(
            search(
                &index,
                "path",
                Some(links_for_path.iter().map(|e| e.id).collect()),
                OrderBy::Relevance,
                0,
                100,
                SearchFuzziness::Exact
            )
            .await
            .unwrap(),
            vec![SearchResult {
                id: input[5].id,
                doc_type: String::from("paragraph")
            },]
        );

        Ok(())
    }
}
