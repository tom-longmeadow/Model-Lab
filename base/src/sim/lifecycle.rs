pub mod particle;
pub(crate) mod selector;
use crate::sim::storage::Storage;

// ---------------------------------------------------------------------------
// Simulation-level lifecycle trait (what Simulation<> holds)
// ---------------------------------------------------------------------------

/// Controls how and when new state enters and leaves the simulation.
/// This is what the `Simulation` struct holds generically.
pub trait Lifecycle<S: Storage> { 
    fn tick(&mut self, _storage: &mut S, _tick: u64);
}

// ---------------------------------------------------------------------------
// Creator — knows the GOAL (how many to add or remove)
// Zero knowledge of storage layout.
// ---------------------------------------------------------------------------
/// Knows the population goal. Exposes accessors that the `Deletor` and
/// `Manager` can call to understand what action is needed.
/// Has no knowledge of storage layout or selection strategy.
pub trait Creator {
    /// How many items should be added this tick.
    fn deficit(&self, current_len: usize) -> usize;

    /// How many items should be removed this tick.
    fn excess(&self, current_len: usize) -> usize;
}

// ---------------------------------------------------------------------------
// Deletor — knows WHICH items to remove (the selection strategy) 
// ---------------------------------------------------------------------------
/// Selects which indices to remove given a slice of scores.
/// Completely storage-agnostic — knows nothing about layout or entity type.
pub trait Deletor {
    fn select(&self, scores: &[f64], n: usize) -> Vec<usize>;
}