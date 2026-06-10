
use crate::sim::lifecycle::{selector::least_n, Deletor};

/// Removes the `n` items with the lowest score (e.g. slowest speed).
pub struct SlowestDeletor;

impl Deletor for SlowestDeletor {
    fn select(&self, scores: &[f64], n: usize) -> Vec<usize> {
        least_n(scores, n)
    }
}