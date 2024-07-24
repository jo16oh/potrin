use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
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

static THREADS_INDEX: Lazy<Index> = Lazy::new(|| {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("title", TEXT);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);
    index
});

static THREADS_WRITER: Lazy<Mutex<IndexWriter>> = Lazy::new(|| {
    let writer = THREADS_INDEX.writer(100_000_000).unwrap();
    Mutex::new(writer)
});

static THREADS_READER: Lazy<IndexReader> =
    Lazy::new(|| THREADS_INDEX.reader().map_err(|e| e.to_string()).unwrap());

static THREADS_SCHEMA: Lazy<Schema> = Lazy::new(|| THREADS_INDEX.schema());

static THREADS_ID_FIELD: Lazy<Field> = Lazy::new(|| THREADS_SCHEMA.get_field("id").unwrap());

static THREADS_TITLE_FIELD: Lazy<Field> = Lazy::new(|| THREADS_SCHEMA.get_field("title").unwrap());

static CARDS_INDEX: Lazy<Index> = Lazy::new(|| {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("content", TEXT);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);
    index
});

static CARDS_SCHEMA: Lazy<Schema> = Lazy::new(|| CARDS_INDEX.schema());

static CARDS_ID_FIELD: Lazy<Field> = Lazy::new(|| CARDS_SCHEMA.get_field("id").unwrap());

static CARDS_CONTENT_FIELD: Lazy<Field> = Lazy::new(|| CARDS_SCHEMA.get_field("content").unwrap());

static CARDS_WRITER: Lazy<Mutex<IndexWriter>> = Lazy::new(|| {
    let writer = CARDS_INDEX.writer(100_000_000).unwrap();
    Mutex::new(writer)
});

static CARDS_READER: Lazy<IndexReader> =
    Lazy::new(|| CARDS_INDEX.reader().map_err(|e| e.to_string()).unwrap());

#[tauri::command]
pub fn index(json: &str) -> Result<(), String> {
    let mut cards_writer = CARDS_WRITER.lock().unwrap();
    let mut threads_writer = THREADS_WRITER.lock().unwrap();

    let index_targets: IndexTargets = serde_json::from_str(json).map_err(|e| e.to_string())?;

    for card in index_targets.cards {
        let term = Term::from_field_text(*CARDS_ID_FIELD, &card.id);
        cards_writer.delete_term(term);

        cards_writer
            .add_document(doc!(
                *CARDS_ID_FIELD => card.id,
                *CARDS_CONTENT_FIELD => card.content
            ))
            .map_err(|e| e.to_string())?;
    }

    for thread in index_targets.threads {
        let term = Term::from_field_text(*THREADS_ID_FIELD, &thread.id);
        threads_writer.delete_term(term);

        threads_writer
            .add_document(doc!(
                *THREADS_ID_FIELD => thread.id,
                *THREADS_TITLE_FIELD => thread.title
            ))
            .map_err(|e| e.to_string())?;
    }

    threads_writer.commit().map_err(|e| e.to_string())?;
    cards_writer.commit().map_err(|e| e.to_string())?;

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
        let searcher = CARDS_READER.searcher();
        let mut query_parser = QueryParser::for_index(&CARDS_INDEX, vec![*CARDS_CONTENT_FIELD]);
        query_parser.set_field_fuzzy(*CARDS_CONTENT_FIELD, true, levenshtein_distance, true);
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
                .get_first(*CARDS_ID_FIELD)
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
        let searcher = THREADS_READER.searcher();
        let mut query_parser = QueryParser::for_index(&THREADS_INDEX, vec![*THREADS_ID_FIELD]);
        query_parser.set_field_fuzzy(*THREADS_TITLE_FIELD, true, levenshtein_distance, true);
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
                .get_first(*THREADS_ID_FIELD)
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
        let json = r#"
            {
                "cards": [{"id": "id", "content": "content"}],
                "threads": [{"id": "id", "title": "title"}]
            }"#;
        let _ = index(json);
        let __ = index(json);

        let res = search("cantnt", 2, 100).unwrap();

        assert_eq!(res.cards, vec!["id"]);
    }
}
