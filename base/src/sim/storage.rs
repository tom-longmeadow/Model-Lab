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

/// [`SoaStorage`] with `pos`, `vel`, `acc` columns.
/// Required by [`NewtonianVerlet`] and [`Leapfrog`] solvers.
///
/// Column layout: blocked — all x values, then all y values etc.
/// i.e. `pos = [x0, x1, ..., y0, y1, ...]` — not interleaved.
///
/// Only combined accessors are exposed. Individual `_mut` accessors cannot
/// be called simultaneously under Rust's borrow rules and are therefore not
/// part of the API. Implementations prove disjointness via direct field
/// access or `split_at_mut`.
pub trait SoaNewtonianStorage: SoaStorage {
    fn pos(&self) -> &[f64];
    fn vel(&self) -> &[f64];
    fn acc(&self) -> &[f64];

    /// Returns `(pos_mut, vel_mut, acc)` — disjoint, used by [`NewtonianVerlet`].
    fn pos_vel_mut_acc(&mut self) -> (&mut [f64], &mut [f64], &[f64]);

    /// Returns `(vel_mut, acc)` — disjoint, used by [`Leapfrog`] half-kick.
    fn vel_mut_acc(&mut self) -> (&mut [f64], &[f64]);

    /// Returns `(pos_mut, vel)` — disjoint, used by [`Leapfrog`] drift.
    fn pos_mut_vel(&mut self) -> (&mut [f64], &[f64]);
}

/// [`SoaStorage`] with `pos`, `pos_old`, `acc` columns.
/// Required by the [`Verlet`] solver.
///
/// Column layout: blocked — all x values, then all y values etc.
pub trait SoaVerletStorage: SoaStorage {
    fn pos(&self)     -> &[f64];
    fn pos_old(&self) -> &[f64];
    fn acc(&self)     -> &[f64];

    /// Returns `(pos_mut, pos_old_mut, acc)` — disjoint, used by [`Verlet`].
    fn pos_pos_old_mut_acc(&mut self) -> (&mut [f64], &mut [f64], &[f64]);
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

/// Tests the [`SoaNewtonianStorage`] contract.
/// `$a` and `$b` are two distinct `$item` values.
#[macro_export]
macro_rules! test_soa_newtonian_storage {
    ($storage:ty, $item:ty, $a:expr, $b:expr) => {
        #[cfg(test)]
        mod soa_newtonian_storage_tests {
            use super::*;

            #[test]
            fn pos_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.pos().len(), s.len());
            }

            #[test]
            fn vel_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.vel().len(), s.len());
            }

            #[test]
            fn acc_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.acc().len(), s.len());
            }

            #[test]
            fn pos_vel_mut_acc_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                let (pos, vel, acc) = s.pos_vel_mut_acc();
                assert_eq!(pos.len(), vel.len());
                assert_eq!(vel.len(), acc.len());
            }

            #[test]
            fn vel_mut_acc_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                let (vel, acc) = s.vel_mut_acc();
                assert_eq!(vel.len(), acc.len());
            }

            #[test]
            fn pos_mut_vel_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                let (pos, vel) = s.pos_mut_vel();
                assert_eq!(pos.len(), vel.len());
            }

            #[test]
            fn clear_empties_columns() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.clear();
                assert!(s.pos().is_empty());
                assert!(s.vel().is_empty());
                assert!(s.acc().is_empty());
            }
        }
    };
}

/// Tests the [`SoaVerletStorage`] contract.
/// `$a` and `$b` are two distinct `$item` values.
#[macro_export]
macro_rules! test_soa_verlet_storage {
    ($storage:ty, $item:ty, $a:expr, $b:expr) => {
        #[cfg(test)]
        mod soa_verlet_storage_tests {
            use super::*;

            #[test]
            fn pos_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.pos().len(), s.len());
            }

            #[test]
            fn pos_old_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.pos_old().len(), s.len());
            }

            #[test]
            fn acc_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                assert_eq!(s.acc().len(), s.len());
            }

            #[test]
            fn pos_pos_old_mut_acc_lengths_match() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.push(<$item>::default());
                let (pos, pos_old, acc) = s.pos_pos_old_mut_acc();
                assert_eq!(pos.len(), pos_old.len());
                assert_eq!(pos_old.len(), acc.len());
            }

            #[test]
            fn clear_empties_columns() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.clear();
                assert!(s.pos().is_empty());
                assert!(s.pos_old().is_empty());
                assert!(s.acc().is_empty());
            }
        }
    };
}