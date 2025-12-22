use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub mod build;
pub mod parse;
pub mod rank;
pub mod types;

use crate::types::{SourcesConfig, SourcesStore};
use types::{DocumentLengths, InvertedIndex, SearchResult};

/// Search engine state with pre-computed caches
struct SearchEngine {
    sources: SourcesStore,
    index: InvertedIndex,
    doc_lengths: DocumentLengths,
    avg_doc_length: f32,
    num_docs: u16,
    sources_config: SourcesConfig,
}

impl SearchEngine {
    /// Create a new search engine from a document store and config
    fn from_docs(all_sources: SourcesStore, sources_config: SourcesConfig) -> Self {
        // Build inverted index
        let (index, doc_lengths) = build::build_sources_index(&all_sources);

        // Calculate average document length
        let avg_doc_length = if doc_lengths.is_empty() {
            0.0
        } else {
            doc_lengths
                .values()
                .map(|field_lens: &HashMap<String, u16>| field_lens.values().sum::<u16>() as f32)
                .sum::<f32>()
                / doc_lengths.len() as f32
        };

        let num_docs = all_sources.values().map(|docs| docs.len()).sum::<usize>() as u16;

        SearchEngine {
            sources: all_sources,
            index,
            doc_lengths,
            avg_doc_length,
            num_docs,
            sources_config,
        }
    }

    fn search(&self, query: &str, n: usize) -> Vec<SearchResult> {
        let start_total = js_sys::Date::now();

        let query_trigrams = parse::parse_query(query);
        let trigram_count = query_trigrams.len();

        let start_scoring = js_sys::Date::now();

        // Accumulate scores for each document (deduplicate by doc, not by field)
        let mut overall_scores: HashMap<(String, String), f32> = HashMap::new();
        let mut total_postings = 0usize;
        let prune_count = 0usize;

        for trigram in &query_trigrams {
            if let Some(postings) = self.index.get(trigram) {
                let doc_freq = postings.len() as u16;
                total_postings += postings.len();

                for (source_id, doc_id, term_freq, field_name) in postings {
                    // Use cached document length (skip if missing)
                    let doc_len = match self.doc_lengths.get(doc_id) {
                        Some(field_lens) => field_lens.get(field_name).copied().unwrap_or(1),
                        None => continue,
                    };

                    // Get field weight from config (default to 0.0 if not configured)
                    let field_weight = self
                        .sources_config
                        .get(source_id)
                        .and_then(|cfg| cfg.weights.get(field_name).copied())
                        .unwrap_or(0.0);

                    let score = rank::bm25_term(
                        *term_freq,
                        doc_len,
                        doc_freq,
                        self.num_docs,
                        self.avg_doc_length,
                    ) * field_weight;

                    overall_scores
                        .entry((source_id.clone(), doc_id.clone()))
                        .and_modify(|s| *s += score)
                        .or_insert(score);
                }
            }
        }

        let scoring_time = js_sys::Date::now() - start_scoring;
        let start_ranking = js_sys::Date::now();

        // Convert to vector and get top N
        let score_vec: Vec<(f32, (String, String))> = overall_scores
            .into_iter()
            .map(|(doc_path, score)| (score, doc_path))
            .collect();
        let top_results = rank::top_n(&score_vec, n);

        let ranking_time = js_sys::Date::now() - start_ranking;
        let start_fetch = js_sys::Date::now();

        // Map to SearchResult with full document info (filter out any missing docs)
        let results: Vec<SearchResult> = top_results
            .into_iter()
            .filter_map(|(score, (source_id, doc_id))| {
                let doc = self.sources.get(&source_id)?.get(&doc_id)?.clone();
                Some(SearchResult {
                    document: doc,
                    score,
                })
            })
            .collect();

        let fetch_time = js_sys::Date::now() - start_fetch;
        let total_time = js_sys::Date::now() - start_total;

        // Log performance stats
        web_sys::console::log_1(&format!(
            "[perf] query='{}' trigrams={} postings={} prunes={} | score={:.1}ms rank={:.1}ms fetch={:.1}ms | total={:.1}ms",
            query, trigram_count, total_postings, prune_count,
            scoring_time, ranking_time, fetch_time, total_time
        ).into());

        results
    }
}

