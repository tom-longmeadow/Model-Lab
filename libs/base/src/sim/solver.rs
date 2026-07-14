pub mod partition;
pub mod verlet;
pub mod tuning;
pub mod constraint;
pub mod solver_2d;


use crate::{math::Bounds, sim::storage::Storage};

 
/// Loop contract only. No physics assumptions. No layout assumptions.
/// All solvers implement this regardless of storage layout or simulation type.
///
/// Fully monomorphized — `Simulation<St, Sv, Cr>` is a concrete type per configuration.
/// For side-by-side comparison, instantiate two concrete `Simulation` values.
pub trait Solver<S: Storage> {
    fn init(&mut self,      _storage: &mut S) {}
    fn substep_count(&self) -> u64 { 1 }
    fn pre_step(&mut self,  _storage: &mut S, _dt: f64, _tick: u64, _bounds: &Bounds) {}
    fn sub_step(&mut self,   storage: &mut S,  dt: f64);
    fn post_step(&mut self, _storage: &mut S, _dt: f64) {}
}
 
 