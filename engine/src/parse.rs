/// Common 4-grams to filter out (provide little search value, slow queries)
const STOP_NGRAMS: &[&str] = &[
    // Very common English patterns (4-char)
    "#the", "the#", "tion", "atio", "ment", "ents", "ting", "ding", "with", "ith#", "#wit", "have",
    "from", "that", "this", "will", "been", "were", "them", "they", "what", "your", "when", "more",
    "some", "than", "also", "into", "only", "come", "made", "just", "over", "such", "make", "like",
    "time", "very", "each", "much", "most", "back", "part", "work", "year", "take", "even", "good",
    "#and", "and#", "#for", "for#", "#are", "are#", "#not", "not#", "stud", "tude", "uden", "dent",
    "#stu", "ent#", "cour", "ours", "urse", "#cou", "rse#", "prog", "rogr", "ogra", "gram", "#pro",
    "ram#", "intr", "ntro", "trod", "rodu", "oduc", "duct", "#int", "tion",
];

/// Generate 4-grams from a string (e.g., "hello" -> ["#hel", "hell", "ello", "llo#"])
/// Filters out common stop-ngrams that provide little search value
pub fn ngrams(s: &String) -> Vec<String> {
    let s = format!("###{}###", s);
    let chars: Vec<char> = s.chars().collect();
    let mut result = vec![];
    if chars.len() >= 4 {
        for i in 0..chars.len() - 3 {
            let ngram: String = chars[i..i + 4].iter().collect();
            // Filter out stop ngrams
            if !STOP_NGRAMS.contains(&ngram.as_str()) {
                result.push(ngram);
            }
        }
    }
    result
}

/// Parse query into trigrams for matching
/// Splits on non-alphanumeric, lowercases, and generates trigrams
pub fn parse_query(query: &str) -> Vec<String> {
    let query = query.trim().to_lowercase();

    // Split on non-alphanumeric characters
    let words: Vec<&str> = query
        .split(|c: char| !c.is_alphanumeric() && c != ' ')
        .filter(|s| !s.is_empty())
        .collect();

    // Generate 4-grams for each word
    words
        .iter()
        .flat_map(|word| {
            word.split_whitespace()
                .flat_map(|w| ngrams(&String::from(w)))
        })
        .collect()
}

/// Tokenize text into words for indexing (lowercase, split on whitespace/punctuation)
pub fn tokenize(text: &String) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() > 1)
        .map(|s| s.to_string())
        .collect()
}

/// Generate 4-grams from all words in a text for indexing (Vec version)
pub fn doc_to_ngrams(text: &Vec<String>) -> Vec<Vec<String>> {
    text.iter()
        .map(|field| {
            // First break apart whitespace
            let tokens = tokenize(field);
            // Then generate 4-grams with standardized whitespace
            ngrams(&tokens.join(" "))
        })
        .collect()
}

use crate::types::Document;
use std::collections::HashMap;

/// Generate 4-grams from all fields in a document (HashMap version)
pub fn doc_to_ngrams_map(doc: &Document) -> HashMap<String, Vec<String>> {
    doc.iter()
        .map(|(field_name, field_value)| {
            let tokens = tokenize(field_value);
            let field_ngrams = ngrams(&tokens.join(" "));
            (field_name.clone(), field_ngrams)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ngrams() {
        let result = ngrams(&String::from("hello"));
        assert_eq!(result, vec!["#hel", "hell", "ello", "llo#"]);
    }

    #[test]
    fn test_parse_query() {
        let result = parse_query("rust programming");
        assert!(result.contains(&"#rus".to_string()));
        assert!(result.contains(&"rus".to_string()));
    }

    #[test]
    fn test_tokenize() {
        let result = tokenize(&String::from("Hello, World!"));
        assert_eq!(result, vec!["hello", "world"]);
    }
}
