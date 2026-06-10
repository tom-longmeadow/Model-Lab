use crate::sim::storage::{AosCpuStorage, CpuStorage, Storage};

/// Generic CPU-side Array-of-Structs storage.
/// `T` is the entity type — Particle, Node, GridCell, etc.
pub struct AosVecStorage<T> {
    data: Vec<T>,
}

impl<T> Storage for AosVecStorage<T> {
    fn len(&self) -> usize {
        self.data.len()
    }
    fn capacity(&self) -> usize {
        self.data.capacity()
    }
    fn clear(&mut self) {
        self.data.clear();
    }

    fn remove_indices(&mut self, mut indices: Vec<usize>) {
        indices.sort_unstable_by(|a, b| b.cmp(a));
        indices.dedup();
        for i in indices {
            self.data.swap_remove(i);
        }
    }
}

impl<T> CpuStorage for AosVecStorage<T> {
    fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }
}

impl<T> AosCpuStorage for AosVecStorage<T> {
    type Item = T;

    fn push(&mut self, item: T) {
        self.data.push(item);
    }

    fn swap_remove(&mut self, i: usize) -> T {
        self.data.swap_remove(i)
    }

    fn as_slice(&self) -> &[T] {
        &self.data
    }

    fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::storage::CpuStorage;

    // Used by the contract macros
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    pub struct MockEntity {
        pub d64: f64,
        pub c8:  u8,
    }

    // Used by the remove_indices tests
    #[derive(Default, Clone, Copy, Debug, PartialEq)]
    struct Mock { v: f64 }

    // --- contract macro tests ---
    crate::test_cpu_storage!(AosVecStorage<MockEntity>);
    crate::test_aos_cpu_storage!(
        AosVecStorage<MockEntity>,
        MockEntity,
        MockEntity { d64: 1.0, c8: 1 },
        MockEntity { d64: 2.0, c8: 2 }
    );

    // --- remove_indices tests ---
    #[test]
    fn remove_indices_removes_correct_items() {
        let mut s = AosVecStorage::new(8);
        for v in [1.0f64, 2.0, 3.0, 4.0, 5.0] {
            s.push(Mock { v });
        }
        s.remove_indices(vec![1, 3]);
        assert_eq!(s.len(), 3);
        let remaining: Vec<f64> = s.as_slice().iter().map(|p| p.v).collect();
        assert!(!remaining.contains(&2.0));
        assert!(!remaining.contains(&4.0));
    }

    #[test]
    fn remove_indices_handles_empty() {
        let mut s = AosVecStorage::new(4);
        s.push(Mock::default());
        s.remove_indices(vec![]);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn remove_indices_deduplicates() {
        let mut s = AosVecStorage::new(4);
        s.push(Mock::default());
        s.push(Mock::default());
        s.remove_indices(vec![0, 0, 0]);
        assert_eq!(s.len(), 1);
    }
}