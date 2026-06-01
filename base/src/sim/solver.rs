pub mod newtonian;
pub mod verlet;
pub mod step_model;
pub mod integrator;

 
use crate::sim::storage::{Storage};

 
/// Loop contract only. No physics assumptions. No layout assumptions.
/// All solvers implement this regardless of storage layout or simulation type.
///
/// Fully monomorphized — `Simulation<St, Sv, Cr>` is a concrete type per configuration.
/// For side-by-side comparison, instantiate two concrete `Simulation` values.
pub trait Solver<S: Storage> {
    fn init(&mut self,      _storage: &mut S) {}
    fn substep_count(&self) -> usize { 1 }
    fn pre_step(&mut self,  _storage: &mut S, _dt: f64, _tick: u64) {}
    fn sub_step(&mut self,   storage: &mut S,  dt: f64);
    fn post_step(&mut self, _storage: &mut S, _dt: f64) {}
}
 

/// Behavior applied before and after each integration step.
///
/// Forces, constraints, advection, and any other per-step logic all implement this trait.
/// Layout-agnostic — works with both [`AosStorage`] and [`SoaStorage`].
/// Compose behaviors by writing a named struct (e.g. `BoxModel2d`) whose fields
/// are the scalar kernels from `solver::step_model` and `solver::verlet::step_model`.
///
/// | Hook | Runs | Typical uses |
/// |------|------|--------------|
/// | `pre`  | before integration | clear acc, accumulate forces, advection |
/// | `post` | after  integration | clamp positions, resolve contacts, joint limits |
pub trait StepModel<S: Storage> {
    fn pre(&mut self,  _storage: &mut S, _dt: f64) {}
    fn post(&mut self, _storage: &mut S, _dt: f64) {}
}
 
/// A [`Solver`] that owns a [`StepModel`].
pub trait StepModelSolver<S: Storage>: Solver<S> {
    type Model: StepModel<S>;
    fn model(&mut self) -> &mut Self::Model;
}



