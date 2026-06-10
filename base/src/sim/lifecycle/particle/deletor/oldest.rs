use crate::sim::lifecycle::Deletor;

/// Removes the `n` oldest items (first in storage order).
/// No scoring needed — O(1).
pub struct OldestDeletor;

impl Deletor for OldestDeletor {
    fn select(&self, scores: &[f64], n: usize) -> Vec<usize> {
        let n = n.min(scores.len());
        (0..n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oldest_deletor_returns_first_n_indices() {
        // scores are irrelevant for OldestDeletor — it only uses n
        let scores = vec![9.0, 8.0, 7.0, 6.0];
        let d = OldestDeletor;
        assert_eq!(d.select(&scores, 2), vec![0, 1]);
    }

    #[test]
    fn oldest_deletor_capped_at_len() {
        let scores = vec![1.0, 2.0];
        let d = OldestDeletor;
        assert_eq!(d.select(&scores, 100).len(), 2);
    }
}