pub mod particle;
use crate::sim::storage::Storage;

// ---------------------------------------------------------------------------
// Base trait
// ---------------------------------------------------------------------------

/// Controls how and when new state enters the simulation.
/// Makes no assumptions about what is created or how storage is modified.
/// The implementor has full access to storage each tick and decides what to do.
pub trait Creator<S: Storage> {

    /// Called once before the first tick.
    /// Use to populate initial conditions into storage.
    fn init(&mut self, _storage: &mut S) {}

    /// Called once per tick before the solver runs.
    /// Use to add entities, set boundary conditions, or modify field values.
    fn tick(&mut self, _storage: &mut S, _tick: u64) {}
}

// ---------------------------------------------------------------------------
// Index selection — agnostic to storage type and simulation domain.
// Take `&[f64]` scores, return `Vec<usize>` indices.
// ---------------------------------------------------------------------------

/// Indices of the `n` entries with the **lowest** score.
/// Returns up to `scores.len()` indices if `n` exceeds it.
/// Order of returned indices is unspecified — sort before passing to `bulk_remove`.
pub fn slowest_n(scores: &[f64], n: usize) -> Vec<usize> {
    select_n_by(scores, n, |a, b| a.partial_cmp(b).unwrap())
}

/// Indices of the `n` entries with the **highest** score.
pub fn fastest_n(scores: &[f64], n: usize) -> Vec<usize> {
    select_n_by(scores, n, |a, b| b.partial_cmp(a).unwrap())
}

/// Indices of the **first** `n` entries in storage order (oldest by insertion).
pub fn oldest_n(_scores: &[f64], n: usize, total: usize) -> Vec<usize> {
    (0..n.min(total)).collect()
}

/// Internal: partial-sort indices by score comparator, return first `n`.
fn select_n_by<F>(scores: &[f64], n: usize, cmp: F) -> Vec<usize>
where
    F: Fn(&f64, &f64) -> std::cmp::Ordering,
{
    let n = n.min(scores.len());
    let mut indices: Vec<usize> = (0..scores.len()).collect();
    indices.select_nth_unstable_by(n.saturating_sub(1), |&a, &b| cmp(&scores[a], &scores[b]));
    indices.truncate(n);
    indices
}

// ---------------------------------------------------------------------------
// Bulk removal — agnostic to storage layout and simulation domain.
// ---------------------------------------------------------------------------

/// Remove all items at `indices` in a single O(K log K + K) pass.
/// Sorts descending so each `swap_remove` doesn't invalidate subsequent indices.
/// Works identically for AoS and SoA — both implement `Storage::swap_remove`.
pub fn bulk_remove<S: Storage>(storage: &mut S, mut indices: Vec<usize>) {
    indices.sort_unstable_by(|a, b| b.cmp(a));
    indices.dedup();
    for i in indices {
        storage.swap_remove(i);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::storage::{AosStorage, aos_vec::AosVecStorage};

    #[derive(Default, Clone, Copy)]
    struct Mock { v: f64 }
    type MockStorage = AosVecStorage<Mock>;

    // --- select ---

    #[test]
    fn slowest_n_returns_lowest_score_indices() {
        let scores = vec![5.0, 1.0, 3.0, 0.5, 4.0];
        let mut result = slowest_n(&scores, 2);
        result.sort_unstable();
        assert_eq!(result, vec![1, 3]); // 0.5 and 1.0
    }

    #[test]
    fn fastest_n_returns_highest_score_indices() {
        let scores = vec![5.0, 1.0, 3.0, 0.5, 4.0];
        let mut result = fastest_n(&scores, 2);
        result.sort_unstable();
        assert_eq!(result, vec![0, 4]); // 5.0 and 4.0
    }

    #[test]
    fn oldest_n_returns_first_indices() {
        let scores = vec![9.0, 8.0, 7.0, 6.0];
        assert_eq!(oldest_n(&scores, 2, scores.len()), vec![0, 1]);
    }

    #[test]
    fn select_n_capped_at_len() {
        let scores = vec![1.0, 2.0];
        assert_eq!(slowest_n(&scores, 100).len(), 2);
    }

    // --- bulk_remove ---

    #[test]
    fn bulk_remove_removes_correct_count() {
        let mut s = MockStorage::new(8);
        for v in [1.0f64, 2.0, 3.0, 4.0, 5.0] { s.push(Mock { v }); }
        bulk_remove(&mut s, vec![1, 3]);
        assert_eq!(s.len(), 3);
        let remaining: Vec<f64> = s.iter().map(|p| p.v).collect();
        assert!(!remaining.contains(&2.0));
        assert!(!remaining.contains(&4.0));
    }

    #[test]
    fn bulk_remove_handles_empty_indices() {
        let mut s = MockStorage::new(4);
        s.push(Mock::default());
        bulk_remove(&mut s, vec![]);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn bulk_remove_deduplicates_indices() {
        let mut s = MockStorage::new(4);
        s.push(Mock::default());
        s.push(Mock::default());
        bulk_remove(&mut s, vec![0, 0, 0]);
        assert_eq!(s.len(), 1);
    }
}