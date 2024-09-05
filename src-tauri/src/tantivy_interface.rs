mod cjk_bigram_tokenizer;

use crate::utils::{get_once_lock, set_once_lock};
use anyhow::anyhow;
use cjk_bigram_tokenizer::CJKBigramTokenizer;
use diacritics::remove_diacritics;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::sync::{Mutex, OnceLock};
use tantivy::collector::TopDocs;
use tantivy::directory::{ManagedDirectory, MmapDirectory};
use tantivy::query::QueryParser;
use tantivy::tokenizer::{Language, LowerCaser, Stemmer};
use tantivy::tokenizer::{TextAnalyzer, TokenizerManager};
use tantivy::{doc, schema::*, IndexReader};
use tantivy::{Index, IndexWriter};
use tauri::{AppHandle, Manager, Runtime};
use unicode_normalization::UnicodeNormalization;

#[cfg_attr(debug_assertions, derive(Type, Debug))]
#[derive(Serialize, Deserialize)]
pub struct IndexTarget {
    pub id: String,
    pub doc_type: String,
    pub text: String,
}

#[cfg_attr(debug_assertions, derive(Type, Debug, PartialEq))]
#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    id: String,
    doc_type: String,
}

static INITIALIZED: OnceLock<()> = OnceLock::new();

static INDEX: OnceLock<Index> = OnceLock::new();
static READER: OnceLock<IndexReader> = OnceLock::new();
static WRITER: OnceLock<Mutex<IndexWriter>> = OnceLock::new();
static ID_FIELD: OnceLock<Field> = OnceLock::new();
static TYPE_FIELD: OnceLock<Field> = OnceLock::new();
static TEXT_FIELD: OnceLock<Field> = OnceLock::new();
static QUERY_PARSER: OnceLock<Mutex<QueryParser>> = OnceLock::new();

