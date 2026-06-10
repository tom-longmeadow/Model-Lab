use crate::sim::storage::AosCpuStorage;

/// Per-item accessor contract for AoS verlet storage.
/// Mirrors the combined-borrow API of [`SoaVerletStorage`] but at the item level.
pub trait AosVerletItem<const N: usize> {
    /// Mutable acc — used by `ClearAcc` and `ConstantAccel`.
    fn acc_mut(&mut self) -> &mut [f64; N];

    /// `(pos_mut, pos_old_mut, acc)` — disjoint, used by `Verlet::step`.
    fn pos_pos_old_mut_acc(&mut self) -> (&mut [f64; N], &mut [f64; N], &[f64; N]);
}

/// Marker — an [`AosStorage`] whose items implement [`AosVerletItem<N>`].
pub trait AosVerletStorage<const N: usize>: AosCpuStorage
where
    Self::Item: AosVerletItem<N>,
{}

/// Tests the [`AosVerletItem`] contract on a concrete [`AosStorage`].
/// `$item` must implement `AosVerletItem<$n>`, `Default`, and `AosStorage` methods.
/// `$n` is the number of spatial components.
#[macro_export]
macro_rules! test_aos_verlet_storage {
    ($storage:ty, $item:ty, $n:expr) => {
        #[cfg(test)]
        mod aos_verlet_storage_tests {
            use super::*;
            use $crate::sim::storage::{CpuStorage, AosCpuStorage, verlet::aos::AosVerletItem};

            #[test]
            fn acc_mut_len_is_n() {
                let mut s = <$storage>::new(4);
                s.push(<$item>::default());
                let p = s.get_mut(0);
                assert_eq!(p.acc_mut().len(), $n);
            }

            #[test]
            fn pos_pos_old_mut_acc_lens_are_n() {
                let mut s = <$storage>::new(4);
                s.push(<$item>::default());
                let p = s.get_mut(0);
                let (pos, pos_old, acc): (&mut [f64; $n], &mut [f64; $n], &[f64; $n]) =
                    p.pos_pos_old_mut_acc();
                assert_eq!(pos.len(),     $n);
                assert_eq!(pos_old.len(), $n);
                assert_eq!(acc.len(),     $n);
            }

            #[test]
            fn acc_mut_write_is_visible_via_pos_pos_old_mut_acc() {
                let mut s = <$storage>::new(4);
                s.push(<$item>::default());
                {
                    let p = s.get_mut(0);
                    let acc: &mut [f64; $n] = p.acc_mut();
                    for a in acc.iter_mut() { *a = 42.0; }
                }
                {
                    let p = s.get_mut(0);
                    let (_, _, acc): (&mut [f64; $n], &mut [f64; $n], &[f64; $n]) =
                        p.pos_pos_old_mut_acc();
                    for &a in acc.iter() { assert_eq!(a, 42.0); }
                }
            }

            #[test]
            fn pos_write_is_independent_of_pos_old() {
                let mut s = <$storage>::new(4);
                s.push(<$item>::default());
                {
                    let p = s.get_mut(0);
                    let (pos, pos_old, _): (&mut [f64; $n], &mut [f64; $n], &[f64; $n]) =
                        p.pos_pos_old_mut_acc();
                    for v in pos.iter_mut()     { *v = 1.0; }
                    for v in pos_old.iter_mut() { *v = 2.0; }
                }
                {
                    let p2 = s.get_mut(0);
                    let (pos2, pos_old2, _): (&mut [f64; $n], &mut [f64; $n], &[f64; $n]) =
                        p2.pos_pos_old_mut_acc();
                    for &v in pos2.iter()     { assert_eq!(v, 1.0); }
                    for &v in pos_old2.iter() { assert_eq!(v, 2.0); }
                }
            }

            #[test]
            fn works_with_multiple_particles() {
                let mut s = <$storage>::new(8);
                s.push(<$item>::default());
                s.push(<$item>::default());
                for i in 0..s.len() {
                    let p = s.get_mut(i);
                    let acc: &mut [f64; $n] = p.acc_mut();
                    for a in acc.iter_mut() { *a = i as f64; }
                }
                for i in 0..s.len() {
                    let p = s.get_mut(i);
                    let (_, _, acc): (&mut [f64; $n], &mut [f64; $n], &[f64; $n]) =
                        p.pos_pos_old_mut_acc();
                    for &a in acc.iter() { assert_eq!(a, i as f64); }
                }
            }
        }
    };
}
