

use crate::sim::storage::Storage;

/// Wraps any Storage in a double buffer.
/// Solver writes to `back`, renderer reads from `front`.
/// pre_step swaps them.
pub struct DoubleBuffer<S: Storage> {
    front: S,   // renderer reads this
    back:  S,   // solver writes this
}

impl<S: Storage> DoubleBuffer<S> {
    pub fn new(capacity: usize) -> Self {
        Self {
            front: S::new(capacity),
            back:  S::new(capacity),
        }
    }

    /// Solver writes here
    pub fn write(&mut self) -> &mut S { &mut self.back }

    /// Renderer reads here — never blocks solver
    pub fn read(&self) -> &S { &self.front }
}

impl<S: Storage> Storage for DoubleBuffer<S> {
    type Item = S::Item;

    fn new(capacity: usize) -> Self { Self::new(capacity) }
    fn len(&self)      -> usize { self.back.len() }
    fn capacity(&self) -> usize { self.back.capacity() }

    fn pre_step(&mut self) {
        // swap — front becomes available for next read snapshot
        std::mem::swap(&mut self.front, &mut self.back);
        self.back.pre_step();
    }

    fn post_step(&mut self) {
        self.back.post_step();
    }
}