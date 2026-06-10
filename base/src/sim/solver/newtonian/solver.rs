use std::marker::PhantomData;

use crate::sim::storage::{
    AosCpuStorage,
    newtonian::soa::SoaNewtonianStorage,
    newtonian::aos::AosNewtonianItem,
};
use crate::sim::solver::{
    Solver, StepModel, StepModelSolver,
};
use crate::sim::solver::integrator::{
    SymplecticEuler, Leapfrog, VelocityVerlet,
};

// ---------------------------------------------------------------------------
// SymplecticEuler
// ---------------------------------------------------------------------------

/// SoA solver — `vel += acc·dt`, `pos += vel·dt`.
/// Forces are accumulated once per step (model.pre in pre_step), then the
/// full column is integrated in stride-1 inner loops LLVM can auto-vectorise.
pub struct SoaSymplecticEulerSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> SoaSymplecticEulerSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for SoaSymplecticEulerSolver<S, M, N>
where
    S: SoaNewtonianStorage,
    M: StepModel<S>,
{
    fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
        self.model.pre(storage, dt);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        for c in 0..N {
            let (pos, vel, acc) = storage.pos_vel_col_mut_acc(c);
            for i in 0..pos.len() {
                SymplecticEuler::step(&mut pos[i], &mut vel[i], acc[i], dt);
            }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for SoaSymplecticEulerSolver<S, M, N>
where
    S: SoaNewtonianStorage,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

/// AoS solver — `vel += acc·dt`, `pos += vel·dt` per item.
/// Inner loop over N components is const-generic, unrolled by the compiler.
pub struct AosSymplecticEulerSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> AosSymplecticEulerSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for AosSymplecticEulerSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosNewtonianItem<N>,
    M: StepModel<S>,
{
    fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
        self.model.pre(storage, dt);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        for item in storage.iter_mut() {
            let (pos, vel, acc) = item.pos_vel_mut_acc();
            for i in 0..N {
                SymplecticEuler::step(&mut pos[i], &mut vel[i], acc[i], dt);
            }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for AosSymplecticEulerSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosNewtonianItem<N>,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

// ---------------------------------------------------------------------------
// Leapfrog
// ---------------------------------------------------------------------------
// Sequence per sub_step:
//   1. half_kick  (vel += ½·acc·dt)   — current acc
//   2. drift      (pos += vel·dt)
//   3. model.pre  — recompute acc at new positions
//   4. half_kick  (vel += ½·acc·dt)   — new acc
// Forces must be initialised before the first step; call solver.init().

/// SoA leapfrog solver.
pub struct SoaLeapfrogSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> SoaLeapfrogSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for SoaLeapfrogSolver<S, M, N>
where
    S: SoaNewtonianStorage,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) {
        self.model.pre(storage, 0.0);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        // 1. half_kick with current acc
        for c in 0..N {
            let (vel, acc) = storage.vel_col_mut_acc(c);
            for i in 0..vel.len() { Leapfrog::half_kick(&mut vel[i], acc[i], dt); }
        }
        // 2. drift
        for c in 0..N {
            let (pos, vel) = storage.pos_col_mut_vel(c);
            for i in 0..pos.len() { Leapfrog::drift(&mut pos[i], vel[i], dt); }
        }
        // 3. recompute forces at new positions
        self.model.pre(storage, dt);
        // 4. half_kick with new acc
        for c in 0..N {
            let (vel, acc) = storage.vel_col_mut_acc(c);
            for i in 0..vel.len() { Leapfrog::half_kick(&mut vel[i], acc[i], dt); }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for SoaLeapfrogSolver<S, M, N>
where
    S: SoaNewtonianStorage,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

/// AoS leapfrog solver.
pub struct AosLeapfrogSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> AosLeapfrogSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for AosLeapfrogSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosNewtonianItem<N>,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) {
        self.model.pre(storage, 0.0);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        // 1 & 2: half_kick then drift
        for item in storage.iter_mut() {
            let (vel, acc) = item.vel_mut_acc();
            for i in 0..N { Leapfrog::half_kick(&mut vel[i], acc[i], dt); }
            let (pos, vel) = item.pos_mut_vel();
            for i in 0..N { Leapfrog::drift(&mut pos[i], vel[i], dt); }
        }
        // 3. recompute forces
        self.model.pre(storage, dt);
        // 4. half_kick with new acc
        for item in storage.iter_mut() {
            let (vel, acc) = item.vel_mut_acc();
            for i in 0..N { Leapfrog::half_kick(&mut vel[i], acc[i], dt); }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for AosLeapfrogSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosNewtonianItem<N>,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

// ---------------------------------------------------------------------------
// VelocityVerlet
// ---------------------------------------------------------------------------
// Sequence per sub_step:
//   1. step1  — pos += vel·dt + ½·acc·dt²,  vel += ½·acc·dt
//   2. model.pre  — recompute acc at new positions
//   3. step2  — vel += ½·acc_new·dt
// Forces must be initialised before the first step; call solver.init().

/// SoA velocity-verlet solver.
pub struct SoaVelocityVerletSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> SoaVelocityVerletSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for SoaVelocityVerletSolver<S, M, N>
where
    S: SoaNewtonianStorage,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) {
        self.model.pre(storage, 0.0);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        // 1. step1
        for c in 0..N {
            let (pos, vel, acc) = storage.pos_vel_col_mut_acc(c);
            for i in 0..pos.len() {
                VelocityVerlet::step1(&mut pos[i], &mut vel[i], acc[i], dt);
            }
        }
        // 2. recompute forces
        self.model.pre(storage, dt);
        // 3. step2
        for c in 0..N {
            let (vel, acc) = storage.vel_col_mut_acc(c);
            for i in 0..vel.len() {
                VelocityVerlet::step2(&mut vel[i], acc[i], dt);
            }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for SoaVelocityVerletSolver<S, M, N>
where
    S: SoaNewtonianStorage,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

/// AoS velocity-verlet solver.
pub struct AosVelocityVerletSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> AosVelocityVerletSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for AosVelocityVerletSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosNewtonianItem<N>,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) {
        self.model.pre(storage, 0.0);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        // 1. step1
        for item in storage.iter_mut() {
            let (pos, vel, acc) = item.pos_vel_mut_acc();
            for i in 0..N {
                VelocityVerlet::step1(&mut pos[i], &mut vel[i], acc[i], dt);
            }
        }
        // 2. recompute forces
        self.model.pre(storage, dt);
        // 3. step2
        for item in storage.iter_mut() {
            let (vel, acc) = item.vel_mut_acc();
            for i in 0..N {
                VelocityVerlet::step2(&mut vel[i], acc[i], dt);
            }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for AosVelocityVerletSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosNewtonianItem<N>,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

