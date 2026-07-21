use std::{alloc::Layout, cell::Cell, marker::PhantomData, ops::{Deref, DerefMut}, ptr::NonNull, slice};

  /// Represents any simulation storage engine.
/// Makes no assumptions about memory layout, threading, buffering, or the type of simulation.
pub trait Storage {
    /// Number of items or cells currently active in the storage.
    fn len(&self) -> usize;

    /// Maximum items or cells before reallocation or overflow.
    fn capacity(&self) -> usize;

    /// Returns `true` if the storage contains no elements.
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Called before the solver starts a step.
    /// Use for buffer swaps, memory mapping, or fence waits.
    fn pre_step(&mut self) {}

    /// Called after the solver completes a step.
    /// Use for GPU uploads, unmapping, or signalling.
    fn post_step(&mut self) {}

    /// Remove all items from storage without deallocating backing memory.
    fn clear(&mut self);
}


// =========================================================================
// CONTINUOUS GRID STORAGE TRAIT (Fluids, Fields, Cellular Automata)
// =========================================================================

/// A sub-trait for continuous spatial grids (Eulerian simulations).
/// Emphasizes static geometric topologies where elements are never pushed or popped.
pub trait GridStorage<const DIM: usize>: Storage {
    /// The structural data payload inside each discrete spatial cell.
    type CellData;

    /// The read-only view type for the whole grid topology.
    type GridView<'a> where Self: 'a;

    /// The mutable view type enabling arbitrary spatial writes or stencil reads.
    type GridViewMut<'a> where Self: 'a;

    /// Access the unmutated state of the entire grid.
    fn grid_view(&self) -> Self::GridView<'_>;

    /// Access the mutable workspace of the entire grid.
    fn grid_view_mut(&mut self) -> Self::GridViewMut<'_>;
    
    /// Look up a reference to a specific cell directly via its spatial coordinates.
    fn get_cell(&self, coord: [usize; DIM]) -> &Self::CellData;
}


// =========================================================================
// DISCRETE ELEMENT STORAGE TRAIT (Particles, Entities, Agents)
// =========================================================================

/// A sub-trait specifically for discrete, indexable simulation elements.
/// Erases structural details like Array-of-Structures (AoS) or Structure-of-Arrays (SoA).
pub trait ElementStorage: Storage {
    /// The canonical exchange format for an individual item (e.g. your `SoaLayout`).
    type Element;
    
    /// The read-only window into the storage layout (e.g., `&[T]` or `SoaView`).
    type View<'a> where Self: 'a;
    
    /// The mutable window into the storage layout (e.g., `&mut [T]` or `SoaViewMut`).
    type ViewMut<'a> where Self: 'a;

    /// Borrow the storage immutably for structural queries.
    fn view(&self) -> Self::View<'_>;

    /// Borrow the storage mutably for structural updates and queries.
    fn view_mut(&mut self) -> Self::ViewMut<'_>;

    /// Appends an element to the end of the collection.
    fn push(&mut self, element: Self::Element);

    /// Removes an element at the specified index by swapping it with the last element.
    /// This is an O(1) operation but does not preserve element ordering.
    fn swap_remove(&mut self, index: usize) -> Self::Element;

    /// Batch inserts elements from an iterator, allowing allocation pre-sizing checks.
    #[inline]
    fn extend_from_iter<I: IntoIterator<Item = Self::Element>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        for item in iterator {
            self.push(item);
        }
    }

    /// Efficiently removes multiple indices in bulk while tracking index movements.
    /// Expects indices to be unique. Scrambles the tail order of elements due to `swap_remove`.
    fn remove_indices(&mut self, indices: &mut [usize]) {
        if indices.is_empty() {
            return;
        }

        // Sort to process deletions from back-to-front, preventing upcoming shifts
        indices.sort_unstable();
        let mut current_len = self.len();

        for i in (0..indices.len()).rev() {
            let target_idx = indices[i];

            // Safely skip accidental duplicate inputs
            if i > 0 && target_idx == indices[i - 1] {
                continue;
            }

            if target_idx < current_len {
                let tail_idx = current_len - 1;
                let _moved_item = self.swap_remove(target_idx);

                // If the item swapped into the hole came from a tail index that is 
                // scheduled to be deleted later in this loop, we must track where it moved.
                if target_idx != tail_idx {
                    if let Ok(mut pos) = indices[..i].binary_search(&tail_idx) {
                        // Bubble the target_idx left to maintain sorted order
                        while pos > 0 && indices[pos - 1] > target_idx {
                            indices[pos] = indices[pos - 1];
                            pos -= 1;
                        }
                        indices[pos] = target_idx;
                    }
                }
                current_len -= 1;
            }
        }
    }
}




