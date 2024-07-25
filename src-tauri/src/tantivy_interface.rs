use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{doc, schema::*, IndexReader};
use tantivy::{Index, IndexWriter};

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

pub fn init() {
    if let Some(_) = INITIALIZED.get() {
        return;
    }

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

#[tauri::command]
pub fn index(json: &str) -> Result<(), String> {
    let index_targets: IndexTargets = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let handle_card = thread::spawn(|| {
        let mut cards_writer = CARDS_WRITER.get().unwrap().lock().unwrap();
        let cards_id_field = CARDS_ID_FIELD.get().unwrap();
        let cards_content_field = CARDS_CONTENT_FIELD.get().unwrap();

        for card in index_targets.cards {
            let term = Term::from_field_text(*cards_id_field, &card.id);
            cards_writer.delete_term(term);

            cards_writer
                .add_document(doc!(
                    *cards_id_field => card.id,
                    *cards_content_field => card.content
                ))
                .unwrap();
        }

        cards_writer.commit().unwrap();
    });

    let handle_thread = thread::spawn(|| {
        let mut threads_writer = THREADS_WRITER.get().unwrap().lock().unwrap();
        let threads_id_field = THREADS_ID_FIELD.get().unwrap();
        let threads_title_field = THREADS_TITLE_FIELD.get().unwrap();

        for thread in index_targets.threads {
            let term = Term::from_field_text(*threads_id_field, &thread.id);
            threads_writer.delete_term(term);

            threads_writer
                .add_document(doc!(
                    *threads_id_field => thread.id,
                    *threads_title_field => thread.title
                ))
                .unwrap();
        }

        threads_writer.commit().unwrap();
    });

    handle_card.join().unwrap();
    handle_thread.join().unwrap();

    Ok(())
}

#[tauri::command]
pub fn search(
    input: &str,
    levenshtein_distance: u8,
    limit: usize,
) -> Result<SearchResults, String> {
    let thread_ids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let card_ids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let input = input.to_string();

    let input_clone = input.clone();
    let limit_clone = limit.clone();
    let card_ids_clone = Arc::clone(&card_ids);
    let handle_card = thread::spawn(move || {
        let mut ids = card_ids_clone.lock().unwrap();
        let index = CARDS_INDEX.get().unwrap();
        let searcher = CARDS_READER.get().unwrap().searcher();
        let id_field = CARDS_ID_FIELD.get().unwrap();
        let content_field = CARDS_CONTENT_FIELD.get().unwrap();
        let mut query_parser = QueryParser::for_index(&index, vec![*content_field]);
        query_parser.set_field_fuzzy(*content_field, true, levenshtein_distance, true);
        query_parser.set_conjunction_by_default();
        let query = query_parser
            .parse_query(&input_clone)
            .map_err(|e| e.to_string())
            .unwrap();

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit_clone))
            .map_err(|e| e.to_string())
            .unwrap();

        for (_, doc_addres) in top_docs {
            let retreived_doc = searcher
                .doc::<TantivyDocument>(doc_addres)
                .map_err(|e| e.to_string())
                .unwrap();
            let id_value = retreived_doc
                .get_first(*id_field)
                .ok_or("id field of the search result is not defined!")
                .unwrap();

            if let OwnedValue::Str(id) = id_value {
                ids.push(id.clone());
            }
        }
    });

    let thread_ids_clone = Arc::clone(&thread_ids);
    let handle_thread = thread::spawn(move || {
        let mut ids = thread_ids_clone.lock().unwrap();
        let index = THREADS_INDEX.get().unwrap();
        let searcher = THREADS_READER.get().unwrap().searcher();
        let id_field = THREADS_ID_FIELD.get().unwrap();
        let title_field = THREADS_TITLE_FIELD.get().unwrap();
        let mut query_parser = QueryParser::for_index(&index, vec![*title_field]);
        query_parser.set_field_fuzzy(*title_field, true, levenshtein_distance, true);
        query_parser.set_conjunction_by_default();
        let thread_query = query_parser
            .parse_query(&input)
            .map_err(|e| e.to_string())
            .unwrap();

        let thread_top_docs = searcher
            .search(&thread_query, &TopDocs::with_limit(limit))
            .map_err(|e| e.to_string())
            .unwrap();

        for (_, doc_addres) in thread_top_docs {
            let retreived_doc = searcher
                .doc::<TantivyDocument>(doc_addres)
                .map_err(|e| e.to_string())
                .unwrap();
            let id_value = retreived_doc
                .get_first(*id_field)
                .ok_or("id field of the search result is not defined!")
                .unwrap();

            if let OwnedValue::Str(id) = id_value {
                ids.push(id.clone());
            }
        }
    });

    handle_card.join().unwrap();
    handle_thread.join().unwrap();

    let thread_ids: Vec<String> = thread_ids.lock().unwrap().to_vec();
    let card_ids: Vec<String> = card_ids.lock().unwrap().to_vec();

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
        init();

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
}
