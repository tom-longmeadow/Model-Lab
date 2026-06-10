use crate::sim::storage::{SoaLayout, SoaCpuStorage, soa_vec::SoaVecStorage};

/// Marks a [`SoaLayout`] type whose column order is:
/// `[pos_0, …, pos_{N-1}, pos_old_0, …, pos_old_{N-1}, acc_0, …, acc_{N-1}]`
/// with `N` components per field and `f64` element type.
///
/// Implementing this on your particle type automatically provides
/// [`SoaVerletStorage`] for `SoaVecStorage<Self>`.
pub trait SoaVerletLayout: SoaLayout {
    const N: usize;
}

impl<T: SoaVerletLayout> SoaVerletStorage for SoaVecStorage<T> {
    fn pos_col(&self, c: usize) -> &[f64] {
        unsafe { self.col_raw::<f64>(c) }
    }
    fn pos_old_col(&self, c: usize) -> &[f64] {
        unsafe { self.col_raw::<f64>(T::N + c) }
    }
    fn acc_col(&self, c: usize) -> &[f64] {
        unsafe { self.col_raw::<f64>(T::N * 2 + c) }
    }
    fn acc_col_mut(&mut self, c: usize) -> &mut [f64] {
        unsafe { self.col_raw_mut::<f64>(T::N * 2 + c) }
    }
    fn pos_pos_old_col_mut_acc(&mut self, c: usize) -> (&mut [f64], &mut [f64], &[f64]) {
        unsafe { self.col3_mut_mut_ref::<f64>(c, T::N + c, T::N * 2 + c) }
    }
}

/// [`SoaStorage`] with `pos`, `pos_old`, `acc` columns.
/// Required by the [`Verlet`] solver.
///
/// Column layout: one column per component, each of length `len()`.
/// i.e. `pos_col(0)` = all x values, `pos_col(1)` = all y values.
/// Stride-1 inner loops — LLVM can auto-vectorize.
pub trait SoaVerletStorage: SoaCpuStorage {
    /// One component column — length `len()`.
    fn pos_col(&self, c: usize)     -> &[f64];
    fn pos_old_col(&self, c: usize) -> &[f64];
    fn acc_col(&self, c: usize)     -> &[f64];

    /// Mutable acc column — used by `ClearAcc` and `ConstantAccel`.
    fn acc_col_mut(&mut self, c: usize) -> &mut [f64];

    /// `(pos_mut, pos_old_mut, acc)` — disjoint, used by `Verlet::step`.
    fn pos_pos_old_col_mut_acc(&mut self, c: usize) -> (&mut [f64], &mut [f64], &[f64]);
}

/// Tests the [`SoaVerletStorage`] contract.
/// `$item` must implement SoaLayout + Default.
/// `$n` is the number of spatial components (column count per field).
#[macro_export]
macro_rules! test_soa_verlet_storage {
    ($item:ty, $n:expr) => {
        #[cfg(test)]
        mod soa_verlet_storage_tests {
            use super::*;
            use $crate::sim::storage::{CpuStorage, Storage, SoaLayout, SoaCpuStorage, soa_vec::SoaVecStorage};

            type TestStorage = SoaVecStorage<$item>;

            fn push_item(s: &mut TestStorage, item: $item) {
                item.push_cols(s.columns_mut());
                s.increment_len();
            }

            #[test]
            fn pos_col_len_matches_storage_len() {
                let mut s = TestStorage::new(10);
                push_item(&mut s, <$item>::default());
                push_item(&mut s, <$item>::default());
                for c in 0..$n { assert_eq!(s.pos_col(c).len(), s.len()); }
            }

            #[test]
            fn pos_old_col_len_matches_storage_len() {
                let mut s = TestStorage::new(10);
                push_item(&mut s, <$item>::default());
                push_item(&mut s, <$item>::default());
                for c in 0..$n { assert_eq!(s.pos_old_col(c).len(), s.len()); }
            }

            #[test]
            fn acc_col_len_matches_storage_len() {
                let mut s = TestStorage::new(10);
                push_item(&mut s, <$item>::default());
                push_item(&mut s, <$item>::default());
                for c in 0..$n { assert_eq!(s.acc_col(c).len(), s.len()); }
            }

            #[test]
            fn pos_pos_old_col_mut_acc_lengths_match() {
                let mut s = TestStorage::new(10);
                push_item(&mut s, <$item>::default());
                push_item(&mut s, <$item>::default());
                let expected_len = s.len();
                for c in 0..$n {
                    let (pos, pos_old, acc) = s.pos_pos_old_col_mut_acc(c);
                    assert_eq!(pos.len(),     expected_len);
                    assert_eq!(pos_old.len(), expected_len);
                    assert_eq!(acc.len(),     expected_len);
                }
            }

            #[test]
            fn acc_col_mut_len_matches_storage_len() {
                let mut s = TestStorage::new(10);
                push_item(&mut s, <$item>::default());
                push_item(&mut s, <$item>::default());
                for c in 0..$n { assert_eq!(s.acc_col_mut(c).len(), s.len()); }
            }

            #[test]
            fn clear_empties_columns() {
                let mut s = TestStorage::new(10);
                push_item(&mut s, <$item>::default());
                s.clear();
                for c in 0..$n {
                    assert!(s.pos_col(c).is_empty());
                    assert!(s.pos_old_col(c).is_empty());
                    assert!(s.acc_col(c).is_empty());
                }
            }
        }
    };
}