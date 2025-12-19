/// BM-25 parameters
const K1: f32 = 1.2;
const B: f32 = 0.2;

/// Calculate BM-25 score for a single term
/// 
/// Arguments:
/// - term_freq: frequency of term in document
/// - doc_len: number of terms in document  
/// - doc_freq: number of documents containing this term
/// - num_docs: total number of documents
/// - avg_dl: average document length
pub fn bm25_term(term_freq: u16, doc_len: u16, doc_freq: u16, num_docs: u16, avg_dl: f32) -> f32 {
    let dl = doc_len as f32;
    let tf = term_freq as f32;
    let df = doc_freq as f32;
    let n = num_docs as f32;
    
    // IDF component: log((N - df + 0.5) / (df + 0.5) + 1)
    let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
    
    // TF component with length normalization
    let tf_norm = (tf * (K1 + 1.0)) / (tf + K1 * (1.0 - B + B * dl / avg_dl));
    
    idf * tf_norm
}

/// Get top N results from a ranked list
pub fn top_n(rank_list: &[(String, f32)], n: usize) -> Vec<(String, f32)> {
    if rank_list.len() <= n {
        return rank_list.to_vec();
    }
    
    let mut sorted: Vec<_> = rank_list.to_vec();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted.truncate(n);
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_term() {
        // Basic sanity check - higher term freq should give higher score
        let score1 = bm25_term(1, 100, 10, 1000, 100.0);
        let score2 = bm25_term(5, 100, 10, 1000, 100.0);
        assert!(score2 > score1);
    }

    #[test]
    fn test_top_n() {
        let scores = vec![
            ("a".to_string(), 1.0),
            ("b".to_string(), 3.0),
            ("c".to_string(), 2.0),
        ];
        let top = top_n(&scores, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "b");
        assert_eq!(top[1].0, "c");
    }
}
