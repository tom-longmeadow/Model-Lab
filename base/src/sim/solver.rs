use std::marker::PhantomData;
use crate::sim::storage::{Storage, AosStorage, SoaStorage, SoaNewtonianStorage, SoaVerletStorage};

// ---------------------------------------------------------------------------
// Solver
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// StepModel
// ---------------------------------------------------------------------------

/// Universal behavior slot — runs against the whole storage before and after integration.
/// Forces, constraints, advection, behaviour — all implement this.
/// Layout-agnostic: works with [`AosStorage`] and [`SoaStorage`].
/// - `pre()`  — before integration: forces, advection, behaviour
/// - `post()` — after  integration: constraints, pressure, leapfrog second half-kick
pub trait StepModel<S: Storage> {
    fn pre(&mut self,  _storage: &mut S, _dt: f64) {}
    fn post(&mut self, _storage: &mut S, _dt: f64) {}
}

/// No-op [`StepModel`] — use when no behavior is needed.
pub struct NoModel;
impl<S: Storage> StepModel<S> for NoModel {}

/// Chains two [`StepModel`]s — zero allocation, fully inlined.
/// Use the [`chain!`] macro for more than two.
pub struct ModelChain<A, B>(pub A, pub B);
impl<S: Storage, A: StepModel<S>, B: StepModel<S>> StepModel<S> for ModelChain<A, B> {
    #[inline(always)]
    fn pre(&mut self, storage: &mut S, dt: f64) {
        self.0.pre(storage, dt);
        self.1.pre(storage, dt);
    }
    #[inline(always)]
    fn post(&mut self, storage: &mut S, dt: f64) {
        self.0.post(storage, dt);
        self.1.post(storage, dt);
    }
}

//// Chains any number of [`StepModel`]s into a [`ModelChain`] — zero allocation, fully inlined.
/// ```ignore
/// let model = chain!(ClearAcc, Gravity::new(9.81), Drag::new(0.01));
/// ```
#[macro_export] 
macro_rules! chain {
    ($a:expr) => { $a };
    ($a:expr, $($rest:expr),+) => {
        $crate::sim::solver::ModelChain($a, $crate::chain!($($rest),+))
    };
}

// ---------------------------------------------------------------------------
// Particle state traits
// ---------------------------------------------------------------------------

/// Base particle state — position and acceleration.
/// Returns references — zero-copy, direct in-place modification by integrators.
/// Combined accessors required because `&mut self` cannot yield two `&mut` separately.
pub trait Particle<const N: usize> {
    fn pos(&self) -> &[f64; N];
    fn acc(&self) -> &[f64; N];

    /// Returns `(pos_mut, acc)` — disjoint, used by force-only models.
    fn pos_mut_acc(&mut self) -> (&mut [f64; N], &[f64; N]);
}

/// Extends [`Particle`] with explicit velocity.
/// Required by [`NewtonianVerlet`] and [`Leapfrog`] integrators.
/// Returns references — zero-copy, direct in-place modification by integrators.
pub trait NewtonianParticle<const N: usize>: Particle<N> {
    fn vel(&self) -> &[f64; N];

    /// Returns `(pos_mut, vel_mut, acc)` — disjoint, used by [`NewtonianVerlet`].
    fn pos_vel_mut_acc(&mut self) -> (&mut [f64; N], &mut [f64; N], &[f64; N]);

    /// Returns `(vel_mut, acc)` — disjoint, used by [`Leapfrog`] half-kick.
    fn vel_mut_acc(&mut self) -> (&mut [f64; N], &[f64; N]);

    /// Returns `(pos_mut, vel)` — disjoint, used by [`Leapfrog`] drift.
    fn pos_mut_vel(&mut self) -> (&mut [f64; N], &[f64; N]);
}