pub trait AosStorage: ElementStorage {
    fn as_slice(&self) -> &[Self::Element];
    fn as_slice_mut(&mut self) -> &mut [Self::Element];

    #[inline(always)]
    fn get(&self, i: usize) -> &Self::Element {
        &self.as_slice()[i]
    }

    #[inline(always)]
    fn get_mut(&mut self, i: usize) -> &mut Self::Element {
        &mut self.as_slice_mut()[i]
    }
}

// =========================================================================
// STRUCTURE-OF-ARRAYS (SoA) LOW-LEVEL CONFIGURATION MACHINERY
// =========================================================================
#[derive(Copy, Clone, Debug)]
pub struct SoaColumn {
    pub ptr: NonNull<u8>,
    pub cap: usize,
    pub element_layout: Layout,
}

pub unsafe trait SoaLayout: Sized {
    const LAYOUTS: &'static [Layout];
    unsafe fn push_cols(item: Self, cols: &mut [SoaColumn], index: usize);
    unsafe fn read_cols(cols: &[SoaColumn], index: usize) -> Self;
    unsafe fn drop_cols(cols: &[SoaColumn], index: usize);
}

pub trait SoaProperty<L: SoaLayout> {
    type Type;
    const INDEX: usize;
}

// =========================================================================
// HIGH-PERFORMANCE RUNTIME-TRACKED STORAGE VIEWS
// =========================================================================

pub struct SoaView<'a, L: SoaLayout> {
    cols: &'a [SoaColumn],
    len: usize,
    borrow_mask: Cell<u64>, 
    _marker: PhantomData<&'a L>,
}

impl<'a, L: SoaLayout> SoaView<'a, L> {
    #[inline(always)]
    pub fn new(cols: &'a [SoaColumn], len: usize) -> Self {
        Self { 
            cols, 
            len, 
            borrow_mask: Cell::new(0), 
            _marker: PhantomData 
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn slice<'b, P>(&'b self, _prop: P) -> ColumnSharedGuard<'b, L, P::Type>
    where
        P: SoaProperty<L>,
    {
        let index = P::INDEX;
        assert!(index < 32, "Column index exceeds tracking limit of 32.");
        
        let mut_mask = 1 << index;
        let current = self.borrow_mask.get();
        
        assert!(
            (current & mut_mask) == 0,
            "Exclusivity violation: Column {} is already mutably borrowed!",
            index
        );
        
        let read_offset = 32 + index;
        self.borrow_mask.set(current + (1 << read_offset));

        let slice = if std::mem::size_of::<P::Type>() == 0 || self.len == 0 {
            &[]
        } else {
            // Elimination of redundant bounds check via unchecked layout optimizations
            let col = unsafe { self.cols.get_unchecked(index) };
            unsafe { std::slice::from_raw_parts(col.ptr.as_ptr() as *const P::Type, self.len) }
        };

        ColumnSharedGuard {
            slice,
            index,
            borrow_mask: &self.borrow_mask,
            _marker: PhantomData,
        }
    }

    // FIXED: Combined untyped helper to accept a clean generic target directly
    // This removes the intermediate temporary guard allocation and aliasing warnings
   #[inline(always)]
pub fn slice_mut_typed<'b, P>(&'b self, _prop: P) -> ColumnMutGuard<'b, L, P::Type>
where
    P: SoaProperty<L>,
{
    let index = P::INDEX;
    assert!(index < 32, "Column index exceeds tracking limit of 32.");
    
    let bit = 1 << index;
    let current = self.borrow_mask.get();
    
    // Check both unique write block AND read block mirrors
    assert!(
        (current & bit) == 0 && (current & (1 << (32 + index))) == 0,
        "Exclusivity violation: Column {} is already borrowed!",
        index
    );
    
    // Set write borrow lock flag
    self.borrow_mask.set(current | bit);

    let slice = if std::mem::size_of::<P::Type>() == 0 || self.len == 0 {
        &mut []
    } else {
        // Fetch the column metadata directly
        let col = unsafe { self.cols.get_unchecked(index) };
        
        // CRITICAL FIX: The pointer MUST point strictly to the base address.
        // DO NOT let the AI add any byte offsets here!
        unsafe { std::slice::from_raw_parts_mut(col.ptr.as_ptr() as *mut P::Type, self.len) }
    };

    ColumnMutGuard {
        slice,
        index,
        borrow_mask: &self.borrow_mask,
        _marker: PhantomData,
    }
}

    
}

