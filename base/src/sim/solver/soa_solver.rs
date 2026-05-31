use std::marker::PhantomData;
use crate::sim::{
    solver::{LeapfrogSolver, Solver, StepModel, StepModelSolver, 
        SymplecticEulerSolver, VelocityVerletSolver, VerletSolver,
        integrator::{Leapfrog, SymplecticEuler, VelocityVerlet, Verlet}
    }, 
    storage::{SoaNewtonianStorage, SoaStorage, SoaVerletStorage}
};


/// A [`StepModelSolver`] over [`SoaStorage`].
/// `sub_step` calls the slice integrator over whole columns.
/// Data is modified in place through column slices — compiler can auto-vectorize.
pub trait SoaStepSolver<S: SoaStorage>: StepModelSolver<S> {}

 
/// Concrete SoA solver.
/// - `M` — the [`StepModel`], use [`chain!`] to compose: `chain!(ClearAcc, Gravity, Drag)`
/// - `I` — the integrator: [`SymplecticEuler`], [`Verlet`], [`Leapfrog`]
/// - `N` — spatial dimension (used for marker trait bounds, implicit in column stride)
///
/// `sub_step` calls slice-form integrators directly on column slices — zero copying.
/// The compiler can auto-vectorize and emit SIMD instructions.
pub struct SoaSolver<M, I, const N: usize> {
    pub model:   M,
    _integrator: PhantomData<I>,
}

impl<M, I, const N: usize> SoaSolver<M, I, N> {
    pub fn new(model: M) -> Self { Self { model, _integrator: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for SoaSolver<M, SymplecticEuler, N>
where
    S: SoaStorage + SoaNewtonianStorage,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        self.model.pre(storage, dt);
        let (pos, vel, acc) = storage.pos_vel_mut_acc();
        SymplecticEuler::step_slice(pos, vel, acc, dt);
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> Solver<S> for SoaSolver<M, Verlet, N>
where
    S: SoaStorage + SoaVerletStorage,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        self.model.pre(storage, dt);
        let (pos, pos_old, acc) = storage.pos_pos_old_mut_acc();
        Verlet::step_slice(pos, pos_old, acc, dt);
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> Solver<S> for SoaSolver<M, Leapfrog, N>
where
    S: SoaStorage + SoaNewtonianStorage,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        let (vel, acc) = storage.vel_mut_acc();
        Leapfrog::half_kick_slice(vel, acc, dt);
        let (pos, vel) = storage.pos_mut_vel();
        Leapfrog::drift_slice(pos, vel, dt);
        self.model.pre(storage, dt);
        let (vel, acc) = storage.vel_mut_acc();
        Leapfrog::half_kick_slice(vel, acc, dt);
        self.model.post(storage, dt);
    }
}

impl<S, M, I, const N: usize> StepModelSolver<S> for SoaSolver<M, I, N>
where
    S: SoaStorage,
    M: StepModel<S>,
    SoaSolver<M, I, N>: Solver<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

impl<S, M, I, const N: usize> SoaStepSolver<S> for SoaSolver<M, I, N>
where
    S: SoaStorage,
    M: StepModel<S>,
    SoaSolver<M, I, N>: Solver<S> + StepModelSolver<S>,
{}

impl<S, M, const N: usize> SymplecticEulerSolver<S, N> for SoaSolver<M, SymplecticEuler, N>
where
    S: SoaStorage + SoaNewtonianStorage,
    M: StepModel<S>,
{}

impl<S, M, const N: usize> VerletSolver<S, N> for SoaSolver<M, Verlet, N>
where
    S: SoaStorage + SoaVerletStorage,
    M: StepModel<S>,
{}

impl<S, M, const N: usize> LeapfrogSolver<S, N> for SoaSolver<M, Leapfrog, N>
where
    S: SoaStorage + SoaNewtonianStorage,
    M: StepModel<S>,
{}

impl<S, M, const N: usize> Solver<S> for SoaSolver<M, VelocityVerlet, N>
where
    S: SoaStorage + SoaNewtonianStorage,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        // step 1 — pos update + first half vel-kick using current acc
        let (pos, vel, acc) = storage.pos_vel_mut_acc();
        VelocityVerlet::step1_slice(pos, vel, acc, dt);
        // recompute forces at new positions
        self.model.pre(storage, dt);
        // step 2 — second half vel-kick using new acc
        let (vel, acc) = storage.vel_mut_acc();
        VelocityVerlet::step2_slice(vel, acc, dt);
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> VelocityVerletSolver<S, N> for SoaSolver<M, VelocityVerlet, N>
where
    S: SoaStorage + SoaNewtonianStorage,
    M: StepModel<S>,
{}