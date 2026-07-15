pub mod particle; 
pub mod aos_vec_storage;

use crate::{sim::storage::Storage};

 
/// Loop contract only. No physics assumptions. No layout assumptions.
/// All solvers implement this regardless of storage layout or simulation type.
///
/// Fully monomorphized — `Simulation<St, Sv, Cr>` is a concrete type per configuration.
/// For side-by-side comparison, instantiate two concrete `Simulation` values.
pub trait Solver<S: Storage> {
    type Bounds;

    fn init(&mut self,      _storage: &mut S) {}
    fn substep_count(&self) -> u64 { 1 }
    fn pre_step(&mut self,  _storage: &mut S, _dt: f64, _tick: u64, _bounds: &Self::Bounds) {}
    fn sub_step(&mut self,   storage: &mut S,  dt: f64);
    fn post_step(&mut self, _storage: &mut S, _dt: f64) {}
}
 
 