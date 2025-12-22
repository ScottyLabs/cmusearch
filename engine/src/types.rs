use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Document: field_name -> field_value
pub type Document = HashMap<String, String>;

/// Document store: document_id -> Document
pub type DocumentStore = HashMap<String, Document>;

/// Sources store: source_id -> DocumentStore
pub type SourcesStore = HashMap<String, DocumentStore>;

/// Search result with document info and relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
}

/// Document lengths: document_id -> field_name -> length
pub type DocumentLengths = HashMap<String, HashMap<String, u16>>;

/// Inverted index: term -> list of (source_id, document_id, term_frequency, field_name)
pub type InvertedIndex = HashMap<String, Vec<(String, String, u16, String)>>;

/// Field weights: field_name -> weight
pub type FieldWeights = HashMap<String, f32>;

/// Source config: url and field weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub url: String,
    pub weights: FieldWeights,
}

/// Sources config: source_id -> SourceConfig
pub type SourcesConfig = HashMap<String, SourceConfig>;

/// Document ID tuple: (source_id, document_id, field_name)
pub type FieldPath = (String, String, String);

/// Document scores ordered tuple: (score, field_path)
pub type DocumentScore = (f32, FieldPath);

/// Cachable index data for IndexedDB storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachableIndex {
    pub index: InvertedIndex,
    pub doc_lengths: DocumentLengths,
    pub avg_doc_length: f32,
    pub num_docs: u16,
}
