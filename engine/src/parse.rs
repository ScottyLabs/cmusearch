/// Generate trigrams from a string (e.g., "hello" -> ["#he", "hel", "ell", "llo", "lo#"])
pub fn trigrams(s: &str) -> Vec<String> {
    let s = format!("#{}#", s);
    let chars: Vec<char> = s.chars().collect();
    let mut result = vec![];
    if chars.len() >= 3 {
        for i in 0..chars.len() - 2 {
            result.push(chars[i..i+3].iter().collect());
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
    
    // Generate trigrams for each word
    words.iter()
        .flat_map(|word| {
            word.split_whitespace()
                .flat_map(|w| trigrams(w))
        })
        .collect()
}

/// Tokenize text into words for indexing (lowercase, split on whitespace/punctuation)
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() > 1)
        .map(|s| s.to_string())
        .collect()
}

/// Generate trigrams from all words in a text for indexing
pub fn text_to_trigrams(text: &str) -> Vec<String> {
    let words = tokenize(text);
    words.iter()
        .flat_map(|word| trigrams(word))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigrams() {
        let result = trigrams("hello");
        assert_eq!(result, vec!["#he", "hel", "ell", "llo", "lo#"]);
    }

    #[test]
    fn test_parse_query() {
        let result = parse_query("rust programming");
        assert!(result.contains(&"#ru".to_string()));
        assert!(result.contains(&"rus".to_string()));
    }

    #[test]
    fn test_tokenize() {
        let result = tokenize("Hello, World!");
        assert_eq!(result, vec!["hello", "world"]);
    }
}
