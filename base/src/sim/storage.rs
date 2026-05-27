 
/// Represents any simulation storage.
/// Makes no assumptions about memory layout, threading,
/// buffering, or the type of simulation.
pub trait Storage {
    /// The unit of data this storage holds.
    /// Could be a particle, a grid cell, a node, or a scalar field value.
    type Item;

    /// Creates storage with an initial capacity hint.
    fn new(capacity: usize) -> Self;

    /// Number of items currently stored.
    fn len(&self) -> usize;

    /// Maximum items before reallocation or overflow.
    fn capacity(&self) -> usize;

    fn is_empty(&self) -> bool { self.len() == 0 }

    /// Called before the solver starts a step.
    /// Use for buffer swaps, memory mapping, fence waits.
    fn pre_step(&mut self)  {}

    /// Called after the solver completes a step.
    /// Use for GPU uploads, unmapping, signalling.
    fn post_step(&mut self) {}
}


 
 