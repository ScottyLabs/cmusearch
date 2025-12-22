use std::collections::HashMap;

use crate::{
    parse::doc_to_ngrams_map,
    types::{DocumentLengths, InvertedIndex, SourcesStore},
};

/// Build inverted index from documents
/// Maps each 4-gram to list of (source_id, doc_id, term_frequency, field_name) pairs
pub fn build_sources_index(sources: &SourcesStore) -> (InvertedIndex, DocumentLengths) {
    let mut index: InvertedIndex = HashMap::new();
    let mut doc_lengths: DocumentLengths = HashMap::new();

    for (source_id, docs) in sources {
        for (doc_id, doc_fields) in docs {
            // Generate 4-grams for each field and count frequencies
            let doc_ngrams = doc_to_ngrams_map(doc_fields);

            for (field_name, field_ngrams) in &doc_ngrams {
                // Count term frequencies for this field
                let mut term_freq: HashMap<String, u16> = HashMap::new();
                for ngram in field_ngrams {
                    *term_freq.entry(ngram.clone()).or_insert(0) += 1;
                }

                // Record document length for this field
                doc_lengths
                    .entry(doc_id.clone())
                    .or_insert_with(HashMap::new)
                    .insert(field_name.clone(), field_ngrams.len() as u16);

                // Skip empty fields (no ngrams generated)
                if field_ngrams.is_empty() {
                    continue;
                }

                // Index each unique ngram in this field
                for (ngram, &freq) in &term_freq {
                    index.entry(ngram.clone()).or_insert_with(Vec::new).push((
                        source_id.clone(),
                        doc_id.clone(),
                        freq,
                        field_name.clone(),
                    ));
                }
            }
        }
    }

    (index, doc_lengths)
}
