use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::thread::JoinHandle;
use std::{fs, thread};
use tantivy::collector::TopDocs;
use tantivy::directory::{ManagedDirectory, MmapDirectory};
use tantivy::query::QueryParser;
use tantivy::{doc, schema::*, IndexReader};
use tantivy::{Index, IndexWriter};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize)]
struct Thread {
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize)]
struct Card {
    id: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct IndexTargets {
    threads: Vec<Thread>,
    cards: Vec<Card>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    threads: Vec<String>,
    cards: Vec<String>,
}

static INITIALIZED: OnceLock<()> = OnceLock::new();

static THREADS_INDEX: OnceLock<Index> = OnceLock::new();
static THREADS_WRITER: OnceLock<Mutex<IndexWriter>> = OnceLock::new();
static THREADS_READER: OnceLock<IndexReader> = OnceLock::new();
static THREADS_ID_FIELD: OnceLock<Field> = OnceLock::new();
static THREADS_TITLE_FIELD: OnceLock<Field> = OnceLock::new();

static CARDS_INDEX: OnceLock<Index> = OnceLock::new();
static CARDS_READER: OnceLock<IndexReader> = OnceLock::new();
static CARDS_WRITER: OnceLock<Mutex<IndexWriter>> = OnceLock::new();
static CARDS_ID_FIELD: OnceLock<Field> = OnceLock::new();
static CARDS_CONTENT_FIELD: OnceLock<Field> = OnceLock::new();

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

fn join_handle<T>(handle: JoinHandle<anyhow::Result<T>>) -> anyhow::Result<T> {
    handle.join().map_err(|e| {
        if let Some(string) = e.downcast_ref::<String>() {
            anyhow!("Thread panicked: {}", string)
        } else if let Some(&str) = e.downcast_ref::<&str>() {
            anyhow!("Thread panicked: {}", str)
        } else {
            anyhow!("Thread panicked with unknown error")
        }
    })?
}

fn lock_arc_mutex<T>(mutex: &Arc<Mutex<T>>) -> anyhow::Result<MutexGuard<'_, T>> {
    mutex.lock().map_err(|e| anyhow!(e.to_string()))
}

#[tauri::command]
#[macros::anyhow_to_string]
pub fn init(app_handle: AppHandle) -> anyhow::Result<()> {
    if let Some(_) = INITIALIZED.get() {
        return Ok(());
    }

    let app_dir = app_handle.path().app_data_dir()?;
    let card_path = Path::join(&app_dir, "tantivy/cards");
    let thread_path = Path::join(&app_dir, "tantivy/threads");

    if !card_path.exists() {
        fs::create_dir_all(card_path.clone())?;
    }

    if !thread_path.exists() {
        fs::create_dir_all(thread_path.clone())?;
    }

    let card_dir = ManagedDirectory::wrap(Box::new(MmapDirectory::open(card_path)?))?;
    let thread_dir = ManagedDirectory::wrap(Box::new(MmapDirectory::open(thread_path)?))?;

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("title", TEXT);
    let schema = schema_builder.build();
    let id_field = schema.get_field("id")?;
    let title_field = schema.get_field("title")?;
    let index = Index::open_or_create(thread_dir, schema)?;
    let reader = index.reader()?;
    let writer = index.writer(100_000_000)?;

    set_once_lock(&THREADS_INDEX, index)?;
    set_once_lock(&THREADS_READER, reader)?;
    set_once_lock(&THREADS_WRITER, Mutex::new(writer))?;
    set_once_lock(&THREADS_ID_FIELD, id_field)?;
    set_once_lock(&THREADS_TITLE_FIELD, title_field)?;

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("content", TEXT);
    let schema = schema_builder.build();
    let id_field = schema.get_field("id")?;
    let content_field = schema.get_field("content")?;
    let index = Index::open_or_create(card_dir, schema)?;
    let reader = index.reader()?;
    let writer = index.writer(100_000_000)?;

    set_once_lock(&CARDS_INDEX, index)?;
    set_once_lock(&CARDS_READER, reader)?;
    set_once_lock(&CARDS_WRITER, Mutex::new(writer))?;
    set_once_lock(&CARDS_ID_FIELD, id_field)?;
    set_once_lock(&CARDS_CONTENT_FIELD, content_field)?;

    set_once_lock(&INITIALIZED, ())?;
    Ok(())
}

#[tauri::command]
#[macros::anyhow_to_string]
pub fn index(json: &str) -> anyhow::Result<()> {
    let index_targets: IndexTargets = serde_json::from_str(json)?;

    let handle_card = thread::spawn(|| -> anyhow::Result<()> {
        let mut cards_writer = get_once_lock(&CARDS_WRITER)?
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?;
        let cards_id_field = get_once_lock(&CARDS_ID_FIELD)?;
        let cards_content_field = get_once_lock(&CARDS_CONTENT_FIELD)?;

        for card in index_targets.cards {
            let term = Term::from_field_text(*cards_id_field, &card.id);
            cards_writer.delete_term(term);

            cards_writer.add_document(doc!(
                *cards_id_field => card.id,
                *cards_content_field => card.content
            ))?;
        }

        cards_writer.commit()?;

        Ok(())
    });

    let handle_thread = thread::spawn(|| -> anyhow::Result<()> {
        let mut threads_writer = get_once_lock(&THREADS_WRITER)?
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?;
        let threads_id_field = get_once_lock(&THREADS_ID_FIELD)?;
        let threads_title_field = get_once_lock(&THREADS_TITLE_FIELD)?;

        for thread in index_targets.threads {
            let term = Term::from_field_text(*threads_id_field, &thread.id);
            threads_writer.delete_term(term);

            threads_writer.add_document(doc!(
                *threads_id_field => thread.id,
                *threads_title_field => thread.title
            ))?;
        }

        threads_writer.commit()?;

        Ok(())
    });

    join_handle(handle_thread)?;
    join_handle(handle_card)?;

    Ok(())
}

