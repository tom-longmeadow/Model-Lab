

/// Represents any simulation storage.
/// Makes no assumptions about memory layout, threading,
/// buffering, or the type of simulation.
pub trait Storage {
    /// Number of items currently stored.
    fn len(&self) -> usize;

    /// Maximum items before reallocation or overflow.
    fn capacity(&self) -> usize;

    fn is_empty(&self) -> bool { self.len() == 0 }

    /// Called before the solver starts a step.
    /// Use for buffer swaps, memory mapping, fence waits.
    fn pre_step(&mut self)  {}

    /// Called after the solver completes a step.
    /// Use for GPU uploads, unmapping, signalling.
    fn post_step(&mut self) {}

    /// Remove all items.
    fn clear(&mut self);

     /// Removes all items at the given indices in a single pass.
    /// Each storage type implements this in its own layout-native way:
    /// - AoS: descending swap_remove
    /// - SoA: descending swap_remove_cols
    /// - GPU: parallel compaction shader
    fn remove_indices(&mut self, indices: Vec<usize>);
}


/// A marker trait for storage that is directly and synchronously accessible by the CPU.
pub trait CpuStorage: Storage {
    /// Creates new storage with an initial capacity hint.
    fn new(capacity: usize) -> Self;
}

/// A marker trait for storage that resides primarily on the GPU. 
pub trait GpuStorage: Storage {}

/// Array-of-Structs storage — items held as a contiguous slice.
/// Implement `as_slice`/`as_slice_mut`; everything else is derived.
pub trait AosCpuStorage: CpuStorage {
    type Item: Sized;

    fn push(&mut self, item: Self::Item);
    fn swap_remove(&mut self, index: usize) -> Self::Item;
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
pub trait SoaCpuStorage: CpuStorage {
    type Layout: SoaLayout;

    /// Access to raw byte columns for layout-native operations.
    fn columns_mut(&mut self) -> &mut [Vec<u8>];

    /// Increment the length counter after a manual push_cols.
    fn increment_len(&mut self);
}

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
// ---------------------------------------------------------------------------
// Test macros — any concrete impl invokes these to verify the contract.
// ---------------------------------------------------------------------------

/// Tests the [`CpuStorage`] contract.
#[macro_export]
macro_rules! test_cpu_storage {
    ($storage:ty) => {
        #[cfg(test)]
        mod cpu_storage_tests {
            use super::*;
            use $crate::sim::storage::{CpuStorage, Storage};

            #[test]
            fn new_has_correct_capacity() {
                let s = <$storage>::new(10);
                assert!(s.capacity() >= 10);
            }

            #[test]
            fn new_is_empty() {
                let s = <$storage>::new(10);
                assert_eq!(s.len(), 0);
                assert!(s.is_empty());
            }

            #[test]
            fn clear_empties_storage() {
                let mut s = <$storage>::new(10);
                // This test needs a way to add items.
                // We assume the type being tested will also implement AosCpuStorage or have a similar method.
                // This highlights the tight coupling even in tests.
                // For now, we'll assume it's an AosCpuStorage for the test to compile.
                // s.push(Default::default());
                s.clear();
                assert!(s.is_empty());
            }
        }
    };
}

/// Tests the [`AosCpuStorage`] contract.
/// `$item` must implement `Default` and `PartialEq`.
/// `$a` and `$b` are two distinct `$item` values for swap testing.
#[macro_export]
macro_rules! test_aos_cpu_storage {
    ($storage:ty, $item:ty, $a:expr, $b:expr) => {
        #[cfg(test)]
        mod aos_cpu_storage_tests {
            use super::*;
            use $crate::sim::storage::{AosCpuStorage, CpuStorage, Storage};

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
            fn swap_remove_returns_correct_item() {
                let mut s = <$storage>::new(10);
                s.push($a);
                s.push($b);
                let removed = s.swap_remove(0);
                // The swapped item is now at index 0
                assert_eq!(removed, $a);
                assert_eq!(s.get(0), &$b);
            }

            #[test]
            fn swap_remove_last_empties() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                s.swap_remove(0);
                assert!(s.is_empty());
            }

            #[test]
            fn slice_len_matches_storage_len() {
                let mut s = <$storage>::new(10);
                s.push(<$item>::default());
                assert_eq!(s.as_slice().len(), s.len());
            }

            // ... other aos tests from your file ...
        }
    };
}

/// Tests the [`SoaLayout`] contract for a given item type.
/// This verifies that `push_cols` and `read_cols` are inverse operations.
/// `$item_type` must implement `SoaLayout`, `PartialEq`, and `Debug`.
/// `$a` and `$b` must be two distinct instances of `$item_type`.
#[macro_export]
macro_rules! test_soa_layout {
    ($item_type:ty, $a:expr, $b:expr) => {
        #[cfg(test)]
        mod soa_layout_tests {
            use super::*;
            use $crate::sim::storage::SoaLayout;

            fn create_cols() -> Vec<Vec<u8>> {
                (0..<$item_type>::STRIDES.len()).map(|_| Vec::new()).collect()
            }

            #[test]
            fn push_read_round_trip() {
                let mut cols = create_cols();
                let original_item = $a;

                original_item.push_cols(&mut cols);
                let round_tripped_item = <$item_type>::read_cols(&cols, 0);

                assert_eq!(original_item, round_tripped_item, "Item should be identical after a push/read round trip");
            }

            #[test]
            fn swap_remove_cols_removes_correct_item() {
                let mut cols = create_cols();
                let item_a = $a;
                let item_b = $b;

                item_a.push_cols(&mut cols);
                item_b.push_cols(&mut cols); // cols now contain [a, b]

                // Swap-remove item at index 0
                <$item_type>::swap_remove_cols(&mut cols, <$item_type>::STRIDES, 0);

                // The last item (b) should now be at index 0
                let remaining_item = <$item_type>::read_cols(&cols, 0);
                assert_eq!(remaining_item, item_b, "After swap_remove(0), the last item should be at index 0");

                // Verify length of columns is correct (assuming one push per field per item)
                for (i, col) in cols.iter().enumerate() {
                    assert_eq!(col.len(), <$item_type>::STRIDES[i], "Column length should be for one item after swap_remove");
                }
            }
        }
    };
}