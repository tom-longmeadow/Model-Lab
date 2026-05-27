use crate::sim::storage::Storage;

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