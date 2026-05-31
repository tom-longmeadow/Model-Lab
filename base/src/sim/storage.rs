pub mod newtonian;
pub mod verlet;
pub mod aos_vec;
pub mod soa_vec;

pub use aos_vec::AosVecStorage;
pub use soa_vec::SoaVecStorage;

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

    /// Add an item.
    fn push(&mut self, item: Self::Item);

    /// Remove by index — swaps with last, O(1), order not preserved.
    fn swap_remove(&mut self, index: usize) -> Self::Item;

    /// Called before the solver starts a step.
    /// Use for buffer swaps, memory mapping, fence waits.
    fn pre_step(&mut self)  {}

    /// Called after the solver completes a step.
    /// Use for GPU uploads, unmapping, signalling.
    fn post_step(&mut self) {}

    /// Remove all items.
    fn clear(&mut self);
}

/// Array-of-Structs storage — items held as a contiguous slice.
/// Implement `as_slice`/`as_slice_mut`; everything else is derived.
pub trait AosStorage: Storage {
    fn as_slice(&self)         -> &[Self::Item];
    fn as_slice_mut(&mut self) -> &mut [Self::Item];

    fn get(&self, i: usize)         -> &Self::Item     { &self.as_slice()[i] }
    fn get_mut(&mut self, i: usize) -> &mut Self::Item { &mut self.as_slice_mut()[i] }

    fn iter(&self)         -> std::slice::Iter<'_, Self::Item>    { self.as_slice().iter() }
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Item> { self.as_slice_mut().iter_mut() }

    fn swap(&mut self, a: usize, b: usize) { self.as_slice_mut().swap(a, b); }
}

/// Struct-of-Arrays storage — data held in per-field column slices.
/// Column access is exposed only through named sub-traits
/// ([`SoaNewtonianStorage`], [`SoaVerletStorage`]) — no generic `col<C>`
/// API is provided because a runtime index cannot be verified against a
/// field type at compile time.
pub trait SoaStorage: Storage {}

/// Describes the column layout of a type that can live in SoA storage.
/// Implement this on your entity struct — not on the storage.
/// The storage impl uses this to manage raw byte columns; it knows nothing
/// about field semantics. Typed safe accessors live in the physics sub-traits.
pub trait SoaLayout: Sized {
    /// Byte stride of each column — one entry per field.
    const STRIDES: &'static [usize];

    /// Push all fields into their respective byte columns.
    fn push_cols(&self, cols: &mut [Vec<u8>]);

    /// Reconstruct `Self` from byte columns at `index`.
    fn read_cols(cols: &[Vec<u8>], index: usize) -> Self;

    /// Swap-remove at `index` — keeps all columns in sync.
    fn swap_remove_cols(cols: &mut [Vec<u8>], strides: &[usize], index: usize);
}

// ---------------------------------------------------------------------------
// Test macros — any concrete impl invokes these to verify the contract.
// ---------------------------------------------------------------------------

/// Tests the [`Storage`] contract.
/// `$item` must implement `Default`.
#[macro_export]
macro_rules! test_storage {
    ($storage:ty, $item:ty) => {
        #[cfg(test)]
        mod storage_tests {
            use super::*;

            #[test]
            fn new_has_correct_capacity() {
                let s = <$storage>::new(10);
                assert_eq!(s.capacity(), 10);
            }

            #[test]
            fn new_is_empty() {
                let s = <$storage>::new(10);
                assert_eq!(s.len(), 0);
                assert!(s.is_empty());
            }

            #[test]
            fn push_increases_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                assert_eq!(s.len(), 1);
                s.push(<$item>::default());
                assert_eq!(s.len(), 2);
            }

            #[test]
            fn swap_remove_decreases_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                s.swap_remove(0);
                assert_eq!(s.len(), 1);
            }

            #[test]
            fn swap_remove_last_empties() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.swap_remove(0);
                assert!(s.is_empty());
            }

            #[test]
            fn clear_empties_storage() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                s.clear();
                assert!(s.is_empty());
            }
        }
    };
}

/// Tests the [`AosStorage`] contract.
/// `$a` and `$b` are two distinct `$item` values for swap testing.
#[macro_export]
macro_rules! test_aos_storage {
    ($storage:ty, $item:ty, $a:expr, $b:expr) => {
        #[cfg(test)]
        mod aos_storage_tests {
            use super::*;

            #[test]
            fn slice_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.as_slice().len(), s.len());
            }

            #[test]
            fn slice_empty_when_storage_empty() {
                let s = <$storage>::new(10);
                assert!(s.as_slice().is_empty());
            }

            #[test]
            fn get_matches_slice() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                let from_get   = s.get(0) as *const $item;
                let from_slice = &s.as_slice()[0] as *const $item;
                assert_eq!(from_get, from_slice);
            }

            #[test]
            fn get_mut_matches_slice_mut() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                let from_get   = s.get_mut(0) as *mut $item;
                let from_slice = &mut s.as_slice_mut()[0] as *mut $item;
                assert_eq!(from_get, from_slice);
            }

            #[test]
            fn swap_exchanges_slice_positions() {
                let mut s = <$storage>::new(10);
                s.push($a);
                s.push($b);
                let val_0_before = unsafe { *(s.get(0) as *const $item as *const u8) };
                let val_1_before = unsafe { *(s.get(1) as *const $item as *const u8) };
                s.swap(0, 1);
                let val_0_after = unsafe { *(s.get(0) as *const $item as *const u8) };
                let val_1_after = unsafe { *(s.get(1) as *const $item as *const u8) };
                assert_eq!(val_0_after, val_1_before);
                assert_eq!(val_1_after, val_0_before);
            }

            #[test]
            fn iter_count_matches_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.iter().count(), s.len());
            }

            #[test]
            fn clear_empties_slice() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.clear();
                assert!(s.as_slice().is_empty());
            }
        }
    };
}