/// Extends [`Particle`] with previous position.
/// Required by the [`Verlet`] integrator. Velocity is implicit in the position difference.
/// Returns references — zero-copy, direct in-place modification by integrators.
pub trait VerletParticle<const N: usize>: Particle<N> {
    fn pos_old(&self) -> &[f64; N];

    /// Returns `(pos_mut, pos_old_mut, acc)` — disjoint, used by [`Verlet`].
    fn pos_pos_old_mut_acc(&mut self) -> (&mut [f64; N], &mut [f64; N], &[f64; N]);
}

// ---------------------------------------------------------------------------
// Integrators — pure functions over data.
// No storage, no traits, no heap. Just math on references or slices.
// scalar form: one item   at a time → AosSolver (compiler inlines per item)
// slice  form: one column at a time → SoaSolver (compiler can auto-vectorize / SIMD)
//
// SoA slice layout: blocked — all x values, then all y values etc.
// i.e. pos = [x0, x1, x2, ..., y0, y1, y2, ...]  not interleaved.
//
// All functions take references — zero copying, data modified in place.
// ---------------------------------------------------------------------------

/// Newtonian (velocity) Verlet: `vel += acc * dt`, `pos += vel * dt`.
/// Requires [`NewtonianParticle<N>`] — needs explicit velocity.
pub struct NewtonianVerlet;
impl NewtonianVerlet {
    #[inline(always)]
    pub fn step_scalar<const N: usize>(
        pos: &mut [f64; N], vel: &mut [f64; N], acc: &[f64; N], dt: f64,
    ) {
        for i in 0..N {
            vel[i] += acc[i] * dt;
            pos[i] += vel[i] * dt;
        }
    }

    #[inline(always)]
    pub fn step_slice(pos: &mut [f64], vel: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..pos.len() {
            vel[i] += acc[i] * dt;
            pos[i] += vel[i] * dt;
        }
    }
}

/// Classic (Störmer) Verlet: `pos_new = 2*pos - pos_old + acc * dt²`.
/// Requires [`VerletParticle<N>`] — needs previous position.
/// Velocity is implicit in the position difference.
pub struct Verlet;
impl Verlet {
    #[inline(always)]
    pub fn step_scalar<const N: usize>(
        pos: &mut [f64; N], pos_old: &mut [f64; N], acc: &[f64; N], dt: f64,
    ) {
        for i in 0..N {
            let new    = 2.0 * pos[i] - pos_old[i] + acc[i] * dt * dt;
            pos_old[i] = pos[i];
            pos[i]     = new;
        }
    }

    #[inline(always)]
    pub fn step_slice(pos: &mut [f64], pos_old: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..pos.len() {
            let new    = 2.0 * pos[i] - pos_old[i] + acc[i] * dt * dt;
            pos_old[i] = pos[i];
            pos[i]     = new;
        }
    }
}

/// Leapfrog (symplectic). Split into half-kick and drift.
/// Requires [`NewtonianParticle<N>`] — needs explicit velocity.
/// Sequence per substep: `half_kick` → `drift` → recompute forces → `half_kick`
/// Conserves energy better than [`NewtonianVerlet`] over long runs.
pub struct Leapfrog;
impl Leapfrog {
    #[inline(always)]
    pub fn half_kick_scalar<const N: usize>(vel: &mut [f64; N], acc: &[f64; N], dt: f64) {
        for i in 0..N { vel[i] += acc[i] * dt * 0.5; }
    }

    #[inline(always)]
    pub fn drift_scalar<const N: usize>(pos: &mut [f64; N], vel: &[f64; N], dt: f64) {
        for i in 0..N { pos[i] += vel[i] * dt; }
    }

    #[inline(always)]
    pub fn half_kick_slice(vel: &mut [f64], acc: &[f64], dt: f64) {
        for i in 0..vel.len() { vel[i] += acc[i] * dt * 0.5; }
    }

    #[inline(always)]
    pub fn drift_slice(pos: &mut [f64], vel: &[f64], dt: f64) {
        for i in 0..pos.len() { pos[i] += vel[i] * dt; }
    }
}