pub async fn init_tantivy<R: Runtime>(app_handle: Option<&AppHandle<R>>) -> anyhow::Result<()> {
    if INITIALIZED.get().is_some() {
        return Ok(());
    }

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("type", STORED);
    schema_builder.add_text_field(
        "text",
        TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("cjkbigram")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        ),
    );

    let schema = schema_builder.build();
    let id_field = schema.get_field("id")?;
    let type_field = schema.get_field("type")?;
    let text_field = schema.get_field("text")?;

    let index: Index = match app_handle {
        Some(handle) => {
            let mut path = handle.path().app_data_dir()?;
            path.push("tantivy");
            if !path.exists() {
                fs::create_dir_all(&path)?;
            }
            let dir = ManagedDirectory::wrap(Box::new(MmapDirectory::open(&path)?))?;
            Index::open_or_create(dir, schema)
        }
        None => Ok(Index::create_in_ram(schema)),
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

    let query_parser = Mutex::new(QueryParser::new(
        index.schema(),
        vec![text_field],
        tokenizer_manager_for_query,
    ));

    let _ = set_once_lock(&INDEX, index);
    let _ = set_once_lock(&READER, reader);
    let _ = set_once_lock(&WRITER, Mutex::new(writer));
    let _ = set_once_lock(&ID_FIELD, id_field);
    let _ = set_once_lock(&TYPE_FIELD, type_field);
    let _ = set_once_lock(&TEXT_FIELD, text_field);
    let _ = set_once_lock(&QUERY_PARSER, query_parser);

    let _ = set_once_lock(&INITIALIZED, ());
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn index(input: Vec<IndexTarget>) -> anyhow::Result<()> {
    let mut writer = get_once_lock(&WRITER)?
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?;
    let id_field = get_once_lock(&ID_FIELD)?;
    let type_field = get_once_lock(&TYPE_FIELD)?;
    let text_field = get_once_lock(&TEXT_FIELD)?;

    for item in input {
        let term = Term::from_field_text(*id_field, &item.id);
        writer.delete_term(term);

        let text = remove_diacritics(item.text.nfc().collect::<String>().as_str());

        writer.add_document(doc!(
            *id_field => item.id,
            *type_field => item.doc_type,
            *text_field => text
        ))?;
    }

    writer.commit()?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
#[macros::anyhow_to_string]
pub async fn search(
    query: &str,
    levenshtein_distance: u8,
    limit: u8,
) -> anyhow::Result<Vec<SearchResult>> {
    let mut results: Vec<SearchResult> = vec![];
    let query = remove_diacritics(query.nfc().collect::<String>().as_str());

    let searcher = get_once_lock(&READER)?.searcher();
    let id_field = get_once_lock(&ID_FIELD)?;
    let type_field = get_once_lock(&TYPE_FIELD)?;
    let text_field = get_once_lock(&TEXT_FIELD)?;

    let mut query_parser = get_once_lock(&QUERY_PARSER)?
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?;
    query_parser.set_field_fuzzy(*text_field, true, levenshtein_distance, true);
    query_parser.set_conjunction_by_default();

    let query_parsed = query_parser.parse_query(&query)?;

    let top_docs = searcher.search(&query_parsed, &TopDocs::with_limit(limit as usize))?;

    for (_, doc_addres) in top_docs {
        let retreived_doc = searcher.doc::<TantivyDocument>(doc_addres)?;
        let id_value = retreived_doc
            .get_first(*id_field)
            .ok_or(anyhow!("id field of the search result is not defined!"))?
            .as_str()
            .ok_or(anyhow!("id field of the search result is not defined!"))?;

        let type_value = retreived_doc
            .get_first(*type_field)
            .ok_or(anyhow!("type field of the search result is not defined!"))?
            .as_str()
            .ok_or(anyhow!("type field of the search result is not defined!"))?;

        results.push(SearchResult {
            id: id_value.to_string(),
            doc_type: type_value.to_string(),
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::MockRuntime;

    #[tokio::test]
    async fn test() {
        let _ = init_tantivy::<MockRuntime>(None).await;

        let input = vec![
            IndexTarget {
                id: String::from("1"),
                doc_type: String::from("card"),
                text: String::from("content brûlée connection"),
            },
            IndexTarget {
                id: String::from("2"),
                doc_type: String::from("thread"),
                text: String::from("東京国際空港（とうきょうこくさいくうこう、英語: Tokyo International Airport）は、東京都大田区にある日本最大の空港。通称は羽田空港（はねだくうこう、英語: Haneda Airport）であり、単に「羽田」と呼ばれる場合もある。空港コードはHND。"),
            },
            IndexTarget {
                id: String::from("3"),
                doc_type: String::from("thread"),
                text: String::from("股份有限公司"),
            },
            IndexTarget {
                id: String::from("4"),
                doc_type: String::from("card"),
                text: String::from("デカすぎで草"),
            },
        ];

        let _ = index(input).await;
        READER.get().unwrap().reload().unwrap();

        // prefix search
        assert_eq!(
            search("c", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("1"),
                doc_type: String::from("card")
            },]
        );

        // remove diacritics
        assert_eq!(
            search("brulee", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("1"),
                doc_type: String::from("card")
            },]
        );

        // NFC normalization
        assert_eq!(
            search("brûlée".nfd().collect::<String>().as_str(), 0, 100)
                .await
                .unwrap(),
            vec![SearchResult {
                id: String::from("1"),
                doc_type: String::from("card")
            },]
        );

        // english stemming
        assert_eq!(
            search("connected", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("1"),
                doc_type: String::from("card")
            },]
        );

        // fuzzy search
        assert_eq!(
            search("cantnt", 2, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("1"),
                doc_type: String::from("card")
            },]
        );

        // japanese bigram
        assert_eq!(
            search("はねだ", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("2"),
                doc_type: String::from("thread")
            },]
        );

        // english and japanese compound
        assert_eq!(
            search("羽田Airport", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("2"),
                doc_type: String::from("thread")
            },]
        );

        // lowercase
        assert_eq!(
            search("hnd", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("2"),
                doc_type: String::from("thread")
            },]
        );

        // chinese bigram
        assert_eq!(
            search("份有", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("3"),
                doc_type: String::from("thread")
            },]
        );

        // search one character word on the end of the sentence
        assert_eq!(
            search("草", 0, 100).await.unwrap(),
            vec![SearchResult {
                id: String::from("4"),
                doc_type: String::from("card")
            },]
        );
    }
}
