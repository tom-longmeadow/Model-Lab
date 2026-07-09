use crate::sim::lifecycle::Deletor;

/// Removes the `n` oldest items (first in storage order). O(1) — no scoring needed.
pub struct Oldest;

impl Deletor for Oldest {
    fn select(&self, scores: &[f64], n: usize) -> Vec<usize> {
        let n = n.min(scores.len());
        (0..n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_first_n_indices() {
        let scores = vec![9.0, 8.0, 7.0, 6.0];
        assert_eq!(Oldest.select(&scores, 2), vec![0, 1]);
    }

    #[test]
    fn capped_at_len() {
        let scores = vec![1.0, 2.0];
        assert_eq!(Oldest.select(&scores, 100).len(), 2);
    }

    #[test]
    fn empty_scores() {
        assert_eq!(Oldest.select(&[], 3), vec![]);
    }
}
