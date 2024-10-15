mod cjk_bigram_tokenizer;

use crate::types::state::AppState;
use crate::types::util::Base64;
use crate::utils::set_rw_state;
use anyhow::anyhow;
use cjk_bigram_tokenizer::CJKBigramTokenizer;
use diacritics::remove_diacritics;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::any::TypeId;
use std::fs;
use tantivy::collector::TopDocs;
use tantivy::directory::{ManagedDirectory, MmapDirectory};
use tantivy::query::QueryParser;
use tantivy::tokenizer::{Language, LowerCaser, Stemmer};
use tantivy::tokenizer::{TextAnalyzer, TokenizerManager};
use tantivy::{doc, schema::*, IndexReader};
use tantivy::{Index, IndexWriter};
use tauri::async_runtime::RwLock;
use tauri::test::MockRuntime;
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

pub struct Fields {
    id_field: Field,
    type_field: Field,
    text_field: Field,
}

pub async fn load_index<R: Runtime>(
    app_handle: &AppHandle<R>,
    levenshtein_distance: u8,
) -> anyhow::Result<()> {
    if levenshtein_distance != 0 && levenshtein_distance != 1 && levenshtein_distance != 2 {
        return Err(anyhow!("Levenstein distance must be between 0 and 2"));
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

    let fields = Fields {
        id_field: schema.get_field("id")?,
        type_field: schema.get_field("type")?,
        text_field: schema.get_field("text")?,
    };

    let index: Index = if TypeId::of::<R>() == TypeId::of::<MockRuntime>() {
        Ok(Index::create_in_ram(schema))
    } else {
        let mut path = app_handle.path().app_data_dir()?;
        let lock = app_handle.state::<RwLock<AppState>>().inner();
        let app_state = lock.read().await;
        let pot_id = &app_state
            .pot
            .as_ref()
            .ok_or(anyhow!("pot is not initialized"))?
            .id;
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

    let mut query_parser = QueryParser::new(
        index.schema(),
        vec![fields.text_field],
        tokenizer_manager_for_query,
    );
    query_parser.set_field_fuzzy(fields.text_field, true, levenshtein_distance, true);
    query_parser.set_conjunction_by_default();

    set_rw_state::<R, Fields>(app_handle, fields).await?;
    set_rw_state::<R, IndexReader>(app_handle, reader).await?;
    set_rw_state::<R, IndexWriter>(app_handle, writer).await?;
    set_rw_state::<R, QueryParser>(app_handle, query_parser).await?;

    Ok(())
}

pub async fn set_levenstein_distance(
    fields: &Fields,
    query_parser: &mut QueryParser,
    levenshtein_distance: u8,
) -> anyhow::Result<()> {
    if levenshtein_distance != 0 && levenshtein_distance != 1 && levenshtein_distance != 2 {
        return Err(anyhow!("Levenstein distance must be between 0 and 2"));
    }

    query_parser.set_field_fuzzy(fields.text_field, true, levenshtein_distance, true);
    query_parser.set_conjunction_by_default();

    Ok(())
}

pub async fn add_index(
    fields: &Fields,
    writer: &mut IndexWriter,
    input: Vec<IndexTarget>,
) -> anyhow::Result<()> {
    for item in input {
        let term = Term::from_field_text(fields.id_field, &item.id);
        writer.delete_term(term);

        let text = remove_diacritics(item.text.nfc().collect::<String>().as_str());

        writer.add_document(doc!(
            fields.id_field => item.id,
            fields.type_field => item.doc_type,
            fields.text_field => text
        ))?;
    }

    writer.commit()?;
    Ok(())
}

pub async fn remove_index(
    fields: &Fields,
    writer: &mut IndexWriter,
    target_ids: Vec<Base64>,
) -> anyhow::Result<()> {
    for id in target_ids {
        let term = Term::from_field_text(fields.id_field, &id.to_string());
        writer.delete_term(term);
    }

    writer.commit()?;
    Ok(())
}

pub async fn search(
    fields: &Fields,
    reader: &IndexReader,
    query_parser: &QueryParser,
    query: &str,
    limit: u8,
) -> anyhow::Result<Vec<SearchResult>> {
    let query = remove_diacritics(query.nfc().collect::<String>().as_str());
    let parsed_query = query_parser.parse_query(&query)?;
    let searcher = reader.searcher();

    let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(limit as usize))?;

    let mut results: Vec<SearchResult> = vec![];

    for (_, doc_addres) in top_docs {
        let retreived_doc = searcher.doc::<TantivyDocument>(doc_addres)?;
        let id_value = retreived_doc
            .get_first(fields.id_field)
            .ok_or(anyhow!("id field of the search result is not defined!"))?
            .as_str()
            .ok_or(anyhow!("id field of the search result is not defined!"))?;

        let type_value = retreived_doc
            .get_first(fields.type_field)
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
    use crate::{test::run_in_mock_app, utils::get_rw_state};
    use tauri::test::MockRuntime;

    #[test]
    fn test() {
        run_in_mock_app!(|app_handle: &AppHandle<MockRuntime>| async {
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

            let fields_lock = get_rw_state::<MockRuntime, Fields>(app_handle).unwrap();
            let reader_lock = get_rw_state::<MockRuntime, IndexReader>(app_handle).unwrap();
            let writer_lock = get_rw_state::<MockRuntime, IndexWriter>(app_handle).unwrap();
            let query_parser_lock = get_rw_state::<MockRuntime, QueryParser>(app_handle).unwrap();
            let fields = fields_lock.read().await;
            let reader = reader_lock.read().await;
            let mut writer = writer_lock.write().await;
            let mut query_parser = query_parser_lock.write().await;

            add_index(&fields, &mut writer, input).await.unwrap();
            reader.reload().unwrap();

            // prefix search
            assert_eq!(
                search(&fields, &reader, &query_parser, "c", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("1"),
                    doc_type: String::from("card")
                },]
            );

            // remove diacritics
            assert_eq!(
                search(&fields, &reader, &query_parser, "brulee", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("1"),
                    doc_type: String::from("card")
                },]
            );

            // NFC normalization
            assert_eq!(
                search(
                    &fields,
                    &reader,
                    &query_parser,
                    "brûlée".nfd().collect::<String>().as_str(),
                    100
                )
                .await
                .unwrap(),
                vec![SearchResult {
                    id: String::from("1"),
                    doc_type: String::from("card")
                },]
            );

            // english stemming
            assert_eq!(
                search(&fields, &reader, &query_parser, "connected", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("1"),
                    doc_type: String::from("card")
                },]
            );

            set_levenstein_distance(&fields, &mut query_parser, 2)
                .await
                .unwrap();
            // fuzzy search
            assert_eq!(
                search(&fields, &reader, &query_parser, "cantnt", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("1"),
                    doc_type: String::from("card")
                },]
            );
            set_levenstein_distance(&fields, &mut query_parser, 0)
                .await
                .unwrap();

            // japanese bigram
            assert_eq!(
                search(&fields, &reader, &query_parser, "はねだ", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("2"),
                    doc_type: String::from("thread")
                },]
            );

            // english and japanese compound
            assert_eq!(
                search(&fields, &reader, &query_parser, "羽田Airport", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("2"),
                    doc_type: String::from("thread")
                },]
            );

            // lowercase
            assert_eq!(
                search(&fields, &reader, &query_parser, "hnd", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("2"),
                    doc_type: String::from("thread")
                },]
            );

            // chinese bigram
            assert_eq!(
                search(&fields, &reader, &query_parser, "份有", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("3"),
                    doc_type: String::from("thread")
                },]
            );

            // search one character word on the end of the sentence
            assert_eq!(
                search(&fields, &reader, &query_parser, "草", 100)
                    .await
                    .unwrap(),
                vec![SearchResult {
                    id: String::from("4"),
                    doc_type: String::from("card")
                },]
            );
        });
    }
}