// =========================================================================
// CLEAN UNIFIED LOAN GUARDS
// =========================================================================

pub struct ColumnSharedGuard<'b, L: SoaLayout, T> {
    slice: &'b [T],
    index: usize,
    borrow_mask: &'b Cell<u64>,
    _marker: PhantomData<L>,
}

impl<'b, L: SoaLayout, T> Deref for ColumnSharedGuard<'b, L, T> {
    type Target = [T];
    #[inline(always)] fn deref(&self) -> &Self::Target { self.slice }
}

impl<'b, L: SoaLayout, T> Drop for ColumnSharedGuard<'b, L, T> {
    #[inline(always)]
    fn drop(&mut self) {
        let read_offset = 32 + self.index;
        let current = self.borrow_mask.get();
        self.borrow_mask.set(current - (1 << read_offset));
    }
}

pub struct ColumnMutGuard<'b, L: SoaLayout, T> {
    slice: &'b mut [T],
    index: usize,
    borrow_mask: &'b Cell<u64>,
    _marker: PhantomData<L>,
}

impl<'b, L: SoaLayout, T> Deref for ColumnMutGuard<'b, L, T> {
    type Target = [T];
    #[inline(always)] fn deref(&self) -> &Self::Target { self.slice }
}

impl<'b, L: SoaLayout, T> DerefMut for ColumnMutGuard<'b, L, T> {
    #[inline(always)] fn deref_mut(&mut self) -> &mut Self::Target { self.slice }
}

impl<'b, L: SoaLayout, T> Drop for ColumnMutGuard<'b, L, T> {
    #[inline(always)]
    fn drop(&mut self) {
        let mask = !(1 << self.index);
        let current = self.borrow_mask.get();
        self.borrow_mask.set(current & mask);
    }
}

// /// A specialized storage backend managing discrete, uniform entities.
// /// Fully compatible with both AoS (Array-of-Structs) and SoA (Struct-of-Arrays).
// pub trait ItemStorage: Storage {
//     type Item: Sized;

//     /// Creates new storage with an initial capacity hint.
//     fn with_capacity(capacity: usize) -> Self;

//     /// Appends a single item to the backend. 
//     fn push(&mut self, item: Self::Item);

//     /// Removes an item at the given index by swapping it with the last element.
//     fn swap_remove(&mut self, index: usize) -> Self::Item;

//     /// OPTIONAL EXPLICIT OPTIMIZATION: Batch appends from an iterator.
//     /// SoA backends can override this to optimize allocations and streaming steps.
//     fn extend_from_iter<I: IntoIterator<Item = Self::Item>>(&mut self, iter: I) {
//         for item in iter {
//             self.push(item);
//         }
//     }

//     /// HIGH-PERFORMANCE BATCH REMOVAL
//     /// Works flawlessly on both AoS and SoA layouts. Safely accounts for 
//     /// cascading element shifts caused by swap-removal mechanics.
//     fn remove_indices(&mut self, indices: &mut [usize]) {
//         if indices.is_empty() { return; }

//         // 1. Sort descending to process highest indices first
//         indices.sort_unstable_by(|a, b| b.cmp(a));
        
//         let mut current_len = self.len();
//         let mut last_seen = None;
        
