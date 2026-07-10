pub mod acceleration;
pub mod integrator; 
pub mod newtonian;
pub mod verlet;

use std::marker::PhantomData;

use crate::sim::{storage::{AosCpuStorage, Storage}, Bounds};

 
/// Loop contract only. No physics assumptions. No layout assumptions.
/// All solvers implement this regardless of storage layout or simulation type.
///
/// Fully monomorphized — `Simulation<St, Sv, Cr>` is a concrete type per configuration.
/// For side-by-side comparison, instantiate two concrete `Simulation` values.
pub trait Solver<S: Storage> {
    fn init(&mut self,      _storage: &mut S) {}
    fn substep_count(&self) -> usize { 1 }
    fn pre_step(&mut self,  _storage: &mut S, _dt: f64, _tick: u64, _bounds: &Bounds) {}
    fn sub_step(&mut self,   storage: &mut S,  dt: f64);
    fn post_step(&mut self, _storage: &mut S, _dt: f64) {}
}
 

// /// Behavior applied before and after each integration step.
// ///
// /// Forces, constraints, advection, and any other per-step logic all implement this trait.
// /// Layout-agnostic — works with both [`AosStorage`] and [`SoaStorage`].
// /// Compose behaviors by writing a named struct (e.g. `BoxModel2d`) whose fields
// /// are the scalar kernels from `solver::step_model` and `solver::verlet::step_model`.
// ///
// /// | Hook | Runs | Typical uses |
// /// |------|------|--------------|
// /// | `pre`  | before integration | clear acc, accumulate forces, advection |
// /// | `post` | after  integration | clamp positions, resolve contacts, joint limits |
// pub trait StepModel<S: Storage> {
//     fn pre(&mut self,  _storage: &mut S, _dt: f64) {}
//     fn post(&mut self, _storage: &mut S, _dt: f64) {}
// }
 
// // /// A [`Solver`] that owns a [`StepModel`].
// // pub trait StepModelSolver<S: Storage>: Solver<S> {
// //     type Model: StepModel<S>;
// //     fn model(&mut self) -> &mut Self::Model;
// // }

// /// An AoS item that knows how to integrate itself.
// /// Replaces the need for AosVerletItem + AosNewtonianItem in the SOLVER.
// /// (Those accessor traits can stay for use in StepModels.)
// pub trait AosIntegrate {
//     fn integrate(&mut self, dt: f64);
// }

// /// A SoA storage that knows how to integrate itself.
// /// Replaces SoaVerletStorage + SoaNewtonianStorage as solver bounds.
// pub trait SoaIntegrate: Storage {
//     fn integrate(&mut self, dt: f64);
// }

// /// Split-step items expose half_kick and drift separately
// /// because the StepModel must run between them.
// pub trait AosLeapfrogIntegrate {
//     fn half_kick(&mut self, dt: f64);
//     fn drift(&mut self, dt: f64);
// }

// pub trait SoaLeapfrogIntegrate: Storage {
//     fn half_kick(&mut self, dt: f64);
//     fn drift(&mut self, dt: f64);
// }

// pub struct AosSolver<S, M> {
//     pub model: M,
//     _p: PhantomData<S>,
// }

// impl<S, M> AosSolver<S, M> {
//     pub fn new(model: M) -> Self {
//         Self { model, _p: PhantomData }
//     }
// }

// impl<S, M> Solver<S> for AosSolver<S, M>
// where
//     S: AosCpuStorage,
//     S::Item: AosIntegrate,
//     M: StepModel<S>,
// {
//     fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64, _bounds: &Bounds) {
//         self.model.pre(storage, dt);
//     }
//     fn sub_step(&mut self, storage: &mut S, dt: f64) {
//         // One line. Covers ALL physics. Forever.
//         for item in storage.iter_mut() {
//             item.integrate(dt);
//         }
//     }
//     fn post_step(&mut self, storage: &mut S, dt: f64) {
//         self.model.post(storage, dt);
//     }
// }

// pub struct SoaSolver<S, M> {
//     pub model: M,
//     _p: PhantomData<S>,
// }

// impl<S, M> Solver<S> for SoaSolver<S, M>
// where
//     S: SoaIntegrate,
//     M: StepModel<S>,
// {
//     fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64, _bounds: &Bounds) {
//         self.model.pre(storage, dt);
//     }
//     fn sub_step(&mut self, storage: &mut S, dt: f64) {
//         storage.integrate(dt);
//     }
//     fn post_step(&mut self, storage: &mut S, dt: f64) {
//         self.model.post(storage, dt);
//     }
// }


// pub struct LeapfrogAosSolver<S, M> {
//     pub model: M,
//     _p: PhantomData<S>,
// }

// impl<S, M> LeapfrogAosSolver<S, M> {
//     pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
// }

// impl<S, M> Solver<S> for LeapfrogAosSolver<S, M>
// where
//     S: AosCpuStorage,
//     S::Item: AosLeapfrogIntegrate,
//     M: StepModel<S>,
// {
//     fn init(&mut self, storage: &mut S) {
//         self.model.pre(storage, 0.0);
//     }
//     fn sub_step(&mut self, storage: &mut S, dt: f64) {
//         let half = dt * 0.5;
//         for item in storage.iter_mut() { item.half_kick(half); }
//         for item in storage.iter_mut() { item.drift(dt); }
//         self.model.pre(storage, dt);   // recompute forces at new positions
//         for item in storage.iter_mut() { item.half_kick(half); }
//     }
//     fn post_step(&mut self, storage: &mut S, dt: f64) {
//         self.model.post(storage, dt);
//     }
// }

// pub struct LeapfrogSoaSolver<S, M> {
//     pub model: M,
//     _p: PhantomData<S>,
// }

// impl<S, M> Solver<S> for LeapfrogSoaSolver<S, M>
// where
//     S: SoaLeapfrogIntegrate,
//     M: StepModel<S>,
// {
//     fn init(&mut self, storage: &mut S) {
//         self.model.pre(storage, 0.0);
//     }
//     fn sub_step(&mut self, storage: &mut S, dt: f64) {
//         let half = dt * 0.5;
//         storage.half_kick(half);
//         storage.drift(dt);
//         self.model.pre(storage, dt);
//         storage.half_kick(half);
//     }
//     fn post_step(&mut self, storage: &mut S, dt: f64) {
//         self.model.post(storage, dt);
//     }
// }