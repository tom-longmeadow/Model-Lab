use crate::sim::lifecycle::Deletor;

/// Removes the `n` items furthest from origin (highest distance score).
pub struct Furthest;

impl Deletor for Furthest {
    fn select(&self, scores: &[f64], n: usize) -> Vec<usize> {
        let n = n.min(scores.len());
        if n == 0 { return vec![]; }
        let mut indices: Vec<usize> = (0..scores.len()).collect();
        indices.select_nth_unstable_by(n.saturating_sub(1), |&a, &b| {
            scores[b].partial_cmp(&scores[a]).unwrap()
        });
        indices.truncate(n);
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_highest_score_indices() {
        let scores = vec![5.0, 1.0, 3.0, 0.5, 4.0];
        let mut result = Furthest.select(&scores, 2);
        result.sort_unstable();
        assert_eq!(result, vec![0, 4]); // 5.0 and 4.0
    }

    #[test]
    fn capped_at_len() {
        assert_eq!(Furthest.select(&[1.0, 2.0], 100).len(), 2);
    }

    #[test]
    fn zero_returns_empty() {
        assert_eq!(Furthest.select(&[1.0, 2.0], 0), vec![]);
    }
}