// ---------------------------------------------------------------------------
// Solver marker traits
// ---------------------------------------------------------------------------

/// A [`Solver`] that owns a [`StepModel`].
pub trait StepModelSolver<S: Storage>: Solver<S> {
    type Model: StepModel<S>;
    fn model(&mut self) -> &mut Self::Model;
}

/// A [`StepModelSolver`] over [`AosStorage`].
/// `sub_step` calls the scalar integrator once per item via `iter_mut()`.
/// Data is modified in place through references — zero copying.
pub trait AosStepSolver<S: AosStorage>: StepModelSolver<S> {}

/// A [`StepModelSolver`] over [`SoaStorage`].
/// `sub_step` calls the slice integrator over whole columns.
/// Data is modified in place through column slices — compiler can auto-vectorize.
pub trait SoaStepSolver<S: SoaStorage>: StepModelSolver<S> {}

/// A solver using [`NewtonianVerlet`] integration.
/// Items must provide [`NewtonianParticle<N>`].
pub trait NewtonianSolver<S: Storage, const N: usize>: StepModelSolver<S> {}

/// A solver using [`Verlet`] integration.
/// Items must provide [`VerletParticle<N>`].
pub trait VerletSolver<S: Storage, const N: usize>: StepModelSolver<S> {}

/// A solver using [`Leapfrog`] integration.
/// Items must provide [`NewtonianParticle<N>`].
/// Symplectic — conserves energy better than [`NewtonianVerlet`] over long runs.
pub trait LeapfrogSolver<S: Storage, const N: usize>: StepModelSolver<S> {}

// ---------------------------------------------------------------------------
// AosSolver
// ---------------------------------------------------------------------------

/// Concrete AoS solver.
/// - `M` — the [`StepModel`], use [`chain!`] to compose: `chain!(ClearAcc, Gravity, Drag)`
/// - `I` — the integrator: [`NewtonianVerlet`], [`Verlet`], [`Leapfrog`]
/// - `N` — spatial dimension
///
/// `sub_step` modifies particle data in place through references — zero copying.
/// One struct satisfies [`NewtonianSolver`], [`VerletSolver`], or [`LeapfrogSolver`]
/// automatically based on `I` and the item type of `S`.
pub struct AosSolver<M, I, const N: usize> {
    pub model:   M,
    _integrator: PhantomData<I>,
}

impl<M, I, const N: usize> AosSolver<M, I, N> {
    pub fn new(model: M) -> Self { Self { model, _integrator: PhantomData } }
}

impl<S, M, const N: usize> Solver<S> for AosSolver<M, NewtonianVerlet, N>
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
            NewtonianVerlet::step_scalar(pos, vel, acc, dt);
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

impl<S, M, const N: usize> NewtonianSolver<S, N> for AosSolver<M, NewtonianVerlet, N>
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

// ---------------------------------------------------------------------------
// SoaSolver
// ---------------------------------------------------------------------------

/// Concrete SoA solver.
/// - `M` — the [`StepModel`], use [`chain!`] to compose: `chain!(ClearAcc, Gravity, Drag)`
/// - `I` — the integrator: [`NewtonianVerlet`], [`Verlet`], [`Leapfrog`]
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

impl<S, M, const N: usize> Solver<S> for SoaSolver<M, NewtonianVerlet, N>
where
    S: SoaStorage + SoaNewtonianStorage,
    M: StepModel<S>,
{
    fn init(&mut self, storage: &mut S) { self.model.pre(storage, 0.0); }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        self.model.pre(storage, dt);
        let (pos, vel, acc) = storage.pos_vel_mut_acc();
        NewtonianVerlet::step_slice(pos, vel, acc, dt);
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

impl<S, M, const N: usize> NewtonianSolver<S, N> for SoaSolver<M, NewtonianVerlet, N>
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