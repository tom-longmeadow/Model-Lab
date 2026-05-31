use std::marker::PhantomData;
use crate::sim::{
    solver::{LeapfrogSolver, Solver, StepModel, StepModelSolver, SymplecticEulerSolver, 
        VelocityVerletSolver, VerletSolver, 
        integrator::{Leapfrog, SymplecticEuler, VelocityVerlet, Verlet}, 
        particle::{NewtonianParticle, VerletParticle}
    }, 
    storage::AosStorage
};

/// A [`StepModelSolver`] over [`AosStorage`].
/// `sub_step` calls the scalar integrator once per item via `iter_mut()`.
/// Data is modified in place through references — zero copying.
pub trait AosStepSolver<S: AosStorage>: StepModelSolver<S> {}

 
/// Concrete AoS solver.
/// - `M` — the [`StepModel`], use [`chain!`] to compose: `chain!(ClearAcc, Gravity, Drag)`
/// - `I` — the integrator: [`SymplecticEuler`], [`Verlet`], [`Leapfrog`]
/// - `N` — spatial dimension
///
/// `sub_step` modifies particle data in place through references — zero copying.
/// One struct satisfies [`SymplecticEulerSolver`], [`VerletSolver`], or [`LeapfrogSolver`]
/// automatically based on `I` and the item type of `S`.
pub struct AosSolver<M, I, const N: usize> {
    pub model:   M,
    _integrator: PhantomData<I>,
}

impl<M, I, const N: usize> AosSolver<M, I, N> {
    pub fn new(model: M) -> Self { Self { model, _integrator: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for AosSolver<M, SymplecticEuler, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: NewtonianParticle<N>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        self.model.pre(storage, dt);
        for item in storage.iter_mut() {
            let (pos, vel, acc) = item.pos_vel_mut_acc();
            SymplecticEuler::step_scalar(pos, vel, acc, dt);
        }
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> Solver<S> for AosSolver<M, Verlet, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: VerletParticle<N>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        self.model.pre(storage, dt);
        for item in storage.iter_mut() {
            let (pos, pos_old, acc) = item.pos_pos_old_mut_acc();
            Verlet::step_scalar(pos, pos_old, acc, dt);
        }
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> Solver<S> for AosSolver<M, Leapfrog, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: NewtonianParticle<N>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        // half-kick + drift — one pass, in-place
        for item in storage.iter_mut() {
            let (vel, acc) = item.vel_mut_acc();
            Leapfrog::half_kick_scalar(vel, acc, dt);
            let (pos, vel) = item.pos_mut_vel();
            Leapfrog::drift_scalar(pos, vel, dt);
        }
        // recompute forces at new positions
        self.model.pre(storage, dt);
        // second half-kick — one pass, in-place
        for item in storage.iter_mut() {
            let (vel, acc) = item.vel_mut_acc();
            Leapfrog::half_kick_scalar(vel, acc, dt);
        }
        self.model.post(storage, dt);
    }
}

impl<S, M, I, const N: usize> StepModelSolver<S> for AosSolver<M, I, N>
where
    S: AosStorage,
    M: StepModel<S>,
    AosSolver<M, I, N>: Solver<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

impl<S, M, I, const N: usize> AosStepSolver<S> for AosSolver<M, I, N>
where
    S: AosStorage,
    M: StepModel<S>,
    AosSolver<M, I, N>: Solver<S> + StepModelSolver<S>,
{}

impl<S, M, const N: usize> SymplecticEulerSolver<S, N> for AosSolver<M, SymplecticEuler, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: NewtonianParticle<N>,
{}

impl<S, M, const N: usize> VerletSolver<S, N> for AosSolver<M, Verlet, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: VerletParticle<N>,
{}

impl<S, M, const N: usize> LeapfrogSolver<S, N> for AosSolver<M, Leapfrog, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: NewtonianParticle<N>,
{}

impl<S, M, const N: usize> Solver<S> for AosSolver<M, VelocityVerlet, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: NewtonianParticle<N>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        // step 1 — pos update + first half vel-kick using current acc
        for item in storage.iter_mut() {
            let (pos, vel, acc) = item.pos_vel_mut_acc();
            VelocityVerlet::step1_scalar(pos, vel, acc, dt);
        }
        // recompute forces at new positions
        self.model.pre(storage, dt);
        // step 2 — second half vel-kick using new acc
        for item in storage.iter_mut() {
            let (vel, acc) = item.vel_mut_acc();
            VelocityVerlet::step2_scalar(vel, acc, dt);
        }
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> VelocityVerletSolver<S, N> for AosSolver<M, VelocityVerlet, N>
where
    S: AosStorage,
    M: StepModel<S>,
    S::Item: NewtonianParticle<N>,
{}