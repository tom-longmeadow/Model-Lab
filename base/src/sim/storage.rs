
pub mod aos_vec;
pub mod soa_vec;
 
/// Represents any simulation storage.
/// Makes no assumptions about memory layout, threading,
/// buffering, or the type of simulation.
pub trait Storage {
    /// The unit of data this storage holds.
    /// Could be a particle, a grid cell, a node, or a scalar field value.
    type Item;

    /// Creates storage with an initial capacity hint.
    fn new(capacity: usize) -> Self;

    /// Number of items currently stored.
    fn len(&self) -> usize;

    /// Maximum items before reallocation or overflow.
    fn capacity(&self) -> usize;

    fn is_empty(&self) -> bool { self.len() == 0 }

    /// Add an entity.
    fn push(&mut self, item: Self::Item);

    /// Remove by index — swaps with last, O(1), order not preserved.
    fn swap_remove(&mut self, index: usize) -> Self::Item;

    /// Called before the solver starts a step.
    /// Use for buffer swaps, memory mapping, fence waits.
    fn pre_step(&mut self)  {}

    /// Called after the solver completes a step.
    /// Use for GPU uploads, unmapping, signalling.
    fn post_step(&mut self) {}

    /// Remove all entities.
    fn clear(&mut self);
}


/// Array-of-Structs storage — contiguous slice with index helpers.
/// Implement as_slice/as_slice_mut; get/get_mut are derived for free.
pub trait AosStorage: Storage {
    fn as_slice(&self)         -> &[Self::Item];
    fn as_slice_mut(&mut self) -> &mut [Self::Item];

    fn get(&self, i: usize)         -> &Self::Item     { &self.as_slice()[i] }
    fn get_mut(&mut self, i: usize) -> &mut Self::Item { &mut self.as_slice_mut()[i] }

    fn iter(&self)         -> std::slice::Iter<'_, Self::Item>    { self.as_slice().iter() }
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Item> { self.as_slice_mut().iter_mut() }

    fn swap(&mut self, a: usize, b: usize) { self.as_slice_mut().swap(a, b); } 
}

/// Struct-of-Arrays storage — per-field column slices.
/// The item type declares its own layout via SoaLayout.
/// Enables SIMD over individual fields.
pub trait SoaStorage: Storage {
    fn col<C: 'static>(&self, index: usize)         -> &[C];
    fn col_mut<C: 'static>(&mut self, index: usize) -> &mut [C];
}



/// Tests the base Storage contract.
/// Any type implementing Storage should pass these.
#[macro_export]
macro_rules! test_storage {
    ($storage:ty, $item:ty) => {
        #[cfg(test)]
        mod storage_tests {
            use super::*;
            use base::sim::storage::Storage;

            #[test]
            fn test_new_has_correct_capacity() {
                let s = <$storage>::new(10);
                assert_eq!(s.capacity(), 10);
            }

            #[test]
            fn test_new_is_empty() {
                let s = <$storage>::new(10);
                assert_eq!(s.len(), 0);
                assert!(s.is_empty());
            }

            #[test]
            fn test_push_increases_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                assert_eq!(s.len(), 1);
                s.push(<$item>::default());
                assert_eq!(s.len(), 2);
            }

            #[test]
            fn test_swap_remove_decreases_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                s.swap_remove(0);
                assert_eq!(s.len(), 1);
            }

            #[test]
            fn test_swap_remove_last_empties() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.swap_remove(0);
                assert!(s.is_empty());
            }

            #[test]
            fn test_clear_empties_storage() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                s.clear();
                assert!(s.is_empty());
            }
        }
    };
}

/// Tests the AosStorage contract.
/// Any type implementing AosStorage should pass these.
#[macro_export]
macro_rules! test_aos_storage {
    ($storage:ty, $item:ty) => {
        #[cfg(test)]
        mod aos_storage_tests {
            use super::*;
            use base::sim::storage::{AosStorage, Storage};

            #[test]
            fn test_slice_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.as_slice().len(), s.len());
            }

            #[test]
            fn test_slice_empty_when_storage_empty() {
                let s = <$storage>::new(10);
                assert!(s.as_slice().is_empty());
            }

            #[test]
            fn test_get_matches_slice() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                let from_get   = s.get(0) as *const $item;
                let from_slice = &s.as_slice()[0] as *const $item;
                assert_eq!(from_get, from_slice);
            }

            #[test]
            fn test_get_mut_matches_slice_mut() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                let from_get   = s.get_mut(0) as *mut $item;
                let from_slice = &mut s.as_slice_mut()[0] as *mut $item;
                assert_eq!(from_get, from_slice);
            }

            #[test]
            fn test_swap_swaps_slice_positions() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                let ptr_0_before = s.get(0) as *const $item;
                let ptr_1_before = s.get(1) as *const $item;
                s.swap(0, 1);
                let ptr_0_after  = s.get(0) as *const $item;
                let ptr_1_after  = s.get(1) as *const $item;
                assert_ne!(ptr_0_before, ptr_0_after);
                assert_ne!(ptr_1_before, ptr_1_after);
            }

            #[test]
            fn test_iter_count_matches_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.iter().count(), s.len());
            }

            #[test]
            fn test_clear_empties_slice() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.clear();
                assert!(s.as_slice().is_empty());
            }
        }
    };
}

/// Tests the SoaStorage contract.
/// Any type implementing SoaStorage should pass these.
/// `$col` is a valid column index, `$col_type` is its field type.
#[macro_export]
macro_rules! test_soa_storage {
    ($storage:ty, $item:ty, $col:expr, $col_type:ty) => {
        #[cfg(test)]
        mod soa_storage_tests {
            use super::*;
            use base::sim::storage::{SoaStorage, Storage};

            #[test]
            fn test_col_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                let col: &[$col_type] = s.col($col);
                assert_eq!(col.len(), s.len());
            }

            #[test]
            fn test_col_empty_when_storage_empty() {
                let s = <$storage>::new(10);
                let col: &[$col_type] = s.col($col);
                assert!(col.is_empty());
            }

            #[test]
            fn test_col_mut_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                let col: &[$col_type] = s.col($col);
                assert_eq!(col.len(), 1);
            }

            #[test]
            fn test_clear_empties_col() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.clear();
                let col: &[$col_type] = s.col($col);
                assert!(col.is_empty());
            }
        }
    };
}
 
 