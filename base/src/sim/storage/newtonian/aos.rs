use crate::sim::storage::AosStorage;

/// Per-item accessor contract for AoS newtonian storage.
/// Mirrors the combined-borrow API of [`SoaNewtonianStorage`] but at the item level,
/// so each accessor proves disjointness within a single item.
pub trait AosNewtonianItem<const N: usize> {
    /// Mutable acc — used by `ClearAcc` and `ConstantAccel`.
    fn acc_mut(&mut self) -> &mut [f64; N];

    /// Mutable vel — used by `NewtonianDamping`.
    fn vel_mut(&mut self) -> &mut [f64; N];

    /// `(vel, acc_mut)` — disjoint, used by `NewtonianLinearDrag`.
    fn vel_acc_mut(&mut self) -> (&[f64; N], &mut [f64; N]);

    /// `(pos_mut, vel_mut, acc)` — disjoint, used by `SymplecticEuler` and `VelocityVerlet::step1`.
    fn pos_vel_mut_acc(&mut self) -> (&mut [f64; N], &mut [f64; N], &[f64; N]);

    /// `(vel_mut, acc)` — disjoint, used by `Leapfrog::half_kick` and `VelocityVerlet::step2`.
    fn vel_mut_acc(&mut self) -> (&mut [f64; N], &[f64; N]);

    /// `(pos_mut, vel)` — disjoint, used by `Leapfrog::drift`.
    fn pos_mut_vel(&mut self) -> (&mut [f64; N], &[f64; N]);
}

/// Marker — an [`AosStorage`] whose items implement [`AosNewtonianItem<N>`].
/// The solver bounds are written directly as `S: AosStorage, S::Item: AosNewtonianItem<N>`
/// so this trait is optional, but useful as a single bound in user code.
pub trait AosNewtonianStorage<const N: usize>: AosStorage
where
    Self::Item: AosNewtonianItem<N>,
{}
