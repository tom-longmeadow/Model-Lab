 

use crate::{math::Bounds, sim::storage::Storage};

/// Controls how and when new state enters and leaves the simulation.
/// Implement this directly — compose [`Creator`] and [`Deletor`] as fields.
pub trait Lifecycle<S: Storage> {
    fn tick(&mut self, _storage: &mut S, _tick: u64, _bounds: &Bounds);
}
 