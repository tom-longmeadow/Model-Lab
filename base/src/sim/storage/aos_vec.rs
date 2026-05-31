use crate::sim::storage::{AosStorage, Storage};

/// Generic CPU-side Array-of-Structs storage.
/// `T` is the entity type — Particle, Node, GridCell, etc.
pub struct AosVecStorage<T> {
    data: Vec<T>,
}

impl<T> Storage for AosVecStorage<T> {
    type Item = T;

    fn new(capacity: usize) -> Self { Self { data: Vec::with_capacity(capacity) } }
    fn len(&self)      -> usize { self.data.len() }
    fn capacity(&self) -> usize { self.data.capacity() }
    fn push(&mut self, item: T)              { self.data.push(item); }
    fn swap_remove(&mut self, i: usize) -> T { self.data.swap_remove(i) }
    fn clear(&mut self) { self.data.clear(); }
}

impl<T> AosStorage for AosVecStorage<T> {
    fn as_slice(&self)         -> &[T]     { &self.data }
    fn as_slice_mut(&mut self) -> &mut [T] { &mut self.data }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Default)]
    pub struct MockEntity {
        pub d64: f64,
        pub c8:  u8,
    }

    crate::test_storage!(AosVecStorage<MockEntity>, MockEntity);
    crate::test_aos_storage!(
        AosVecStorage<MockEntity>, MockEntity,
        MockEntity { d64: 1.0, c8: 1 },
        MockEntity { d64: 2.0, c8: 2 }
    );
}