//         for &i in indices.iter() {
//             // Filter duplicates safely
//             if Some(i) == last_seen { continue; }
//             last_seen = Some(i);
            
//             if i < current_len {
//                 self.swap_remove(i);
//                 // CRITICAL FIX: The element at (current_len - 1) just moved to index `i`.
//                 // We must shrink our virtual boundary so we never read past it.
//                 current_len -= 1;
//             }
//         }
//     }
// }


// /// Specialized trait for Array-of-Structs (AoS) memory layouts.
// /// Fully compatible with ItemStorage via associated type mapping.
// pub trait AosStorage: ItemStorage {
//     fn as_slice(&self)         -> &[Self::Item];
//     fn as_slice_mut(&mut self) -> &mut [Self::Item];

//     #[inline(always)]
//     fn get(&self, i: usize)         -> &Self::Item     { &self.as_slice()[i] }
//     #[inline(always)]
//     fn get_mut(&mut self, i: usize) -> &mut Self::Item { &mut self.as_slice_mut()[i] }

//     #[inline(always)]
//     fn iter(&self)         -> std::slice::Iter<'_, Self::Item>    { self.as_slice().iter() }
//     #[inline(always)]
//     fn iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Item> { self.as_slice_mut().iter_mut() }

//     #[inline(always)]
//     fn swap(&mut self, a: usize, b: usize) { self.as_slice_mut().swap(a, b); }
// }

// pub trait SoaProperty {
//     fn column_index(&self) -> usize;
// }

// #[derive(Copy, Clone, Debug)]
// pub struct RawColumn {
//     pub ptr: *mut u8,
//     pub cap: usize,
//     pub element_layout: Layout,
// }

// pub unsafe trait SoaLayout: Sized {
//     const LAYOUTS: &'static [Layout];
    
//     /// Extracts fields from `self` and writes them into parallel memory columns.
//     unsafe fn push_cols(&self, cols: &mut [RawColumn], index: usize);
    
//     /// Reads individual column fields out of raw memory and forms a cohesive Item struct.
//     unsafe fn read_cols(cols: &[RawColumn], index: usize) -> Self;
    
//     /// High-performance memory-shifting operation executing layout-agnostic swap-removals.
//     #[inline]
//     unsafe fn swap_remove_cols(cols: &mut [RawColumn], index: usize, current_len: usize) {
//         if current_len == 0 { return; }
//         let tail_index = current_len - 1;
//         if index == tail_index { return; }
        
//         for col in cols.iter_mut() {
//             let size = col.element_layout.size();
//             unsafe {
//                 ptr::copy_nonoverlapping(
//                     col.ptr.add(tail_index * size),
//                     col.ptr.add(index * size),
//                     size,
//                 );
//             }
//         }
//     }

//     #[inline(always)]
//     fn slice_from_cols<T>(cols: &[RawColumn], col_index: usize, len: usize) -> &[T] {
//         if len == 0 || cols.is_empty() { return &[]; }
//         let ptr = cols[col_index].ptr.cast::<T>();
//         unsafe { slice::from_raw_parts(ptr, len) }
//     }

//     #[inline(always)]
//     fn slice_from_cols_mut<T>(cols: &mut [RawColumn], col_index: usize, len: usize) -> &mut [T] {
//         if len == 0 || cols.is_empty() { return &mut []; }
//         let ptr = cols[col_index].ptr.cast::<T>();
//         unsafe { slice::from_raw_parts_mut(ptr, len) }
//     }
// }

// /// Specialized trait for Structure-of-Arrays (SoA) memory layouts.
// /// We link ItemStorage's Item directly to the layout definition.
// pub trait SoaStorage: ItemStorage<Item = Self::Layout> {
//     type Layout: SoaLayout;
    
//     fn columns(&self) -> &[RawColumn];
//     fn columns_mut(&mut self) -> &mut [RawColumn];
    
//     /// Unsafely modifies the tracked length pointer of the container.
//     /// This is required because batch pushes bypass single-item increment loops.
//     /// 
//     /// # Safety
//     /// The caller must ensure that columns have sufficient capacity and 
//     /// valid data initialized up to `new_len`.
//     unsafe fn set_len(&mut self, new_len: usize);

