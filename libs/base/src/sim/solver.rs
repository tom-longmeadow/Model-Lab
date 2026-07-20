pub mod particle; 
pub mod aos_vec_storage;
pub mod soa_vec_storage;

use crate::{sim::storage::Storage};

 
/// Loop contract only. No physics assumptions. No layout assumptions.
/// All solvers implement this regardless of storage layout or simulation type.
///
/// Fully monomorphized — `Simulation<St, Sv, Cr>` is a concrete type per configuration.
/// For side-by-side comparison, instantiate two concrete `Simulation` values.
pub trait Solver<S: Storage, Env> { 
    fn init(&mut self, _storage: &mut S, _environment: &mut Env); 
    fn pre_step(&mut self,  _storage: &mut S, _tick: u64, step_dt: f64, _environment: &mut Env);
    fn sub_step(&mut self,   storage: &mut S,  sub_step_dt: f64, _environment: &mut Env);
    fn post_step(&mut self, _storage: &mut S, _environment: &Env);
}
 
 