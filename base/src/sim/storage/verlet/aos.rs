use crate::sim::storage::AosStorage;

/// Per-item accessor contract for AoS verlet storage.
/// Mirrors the combined-borrow API of [`SoaVerletStorage`] but at the item level.
pub trait AosVerletItem<const N: usize> {
    /// Mutable acc — used by `ClearAcc` and `ConstantAccel`.
    fn acc_mut(&mut self) -> &mut [f64; N];

    /// `(pos_mut, pos_old_mut, acc)` — disjoint, used by `Verlet::step`.
    fn pos_pos_old_mut_acc(&mut self) -> (&mut [f64; N], &mut [f64; N], &[f64; N]);
}

/// Marker — an [`AosStorage`] whose items implement [`AosVerletItem<N>`].
pub trait AosVerletStorage<const N: usize>: AosStorage
where
    Self::Item: AosVerletItem<N>,
{}
