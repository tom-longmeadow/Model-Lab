pub mod soa;
pub mod aos;
pub mod double_buffer;

/// Represents different ways data is stored in the simulation, like
/// in ram or gram, AOS or SOA. 
/// Implementors decide threading, buffering, and memory location.
pub trait Storage {
    type Item;

    fn new(capacity: usize) -> Self;

    /// how big is the storage
    fn len(&self) -> usize;

    /// what is the largest len before we resize
    fn capacity(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }

    /// Called before the solver starts a step — swap buffers, map memory etc.
    fn pre_step(&mut self) {}

    /// Called after the solver completes a step — upload, unmap, signal etc.
    fn post_step(&mut self) {}
}



 
 