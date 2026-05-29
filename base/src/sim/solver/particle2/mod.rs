use std::marker::PhantomData;
use crate::sim::{solver::Solver, storage::{Storage, AosStorage}};
 

pub trait Particle<const N: usize> {
    fn pos(&self)         -> [f64; N];
    fn acc(&self)         -> [f64; N];
    fn set_pos(&mut self, v: [f64; N]);
    fn set_acc(&mut self, v: [f64; N]);


    fn pre_step(&mut self, dt: f64) {}  
    fn step(&mut self, dt: f64); 
    fn post_step(&mut self, _dt: f64) {}
}

pub trait NewtonianParticle<const N: usize>: Particle<N> {
    fn vel(&self)         -> [f64; N];
    fn set_vel(&mut self, v: [f64; N]);
}

pub trait VerletParticle<const N: usize>: Particle<N> {
    fn pos_old(&self)     -> [f64; N];
    fn set_pos_old(&mut self, v: [f64; N]);
}

// ---------------------------------------------------------------------------
// ForceModel — static composition via ForceChain, zero allocation
// ---------------------------------------------------------------------------

pub trait ForceModel<S: Storage> {
    fn apply(&mut self, storage: &mut S, dt: f64);
}

/// Chains two force models — fully inlined, zero heap.
/// Use nested: ForceChain(ClearAcc, ForceChain(Gravity, Drag))
pub struct ForceChain<A, B>(pub A, pub B);

impl<S: Storage, A: ForceModel<S>, B: ForceModel<S>> ForceModel<S> for ForceChain<A, B> {
    #[inline(always)]
    fn apply(&mut self, storage: &mut S, dt: f64) {
        self.0.apply(storage, dt);
        self.1.apply(storage, dt);
    }
}

/// No-op force — useful for free-flight or testing.
pub struct NoForce;

impl<S: Storage> ForceModel<S> for NoForce {
    #[inline(always)]
    fn apply(&mut self, _: &mut S, _: f64) {}
}

// ---------------------------------------------------------------------------
// ConstraintModel — same static chain pattern
// ---------------------------------------------------------------------------

pub trait ConstraintModel<S: Storage> {
    fn apply(&mut self, storage: &mut S, dt: f64);
}

pub struct ConstraintChain<A, B>(pub A, pub B);

impl<S: Storage, A: ConstraintModel<S>, B: ConstraintModel<S>> ConstraintModel<S> for ConstraintChain<A, B> {
    #[inline(always)]
    fn apply(&mut self, storage: &mut S, dt: f64) {
        self.0.apply(storage, dt);
        self.1.apply(storage, dt);
    }
}

/// No-op constraint.
pub struct NoConstraint;

impl<S: Storage> ConstraintModel<S> for NoConstraint {
    #[inline(always)]
    fn apply(&mut self, _: &mut S, _: f64) {}
}

// ---------------------------------------------------------------------------
// AosParticleSolver — one solver, behavior entirely in FM + C + Particle::integrate
// ---------------------------------------------------------------------------

pub struct AosParticleSolver<S, FM, C>
where
    S:  AosStorage,
    FM: ForceModel<S>,
    C:  ConstraintModel<S>,
{
    pub forces:      FM,
    pub constraints: C,
    _marker: PhantomData<S>,
}

impl<S, FM, C> AosParticleSolver<S, FM, C>
where
    S:  AosStorage,
    FM: ForceModel<S>,
    C:  ConstraintModel<S>,
{
    pub fn new(forces: FM, constraints: C) -> Self {
        Self { forces, constraints, _marker: PhantomData }
    }
}

impl<S, const N: usize, FM, C> Solver<S> for AosParticleSolver<S, FM, C>
where
    S:  AosStorage,
    FM: ForceModel<S>,
    C:  ConstraintModel<S>,
    S::Item: Particle<N>,
{
    fn init(&mut self, storage: &mut S) {
        // compute initial acc so first integrate() has correct forces
        self.forces.apply(storage, 0.0);
    }

    fn pre_step(&mut self, storage: &mut S, dt: f64, _tick: u64) {
        self.forces.apply(storage, dt);
        // leapfrog first half-kick (no-op for velocity/classic verlet particles)
        for p in storage.iter_mut() {
            p.integrate_post(dt * 0.5);
        }
    }

    fn sub_step(&mut self, storage: &mut S, dt: f64) {
        for p in storage.iter_mut() {
            p.integrate(dt);
        }
    }

    fn post_step(&mut self, storage: &mut S, dt: f64) {
        self.constraints.apply(storage, dt);
        // recompute forces then leapfrog second half-kick (no-op for others)
        self.forces.apply(storage, dt);
        for p in storage.iter_mut() {
            p.integrate_post(dt * 0.5);
        }
    }
}