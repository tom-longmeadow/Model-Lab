pub mod creator;
pub mod deletor;

use crate::sim::storage::Storage;

/// Controls how and when new state enters and leaves the simulation.
/// Implement this directly — compose [`Creator`] and [`Deletor`] as fields.
pub trait Lifecycle<S: Storage> {
    fn tick(&mut self, _storage: &mut S, _tick: u64);
}

/// Knows the population goal: how many items to add or remove this tick.
/// No knowledge of storage layout or selection strategy.
pub trait Creator {
    /// How many items should be added this tick.
    fn deficit(&self, current_len: usize) -> usize;
    /// How many items should be removed this tick.
    fn excess(&self, current_len: usize) -> usize;
}

/// Selects which indices to remove given a slice of scores.
/// Storage-agnostic — only sees scores, not entities.
pub trait Deletor {
    fn select(&self, scores: &[f64], n: usize) -> Vec<usize>;
}