//     /// Provides type-safe immutable access to an individual parallel data column.
//     #[inline(always)]
//     fn get_column<T>(&self, property: impl SoaProperty) -> &[T] {
//         let idx = property.column_index();
//         Self::Layout::slice_from_cols(self.columns(), idx, self.len())
//     }

//     /// Provides type-safe mutable access to an individual parallel data column.
//     #[inline(always)]
//     fn get_column_mut<T>(&mut self, property: impl SoaProperty) -> &mut [T] {
//         let idx = property.column_index();
//         let len = self.len();
//         Self::Layout::slice_from_cols_mut(self.columns_mut(), idx, len)
//     }
// }


// // =========================================================================
// // 2. PARADIGM-SPECIFIC EXTENSION TRAITS (AoS vs SoA)
// // =========================================================================

// /// Specialized trait for Array-of-Structs (AoS) memory layouts.
// /// Provides direct access to the underlying contiguous struct slice.
// pub trait AosStorage: ItemStorage {
//     fn as_slice(&self)         -> &[Self::Item];
//     fn as_slice_mut(&mut self) -> &mut [Self::Item];

//     fn get(&self, i: usize)         -> &Self::Item     { &self.as_slice()[i] }
//     fn get_mut(&mut self, i: usize) -> &mut Self::Item { &mut self.as_slice_mut()[i] }

//     fn iter(&self)         -> std::slice::Iter<'_, Self::Item>    { self.as_slice().iter() }
//     fn iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Item> { self.as_slice_mut().iter_mut() }

//     fn swap(&mut self, a: usize, b: usize) { self.as_slice_mut().swap(a, b); }
// }


// pub trait SoaProperty {
//     /// Maps an enum-based property variant directly to its layout column index.
//     fn column_index(&self) -> usize;
// }

// /// Specialized trait for Structure-of-Arrays (SoA) memory layouts.
// pub trait SoaStorage: ItemStorage<Item = Self::Layout> {
//     type Layout: SoaLayout;
//     fn columns(&self) -> &[RawColumn];
//     fn columns_mut(&mut self) -> &mut [RawColumn];
//     fn set_len(&mut self, new_len: usize);
// }

// #[derive(Copy, Clone, Debug)]
// pub struct RawColumn {
//     pub ptr: *mut u8,
//     pub cap: usize,
//     pub element_layout: Layout,
// }

// pub unsafe trait SoaLayout: Sized {
//     const LAYOUTS: &'static [Layout];
    
//     unsafe fn push_cols(&self, cols: &mut [RawColumn], index: usize);
//     unsafe fn read_cols(cols: &[RawColumn], index: usize) -> Self;
    
//     #[inline]
//     unsafe fn swap_remove_cols(cols: &mut [RawColumn], index: usize, current_len: usize) {
//         let tail_index = current_len - 1;
//         if index == tail_index { return; }
        
//         unsafe {
//             for col in cols.iter_mut() {
//                 let size = col.element_layout.size();
//                 ptr::copy_nonoverlapping(
//                     col.ptr.cast::<u8>().add(tail_index * size),
//                     col.ptr.cast::<u8>().add(index * size),
//                     size,
//                 );
//             }
//         }
//     }

//     #[inline(always)]
//     fn slice_from_cols<T>(cols: &[RawColumn], col_index: usize, len: usize) -> &[T] {
//         if len == 0 { return &[]; }
//         let ptr = cols[col_index].ptr.cast::<T>();
//         unsafe { slice::from_raw_parts(ptr, len) }
//     }

//     #[inline(always)]
//     fn slice_from_cols_mut<T>(cols: &mut [RawColumn], col_index: usize, len: usize) -> &mut [T] {
//         if len == 0 { return &mut []; }
//         let ptr = cols[col_index].ptr.cast::<T>();
//         unsafe { slice::from_raw_parts_mut(ptr, len) }
//     }
// }
 
// /// An abstraction representing a layout lane. Right now it owns a Vec.
// /// Later, it will simply hold a pointer offset into your monolithic buffer.
// pub struct Column {
//     bytes: Vec<u8>,
//     element_layout: std::alloc::Layout,
// }

