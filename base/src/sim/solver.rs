pub mod step_model;
pub mod particle;
pub mod integrator;
pub mod aos_solver;
pub mod soa_solver;

use std::marker::PhantomData;
use crate::sim::storage::{Storage, AosStorage, SoaStorage, SoaNewtonianStorage, SoaVerletStorage};

 
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
 

/// Universal behavior slot — runs against the whole storage before and after integration.
/// Forces, constraints, advection, behaviour — all implement this.
/// Layout-agnostic: works with [`AosStorage`] and [`SoaStorage`].
/// - `pre()`  — before integration: forces, advection, behaviour
/// - `post()` — after  integration: constraints, pressure, leapfrog second half-kick
pub trait StepModel<S: Storage> {
    fn pre(&mut self,  _storage: &mut S, _dt: f64) {}
    fn post(&mut self, _storage: &mut S, _dt: f64) {}
}
 
/// A [`Solver`] that owns a [`StepModel`].
pub trait StepModelSolver<S: Storage>: Solver<S> {
    type Model: StepModel<S>;
    fn model(&mut self) -> &mut Self::Model;
}


/// A solver using [`SymplecticEuler`] integration.
/// Items must provide [`NewtonianParticle<N>`].
pub trait SymplecticEulerSolver<S: Storage, const N: usize>: StepModelSolver<S> {}

/// A solver using [`Verlet`] integration.
/// Items must provide [`VerletParticle<N>`].
pub trait VerletSolver<S: Storage, const N: usize>: StepModelSolver<S> {}

/// A solver using [`Leapfrog`] integration.
/// Items must provide [`NewtonianParticle<N>`].
/// Symplectic — conserves energy better than [`SymplecticEuler`] over long runs.
pub trait LeapfrogSolver<S: Storage, const N: usize>: StepModelSolver<S> {}

/// A solver using [`VelocityVerlet`] integration.
/// Items must provide [`NewtonianParticle<N>`].
/// 2nd-order symplectic — conserves energy and directly yields velocity at each step.
pub trait VelocityVerletSolver<S: Storage, const N: usize>: StepModelSolver<S> {}


