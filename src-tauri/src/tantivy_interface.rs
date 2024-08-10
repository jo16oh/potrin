use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tantivy::collector::TopDocs;
use tantivy::directory::{ManagedDirectory, MmapDirectory};
use tantivy::query::QueryParser;
use tantivy::{doc, schema::*, IndexReader};
use tantivy::{Index, IndexWriter};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexTarget {
    id: String,
    doc_type: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

fn set_once_lock<T>(lock: &OnceLock<T>, value: T) -> anyhow::Result<()> {
    lock.set(value)
        .map_err(|_| anyhow!("Failed to set value to OnceLock"))?;
    Ok(())
}

fn get_once_lock<T>(lock: &OnceLock<T>) -> anyhow::Result<&T> {
    let result = lock
        .get()
        .ok_or_else(|| anyhow!("Failed to get value of OnceLock"))?;
    Ok(result)
}

#[tauri::command]
#[macros::anyhow_to_string]
pub fn init(app_handle: AppHandle) -> anyhow::Result<()> {
    if let Some(_) = INITIALIZED.get() {
        return Ok(());
    }

    let app_dir = app_handle.path().app_data_dir()?;
    let path = Path::join(&app_dir, "tantivy");

    if !path.exists() {
        fs::create_dir_all(path.clone())?;
    }

    let dir = ManagedDirectory::wrap(Box::new(MmapDirectory::open(path)?))?;

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("type", STRING | STORED);
    schema_builder.add_text_field("text", TEXT);
    let schema = schema_builder.build();
    let id_field = schema.get_field("id")?;
    let text_field = schema.get_field("text")?;
    let index = Index::open_or_create(dir, schema)?;
    let reader = index.reader()?;
    let writer = index.writer(100_000_000)?;

    set_once_lock(&INDEX, index)?;
    set_once_lock(&READER, reader)?;
    set_once_lock(&WRITER, Mutex::new(writer))?;
    set_once_lock(&ID_FIELD, id_field)?;
    set_once_lock(&TYPE_FIELD, id_field)?;
    set_once_lock(&TEXT_FIELD, text_field)?;

    set_once_lock(&INITIALIZED, ())?;
    Ok(())
}

#[tauri::command]
#[macros::anyhow_to_string]
pub fn index(input: Vec<IndexTarget>) -> anyhow::Result<()> {
    let mut writer = get_once_lock(&WRITER)?
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?;
    let id_field = get_once_lock(&ID_FIELD)?;
    let type_field = get_once_lock(&TYPE_FIELD)?;
    let text_field = get_once_lock(&TEXT_FIELD)?;

    for item in input {
        let term = Term::from_field_text(*id_field, &item.id);
        writer.delete_term(term);

        writer.add_document(doc!(
            *id_field => item.id,
            *type_field => item.doc_type,
            *text_field => item.text
        ))?;
    }

    writer.commit()?;
    Ok(())
}

#[tauri::command]
#[macros::anyhow_to_string]
pub fn search(
    input: &str,
    levenshtein_distance: u8,
    limit: usize,
) -> anyhow::Result<Vec<SearchResult>> {
    let mut results: Vec<SearchResult> = vec![];

    let index = get_once_lock(&INDEX)?;
    let searcher = get_once_lock(&READER)?.searcher();
    let id_field = get_once_lock(&ID_FIELD)?;
    let type_field = get_once_lock(&TYPE_FIELD)?;
    let text_field = get_once_lock(&TEXT_FIELD)?;

    let mut query_parser = QueryParser::for_index(&index, vec![*text_field]);
    query_parser.set_field_fuzzy(*text_field, true, levenshtein_distance, true);
    query_parser.set_conjunction_by_default();

    let query = query_parser.parse_query(&input)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

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

    #[test]
    fn test() {
        init_test();

        let input = vec![
            IndexTarget {
                id: String::from("id"),
                doc_type: String::from("card"),
                text: String::from("content"),
            },
            IndexTarget {
                id: String::from("id2"),
                doc_type: String::from("thread"),
                text: String::from("title"),
            },
        ];

        let _ = index(input);

        READER.get().unwrap().reload().unwrap();

        assert_eq!(
            search("cantnt", 2, 100).unwrap(),
            vec![SearchResult {
                id: String::from("id"),
                doc_type: String::from("card")
            },]
        );

        assert_eq!(
            search("title", 2, 100).unwrap(),
            vec![SearchResult {
                id: String::from("id2"),
                doc_type: String::from("thread")
            },]
        );
    }

    fn init_test() {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("type", STRING | STORED);
        schema_builder.add_text_field("text", TEXT);
        let schema = schema_builder.build();
        let id_field = schema.get_field("id").unwrap();
        let type_field = schema.get_field("type").unwrap();
        let text_field = schema.get_field("text").unwrap();
        let index = Index::create_in_ram(schema);
        let reader = index.reader().map_err(|e| e.to_string()).unwrap();
        let writer = index.writer(100_000_000).unwrap();

        INDEX.set(index).unwrap();
        READER
            .set(reader)
            .map_err(|_| "failed to set THREADS_READER")
            .unwrap();
        WRITER
            .set(Mutex::new(writer))
            .map_err(|_| "failed to set THREADS_WRITER")
            .unwrap();
        ID_FIELD.set(id_field).unwrap();
        TYPE_FIELD.set(type_field).unwrap();
        TEXT_FIELD.set(text_field).unwrap();

        INITIALIZED.set(()).unwrap();
    }
}
