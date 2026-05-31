
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
/// Required by [`SymplecticEuler`] and [`Leapfrog`] integrators.
/// Returns references — zero-copy, direct in-place modification by integrators.
pub trait NewtonianParticle<const N: usize>: Particle<N> {
    fn vel(&self) -> &[f64; N];

    /// Returns `(pos_mut, vel_mut, acc)` — disjoint, used by [`SymplecticEuler`].
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