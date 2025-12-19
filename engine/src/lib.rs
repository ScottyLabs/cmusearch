use wasm_bindgen::prelude::*;
use serde_json;
use std::collections::HashMap;

pub mod types;
pub mod parse;
pub mod build;
pub mod rank;

use types::{DocumentStore, InvertedIndex, SearchResult, CourseStore, RoomStore};

// Static data - will be initialized once
static COURSES_JSON: &str = include_str!("documents/courses.json");
static ROOMS_JSON: &str = include_str!("documents/roomDocuments.json");

/// Initialize the search engine state with pre-computed caches
struct SearchEngine {
    docs: DocumentStore,
    index: InvertedIndex,
    doc_lengths: HashMap<String, u16>,  // Pre-computed document lengths
    avg_doc_length: f32,
    num_docs: u16,  // Pre-cached doc count
}

impl SearchEngine {
    fn new() -> Self {
        // Parse courses from embedded JSON
        let courses: CourseStore = serde_json::from_str(COURSES_JSON)
            .expect("Failed to parse courses.json");
        
        // Parse rooms from embedded JSON
        let rooms: RoomStore = serde_json::from_str(ROOMS_JSON)
            .expect("Failed to parse roomDocuments.json");
        
        // Convert to unified document store
        let course_docs = build::courses_to_docs(courses);
        let room_docs = build::rooms_to_docs(rooms);
        let all_docs = build::merge_docs(course_docs, room_docs);
        
        // Build inverted index
        let index = build::build_index(&all_docs);
        
        // Pre-compute document lengths (expensive operation cached here)
        let doc_lengths: HashMap<String, u16> = all_docs
            .iter()
            .map(|(id, doc)| (id.clone(), build::get_doc_length(doc)))
            .collect();
        
        // Calculate average document length
        let avg_doc_length = if doc_lengths.is_empty() {
            0.0
        } else {
            doc_lengths.values().map(|&l| l as f32).sum::<f32>() / doc_lengths.len() as f32
        };
        
        let num_docs = all_docs.len() as u16;
        
        SearchEngine {
            docs: all_docs,
            index,
            doc_lengths,
            avg_doc_length,
            num_docs,
        }
    }
    
    fn search(&self, query: &str, n: usize) -> Vec<SearchResult> {
        let query_trigrams = parse::parse_query(query);
        
        // Accumulate scores for each document
        let mut overall_scores: HashMap<String, f32> = HashMap::new();
        
        for trigram in &query_trigrams {
            if let Some(postings) = self.index.get(trigram) {
                let doc_freq = postings.len() as u16;
                
                for (doc_id, term_freq) in postings {
                    // Use cached document length
                    let doc_len = *self.doc_lengths.get(doc_id).unwrap_or(&0);
                    let score = rank::bm25_term(
                        *term_freq,
                        doc_len,
                        doc_freq,
                        self.num_docs,
                        self.avg_doc_length,
                    );
                    
                    overall_scores
                        .entry(doc_id.clone())
                        .and_modify(|s| *s += score)
                        .or_insert(score);
                }
            }
        }
        
        // Convert to vector and get top N
        let score_vec: Vec<(String, f32)> = overall_scores.into_iter().collect();
        let top_results = rank::top_n(&score_vec, n);
        
        // Map to SearchResult with full document info
        top_results
            .into_iter()
            .map(|(doc_id, score)| SearchResult {
                document: self.docs.get(&doc_id).unwrap().clone(),
                score,
            })
            .collect()
    }
}

// Use thread_local for WASM compatibility
thread_local! {
    static ENGINE: SearchEngine = SearchEngine::new();
}

/// Search all documents and return JSON results
#[wasm_bindgen]
pub fn search_docs(query: &str, n: usize) -> String {
    ENGINE.with(|engine| {
        let results = engine.search(query, n);
        serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
    })
}

/// Get total number of documents
#[wasm_bindgen]
pub fn get_doc_count() -> usize {
    ENGINE.with(|engine| engine.docs.len())
}

/// Greet function for testing WASM is working
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_docs() {
        let results_json = search_docs("baker hall", 5);
        let results: Vec<SearchResult> = serde_json::from_str(&results_json).unwrap();
        
        // Verify we get results
        assert!(!results.is_empty(), "Search should return results");
        
        // Verify results have scores > 0
        assert!(results.iter().all(|r| r.score > 0.0), "All results should have positive scores");
    }

    #[test]
    fn test_search_courses() {
        let results_json = search_docs("programming", 5);
        let results: Vec<SearchResult> = serde_json::from_str(&results_json).unwrap();
        
        // Verify we get results
        assert!(!results.is_empty(), "Search should return results");
    }

    #[test]
    fn test_doc_count() {
        let count = get_doc_count();
        assert!(count > 0);
        // Should have both courses and rooms
        println!("Total documents: {}", count);
    }

    #[test]
    fn test_greet() {
        assert_eq!(greet("World"), "Hello, World!");
    }
}