// pub struct SoaVecStorage<Entity: SoaLayout> {
//     columns: Vec<Column>,
//     raw_columns: Vec<RawColumn>, // Mirror for low-level traits
//     len: usize,
//     _marker: PhantomData<Entity>,
// }

// impl<Entity: SoaLayout> SoaVecStorage<Entity> {
//     pub fn new(capacity: usize) -> Self {
//         let mut columns = Vec::new();
//         let mut raw_columns = Vec::new();

//         for &layout in Entity::LAYOUTS {
//             columns.push(Column { bytes: Vec::new(), element_layout: layout });
//             raw_columns.push(RawColumn { ptr: std::ptr::null_mut(), cap: 0, element_layout: layout });
//         }

//         let mut storage = Self { columns, raw_columns, len: 0, _marker: PhantomData };
//         if capacity > 0 { storage.grow_to(capacity); }
//         storage
//     }

//     fn grow_to(&mut self, new_capacity: usize) {
//         for col in &mut self.columns {
//             let size = col.element_layout.size();
//             if size > 0 {
//                 let target_capacity = new_capacity * size;
//                 if target_capacity > col.bytes.capacity() {
//                     let additional = target_capacity - col.bytes.len();
//                     col.bytes.reserve(additional);
//                 }
//             }
//         }
//         self.sync_raw_descriptors();
//     }

//     fn sync_raw_descriptors(&mut self) {
//         for (i, col) in self.columns.iter_mut().enumerate() {
//             let size = col.element_layout.size();
//             self.raw_columns[i] = RawColumn {
//                 ptr: col.bytes.as_mut_ptr(),
//                 cap: if size == 0 { 0 } else { col.bytes.capacity() / size },
//                 element_layout: col.element_layout,
//             };
//         }
//     }

//     // ======================================================================
//     // 🟢 THE SACRED INTERFACE: Your solvers ONLY ever call these methods
//     // ======================================================================
//     #[inline(always)]
//     pub fn slice<T, P: SoaProperty>(&self, prop: P) -> &[T] {
//         Entity::slice_from_cols(&self.raw_columns, prop.column_index(), self.len)
//     }

//     #[inline(always)]
//     pub fn slice_mut<T, P: SoaProperty>(&mut self, prop: P) -> &mut [T] {
//         Entity::slice_from_cols_mut(&mut self.raw_columns, prop.column_index(), self.len)
//     }
// }

// pub trait SoaLayout: Sized {
//     /// Byte stride of each column — one entry per field.
//     const STRIDES: &'static [usize];

//     /// Push all fields into their respective byte columns.
//     fn push_cols(&self, cols: &mut [Vec<u8>]);

//     /// Reconstruct `Self` from byte columns at `index`.
//     fn read_cols(cols: &[Vec<u8>], index: usize) -> Self;

//     /// Swap-remove at `index` — keeps all columns in sync.
//     fn swap_remove_cols(cols: &mut [Vec<u8>], strides: &[usize], index: usize);
// }

// ---------------------------------------------------------------------------
// Test macros — any concrete impl invokes these to verify the contract.
// --------------------------------------------------------------------------

// /// Tests the [`CpuStorage`] contract.
// #[macro_export]
// macro_rules! test_cpu_storage {
//     ($storage:ty) => {
//         #[cfg(test)]
//         mod cpu_storage_tests {
//             use super::*;
//             use $crate::sim::storage::{CpuStorage, Storage};

//             #[test]
//             fn new_has_correct_capacity() {
//                 let s = <$storage>::new(10);
//                 assert!(s.capacity() >= 10);
//             }

//             #[test]
//             fn new_is_empty() {
//                 let s = <$storage>::new(10);
//                 assert_eq!(s.len(), 0);
//                 assert!(s.is_empty());
//             }

//             #[test]
//             fn clear_empties_storage() {
//                 let mut s = <$storage>::new(10);
//                 // This test needs a way to add items.
//                 // We assume the type being tested will also implement AosCpuStorage or have a similar method.
//                 // This highlights the tight coupling even in tests.
//                 // For now, we'll assume it's an AosCpuStorage for the test to compile.
//                 // s.push(Default::default());
//                 s.clear();
//                 assert!(s.is_empty());
//             }
//         }
//     };
// }

