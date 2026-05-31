use crate::sim::storage::SoaStorage;

/// [`SoaStorage`] with `pos`, `vel`, `acc` columns.
/// Required by [`SymplecticEuler`], [`Leapfrog`], and [`VelocityVerlet`] solvers.
///
/// Column layout: one column per component, each of length `len()`.
/// i.e. `pos_col(0)` = all x values, `pos_col(1)` = all y values.
/// Stride-1 inner loops — LLVM can auto-vectorize.
///
/// Only combined accessors are exposed for fields that must be borrowed
/// simultaneously. Implementations prove disjointness via direct field
/// access or `split_at_mut`.
pub trait SoaNewtonianStorage: SoaStorage {
    /// One component column — length `len()`.
    fn pos_col(&self, c: usize) -> &[f64];
    fn vel_col(&self, c: usize) -> &[f64];
    fn acc_col(&self, c: usize) -> &[f64];

    /// Mutable acc column — used by `ClearAcc` and `ConstantAccel`.
    fn acc_col_mut(&mut self, c: usize) -> &mut [f64];

    /// Mutable vel column — used by `Damping`.
    fn vel_col_mut(&mut self, c: usize) -> &mut [f64];

    /// `(vel, acc_mut)` — disjoint, used by `LinearDrag`.
    fn vel_col_acc_col_mut(&mut self, c: usize) -> (&[f64], &mut [f64]);

    /// `(pos_mut, vel_mut, acc)` — disjoint, used by `SymplecticEuler` and `VelocityVerlet::step1`.
    fn pos_vel_col_mut_acc(&mut self, c: usize) -> (&mut [f64], &mut [f64], &[f64]);

    /// `(vel_mut, acc)` — disjoint, used by `Leapfrog::half_kick` and `VelocityVerlet::step2`.
    fn vel_col_mut_acc(&mut self, c: usize) -> (&mut [f64], &[f64]);

    /// `(pos_mut, vel)` — disjoint, used by `Leapfrog::drift`.
    fn pos_col_mut_vel(&mut self, c: usize) -> (&mut [f64], &[f64]);
}

/// Tests the [`SoaNewtonianStorage`] contract.
/// `$n` is the number of spatial components (column count per field).
#[macro_export]
macro_rules! test_soa_newtonian_storage {
    ($storage:ty, $item:ty, $n:expr) => {
        #[cfg(test)]
        mod soa_newtonian_storage_tests {
            use super::*;

            #[test]
            fn pos_col_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                for c in 0..$n { assert_eq!(s.pos_col(c).len(), s.len()); }
            }

            #[test]
            fn vel_col_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                for c in 0..$n { assert_eq!(s.vel_col(c).len(), s.len()); }
            }

            #[test]
            fn acc_col_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                for c in 0..$n { assert_eq!(s.acc_col(c).len(), s.len()); }
            }

            #[test]
            fn pos_vel_col_mut_acc_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                for c in 0..$n {
                    let (pos, vel, acc) = s.pos_vel_col_mut_acc(c);
                    assert_eq!(pos.len(), s.len());
                    assert_eq!(vel.len(), s.len());
                    assert_eq!(acc.len(), s.len());
                }
            }

            #[test]
            fn vel_col_mut_acc_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                for c in 0..$n {
                    let (vel, acc) = s.vel_col_mut_acc(c);
                    assert_eq!(vel.len(), acc.len());
                }
            }

            #[test]
            fn pos_col_mut_vel_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                for c in 0..$n {
                    let (pos, vel) = s.pos_col_mut_vel(c);
                    assert_eq!(pos.len(), vel.len());
                }
            }

            #[test]
            fn acc_col_mut_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                for c in 0..$n { assert_eq!(s.acc_col_mut(c).len(), s.len()); }
            }

            #[test]
            fn vel_col_mut_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                for c in 0..$n { assert_eq!(s.vel_col_mut(c).len(), s.len()); }
            }

            #[test]
            fn vel_col_acc_col_mut_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                for c in 0..$n {
                    let (vel, acc) = s.vel_col_acc_col_mut(c);
                    assert_eq!(vel.len(), acc.len());
                }
            }

            #[test]
            fn clear_empties_columns() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.clear();
                for c in 0..$n {
                    assert!(s.pos_col(c).is_empty());
                    assert!(s.vel_col(c).is_empty());
                    assert!(s.acc_col(c).is_empty());
                }
            }
        }
    };
}
