use crate::sim::lifecycle::{selector::greatest_n, Deletor};

/// Removes the `n` items furthest from origin (highest distance score).
pub struct FurthestDeletor;

impl Deletor for FurthestDeletor {
    fn select(&self, scores: &[f64], n: usize) -> Vec<usize> {
        greatest_n(scores, n)
    }
}