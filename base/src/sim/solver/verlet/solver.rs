use std::marker::PhantomData;

use crate::sim::storage::{
    AosCpuStorage,
    verlet::soa::SoaVerletStorage,
    verlet::aos::AosVerletItem,
};
use crate::sim::solver::{
    Solver, StepModel, StepModelSolver,
};
use crate::sim::solver::integrator::Verlet;

// ---------------------------------------------------------------------------
// Verlet
// ---------------------------------------------------------------------------
// Sequence per step:
//   pre_step:  model.pre  — clear acc, accumulate forces at current positions
//   sub_step:  pos_new = 2·pos − pos_old + acc·dt²
//   post_step: model.post — constraints, damping

/// SoA Verlet solver.
/// Forces accumulated once per step, then each column integrated in a
/// stride-1 inner loop that LLVM can auto-vectorise.
pub struct SoaVerletSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> SoaVerletSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for SoaVerletSolver<S, M, N>
where
    S: SoaVerletStorage,
    M: StepModel<S>,
{
    fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
        self.model.pre(storage, dt);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        for c in 0..N {
            let (pos, pos_old, acc) = storage.pos_pos_old_col_mut_acc(c);
            for i in 0..pos.len() {
                Verlet::step(&mut pos[i], &mut pos_old[i], acc[i], dt);
            }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for SoaVerletSolver<S, M, N>
where
    S: SoaVerletStorage,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

/// AoS Verlet solver.
/// Inner loop over N components is const-generic, unrolled by the compiler.
pub struct AosVerletSolver<S, M, const N: usize> {
    pub model: M,
    _p: PhantomData<S>,
}

impl<S, M, const N: usize> AosVerletSolver<S, M, N> {
    pub fn new(model: M) -> Self { Self { model, _p: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for AosVerletSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosVerletItem<N>,
    M: StepModel<S>,
{
    fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
        self.model.pre(storage, dt);
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        for item in storage.iter_mut() {
            let (pos, pos_old, acc) = item.pos_pos_old_mut_acc();
            for i in 0..N {
                Verlet::step(&mut pos[i], &mut pos_old[i], acc[i], dt);
            }
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.model.post(storage, dt);
    }
}

impl<S, M, const N: usize> StepModelSolver<S> for AosVerletSolver<S, M, N>
where
    S: AosCpuStorage,
    S::Item: AosVerletItem<N>,
    M: StepModel<S>,
{
    type Model = M;
    fn model(&mut self) -> &mut M { &mut self.model }
}