// Use thread_local with RefCell for lazy initialization from JS
thread_local! {
    static ENGINE: RefCell<Option<SearchEngine>> = const { RefCell::new(None) };
}

/// Initialize the search engine with documents and config from JavaScript
/// docs_json: JSON string of SourcesStore (HashMap<source_id, HashMap<doc_id, HashMap<field, value>>>)
/// config_json: JSON string of SourcesConfig (HashMap<source_id, SourceConfig>)
#[wasm_bindgen]
pub fn init_engine(docs_json: &str, config_json: &str) -> Result<(), JsError> {
    let sources: SourcesStore = serde_json::from_str(docs_json)
        .map_err(|e| JsError::new(&format!("Failed to parse documents: {}", e)))?;

    let sources_config: SourcesConfig = serde_json::from_str(config_json)
        .map_err(|e| JsError::new(&format!("Failed to parse config: {}", e)))?;

    ENGINE.with(|engine| {
        *engine.borrow_mut() = Some(SearchEngine::from_docs(sources, sources_config));
    });

    Ok(())
}

/// Check if the engine has been initialized
#[wasm_bindgen]
pub fn is_engine_ready() -> bool {
    ENGINE.with(|engine| engine.borrow().is_some())
}

/// Search all documents and return JSON results
#[wasm_bindgen]
pub fn search_docs(query: &str, n: usize) -> Result<String, JsError> {
    ENGINE.with(|engine| {
        let engine_ref = engine.borrow();
        match engine_ref.as_ref() {
            Some(eng) => {
                let results = eng.search(query, n);
                Ok(serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string()))
            }
            None => Err(JsError::new(
                "Engine not initialized. Call init_engine(docs_json, config_json) first.",
            )),
        }
    })
}

/// Get total number of documents
#[wasm_bindgen]
pub fn get_doc_count() -> Result<usize, JsError> {
    ENGINE.with(|engine| {
        let engine_ref = engine.borrow();
        match engine_ref.as_ref() {
            Some(eng) => Ok(eng.sources.values().map(|docs| docs.len()).sum()),
            None => Err(JsError::new(
                "Engine not initialized. Call init_engine(docs_json, config_json) first.",
            )),
        }
    })
}

/// Greet function for testing WASM is working
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_engine() {
        // Sample test documents: source_id -> doc_id -> field_name -> value
        let test_docs = r#"{
            "buildings": {
                "doc1": {"name": "Baker Hall", "desc": "A building on campus"},
                "doc2": {"name": "Gates Center", "desc": "Computer science building"}
            },
            "courses": {
                "doc3": {"name": "Programming 101", "desc": "Introduction to programming"}
            }
        }"#;

        // Config with field weights by name
        let test_config = r#"{
            "buildings": {"url": "", "weights": {"name": 2.0, "desc": 1.0}},
            "courses": {"url": "", "weights": {"name": 2.0, "desc": 1.0}}
        }"#;

        init_engine(test_docs, test_config).expect("Failed to initialize test engine");
    }

    #[test]
    fn test_search_docs() {
        setup_test_engine();
        let results_json = search_docs("baker hall", 5).unwrap();
        let results: Vec<SearchResult> = serde_json::from_str(&results_json).unwrap();

        // Verify we get results
        assert!(!results.is_empty(), "Search should return results");

        // Verify results have scores > 0
        assert!(
            results.iter().all(|r| r.score > 0.0),
            "All results should have positive scores"
        );
    }

    #[test]
    fn test_search_courses() {
        setup_test_engine();
        let results_json = search_docs("programming", 5).unwrap();
        let results: Vec<SearchResult> = serde_json::from_str(&results_json).unwrap();

        // Verify we get results
        assert!(!results.is_empty(), "Search should return results");
    }

    #[test]
    fn test_doc_count() {
        setup_test_engine();
        let count = get_doc_count().unwrap();
        assert!(count > 0);
        println!("Total documents: {}", count);
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