#[tauri::command]
#[macros::anyhow_to_string]
pub fn search(
    input: &str,
    levenshtein_distance: u8,
    limit: usize,
) -> anyhow::Result<SearchResults> {
    let thread_ids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let card_ids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let input = input.to_string();

    let input_clone = input.clone();
    let limit_clone = limit.clone();
    let card_ids_clone = Arc::clone(&card_ids);
    let handle_card = thread::spawn(move || -> anyhow::Result<()> {
        let mut ids = lock_arc_mutex(&card_ids_clone)?;
        let index = get_once_lock(&CARDS_INDEX)?;
        let searcher = get_once_lock(&CARDS_READER)?.searcher();
        let id_field = get_once_lock(&CARDS_ID_FIELD)?;
        let content_field = get_once_lock(&CARDS_CONTENT_FIELD)?;
        let mut query_parser = QueryParser::for_index(&index, vec![*content_field]);
        query_parser.set_field_fuzzy(*content_field, true, levenshtein_distance, true);
        query_parser.set_conjunction_by_default();
        let query = query_parser.parse_query(&input_clone)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit_clone))?;

        for (_, doc_addres) in top_docs {
            let retreived_doc = searcher.doc::<TantivyDocument>(doc_addres)?;
            let id_value = retreived_doc
                .get_first(*id_field)
                .ok_or(anyhow!("id field of the search result is not defined!"))?;

            if let OwnedValue::Str(id) = id_value {
                ids.push(id.clone());
            }
        }

        Ok(())
    });

    let thread_ids_clone = Arc::clone(&thread_ids);
    let handle_thread = thread::spawn(move || -> anyhow::Result<()> {
        let mut ids = lock_arc_mutex(&thread_ids_clone)?;
        let index = get_once_lock(&THREADS_INDEX)?;
        let searcher = get_once_lock(&THREADS_READER)?.searcher();
        let id_field = get_once_lock(&THREADS_ID_FIELD)?;
        let title_field = get_once_lock(&THREADS_TITLE_FIELD)?;
        let mut query_parser = QueryParser::for_index(&index, vec![*title_field]);
        query_parser.set_field_fuzzy(*title_field, true, levenshtein_distance, true);
        query_parser.set_conjunction_by_default();
        let thread_query = query_parser.parse_query(&input)?;

        let thread_top_docs = searcher.search(&thread_query, &TopDocs::with_limit(limit))?;

        for (_, doc_addres) in thread_top_docs {
            let retreived_doc = searcher.doc::<TantivyDocument>(doc_addres)?;
            let id_value = retreived_doc
                .get_first(*id_field)
                .ok_or(anyhow!("id field of the search result is not defined!"))?;

            if let OwnedValue::Str(id) = id_value {
                ids.push(id.clone());
            }
        }

        Ok(())
    });

    join_handle(handle_card)?;
    join_handle(handle_thread)?;

    let thread_ids: Vec<String> = lock_arc_mutex(&thread_ids)?.to_vec();
    let card_ids: Vec<String> = lock_arc_mutex(&card_ids)?.to_vec();

    Ok(SearchResults {
        threads: thread_ids,
        cards: card_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        init_test();

        let json = r#"
            {
                "cards": [{"id": "id", "content": "content"}],
                "threads": [{"id": "id", "title": "title"}]
            }"#;
        index(json).unwrap();

        CARDS_READER.get().unwrap().reload().unwrap();
        THREADS_READER.get().unwrap().reload().unwrap();

        let res = search("cantnt", 2, 100).unwrap();
        assert_eq!(res.cards, vec!["id"]);
        let res = search("title", 2, 100).unwrap();
        assert_eq!(res.threads, vec!["id"]);
    }

    fn init_test() {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT);
        let schema = schema_builder.build();
        let id_field = schema.get_field("id").unwrap();
        let title_field = schema.get_field("title").unwrap();
        let index = Index::create_in_ram(schema);
        let reader = index.reader().map_err(|e| e.to_string()).unwrap();
        let writer = index.writer(100_000_000).unwrap();

        THREADS_INDEX.set(index).unwrap();
        THREADS_READER
            .set(reader)
            .map_err(|_| "failed to set THREADS_READER")
            .unwrap();
        THREADS_WRITER
            .set(Mutex::new(writer))
            .map_err(|_| "failed to set THREADS_WRITER")
            .unwrap();
        THREADS_ID_FIELD.set(id_field).unwrap();
        THREADS_TITLE_FIELD.set(title_field).unwrap();

        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("content", TEXT);
        let schema = schema_builder.build();
        let id_field = schema.get_field("id").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let index = Index::create_in_ram(schema);
        let reader = index.reader().map_err(|e| e.to_string()).unwrap();
        let writer = index.writer(100_000_000).unwrap();

        CARDS_INDEX.set(index).unwrap();
        CARDS_READER
            .set(reader)
            .map_err(|_| "failed to set CARDS_READER")
            .unwrap();
        CARDS_WRITER
            .set(Mutex::new(writer))
            .map_err(|_| "failed to set CARDS_READER")
            .unwrap();
        CARDS_ID_FIELD.set(id_field).unwrap();
        CARDS_CONTENT_FIELD.set(content_field).unwrap();

        INITIALIZED.set(()).unwrap();
    }
}