// /// Tests the [`AosCpuStorage`] contract.
// /// `$item` must implement `Default` and `PartialEq`.
// /// `$a` and `$b` are two distinct `$item` values for swap testing.
// #[macro_export]
// macro_rules! test_aos_cpu_storage {
//     ($storage:ty, $item:ty, $a:expr, $b:expr) => {
//         #[cfg(test)]
//         mod aos_cpu_storage_tests {
//             use super::*;
//             use $crate::sim::storage::{AosCpuStorage, CpuStorage, Storage};

//             #[test]
//             fn push_increases_len() {
//                 let mut s = <$storage>::new(10);
//                 s.push(<$item>::default());
//                 assert_eq!(s.len(), 1);
//                 s.push(<$item>::default());
//                 assert_eq!(s.len(), 2);
//             }

//             #[test]
//             fn swap_remove_decreases_len() {
//                 let mut s = <$storage>::new(10);
//                 s.push(<$item>::default());
//                 s.push(<$item>::default());
//                 s.swap_remove(0);
//                 assert_eq!(s.len(), 1);
//             }

//             #[test]
//             fn swap_remove_returns_correct_item() {
//                 let mut s = <$storage>::new(10);
//                 s.push($a);
//                 s.push($b);
//                 let removed = s.swap_remove(0);
//                 // The swapped item is now at index 0
//                 assert_eq!(removed, $a);
//                 assert_eq!(s.get(0), &$b);
//             }

//             #[test]
//             fn swap_remove_last_empties() {
//                 let mut s = <$storage>::new(10);
//                 s.push(<$item>::default());
//                 s.swap_remove(0);
//                 assert!(s.is_empty());
//             }

//             #[test]
//             fn slice_len_matches_storage_len() {
//                 let mut s = <$storage>::new(10);
//                 s.push(<$item>::default());
//                 assert_eq!(s.as_slice().len(), s.len());
//             }

//             // ... other aos tests from your file ...
//         }
//     };
// }

// /// Tests the [`SoaLayout`] contract for a given item type.
// /// This verifies that `push_cols` and `read_cols` are inverse operations.
// /// `$item_type` must implement `SoaLayout`, `PartialEq`, and `Debug`.
// /// `$a` and `$b` must be two distinct instances of `$item_type`.
// #[macro_export]
// macro_rules! test_soa_layout {
//     ($item_type:ty, $a:expr, $b:expr) => {
//         #[cfg(test)]
//         mod soa_layout_tests {
//             use super::*;
//             use $crate::sim::storage::SoaLayout;

//             fn create_cols() -> Vec<Vec<u8>> {
//                 (0..<$item_type>::STRIDES.len()).map(|_| Vec::new()).collect()
//             }

//             #[test]
//             fn push_read_round_trip() {
//                 let mut cols = create_cols();
//                 let original_item = $a;

//                 original_item.push_cols(&mut cols);
//                 let round_tripped_item = <$item_type>::read_cols(&cols, 0);

//                 assert_eq!(original_item, round_tripped_item, "Item should be identical after a push/read round trip");
//             }

//             #[test]
//             fn swap_remove_cols_removes_correct_item() {
//                 let mut cols = create_cols();
//                 let item_a = $a;
//                 let item_b = $b;

//                 item_a.push_cols(&mut cols);
//                 item_b.push_cols(&mut cols); // cols now contain [a, b]

//                 // Swap-remove item at index 0
//                 <$item_type>::swap_remove_cols(&mut cols, <$item_type>::STRIDES, 0);

//                 // The last item (b) should now be at index 0
//                 let remaining_item = <$item_type>::read_cols(&cols, 0);
//                 assert_eq!(remaining_item, item_b, "After swap_remove(0), the last item should be at index 0");

//                 // Verify length of columns is correct (assuming one push per field per item)
//                 for (i, col) in cols.iter().enumerate() {
//                     assert_eq!(col.len(), <$item_type>::STRIDES[i], "Column length should be for one item after swap_remove");
//                 }
//             }
//         }
//     };
// }