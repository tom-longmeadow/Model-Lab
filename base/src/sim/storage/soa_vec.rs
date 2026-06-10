use crate::sim::storage::{CpuStorage, SoaCpuStorage, SoaLayout, Storage};
use std::marker::PhantomData;

/// Generic CPU-side Struct-of-Arrays storage backed by `Vec<u8>` byte columns.
/// `T` declares its own column layout via [`SoaLayout`].
/// Storage manages raw byte columns — knows nothing about field semantics.
/// Typed safe column access lives in the physics sub-traits
/// ([`SoaNewtonianStorage`], [`SoaVerletStorage`]).
pub struct SoaVecStorage<T: SoaLayout> {
    /// One `Vec<u8>` per field — length is `len * stride` for that column.
    pub(crate) columns: Vec<Vec<u8>>,
    len: usize,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T: SoaLayout> SoaVecStorage<T> {
     

    /// Typed immutable slice over one byte column.
    ///
    /// # Safety
    /// `C` must match the actual element type for `col`.
    /// Only called from physics sub-trait impls which know the correct type.
    pub(crate) unsafe fn col_raw<C>(&self, col: usize) -> &[C] {
        let bytes = &self.columns[col];
        let stride = T::STRIDES[col];
        unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const C, bytes.len() / stride) }
    }

    /// Typed mutable slice over one byte column.
    ///
    /// # Safety
    /// `C` must match the actual element type for `col`.
    /// Only called from physics sub-trait impls which know the correct type.
    pub(crate) unsafe fn col_raw_mut<C>(&mut self, col: usize) -> &mut [C] {
        let stride = T::STRIDES[col];
        let len = self.columns[col].len() / stride;
        let bytes = &mut self.columns[col];
        unsafe { std::slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut C, len) }
    }

    /// Borrow columns `a` and `b` mutably and column `c` immutably in one call.
    /// All three indices must be distinct.
    ///
    /// # Safety
    /// `C` must match the actual element type for all three columns.
    /// `a`, `b`, `c` must be distinct valid column indices.
    pub(crate) unsafe fn col3_mut_mut_ref<C: Copy>(
        &mut self,
        a: usize, // mutable
        b: usize, // mutable
        c: usize, // immutable
    ) -> (&mut [C], &mut [C], &[C]) {
        debug_assert!(a != b && a != c && b != c, "col3_mut_mut_ref: indices must be distinct");
        let ptr = self.columns.as_mut_ptr();
        // SAFETY: a, b, c are distinct so the three borrows don't alias.
        let col_a = unsafe { &mut *ptr.add(a) };
        let col_b = unsafe { &mut *ptr.add(b) };
        let col_c = unsafe { &*ptr.add(c) };
        let len_a = col_a.len() / T::STRIDES[a];
        let len_b = col_b.len() / T::STRIDES[b];
        let len_c = col_c.len() / T::STRIDES[c];
        unsafe {
            (
                std::slice::from_raw_parts_mut(col_a.as_mut_ptr() as *mut C, len_a),
                std::slice::from_raw_parts_mut(col_b.as_mut_ptr() as *mut C, len_b),
                std::slice::from_raw_parts(col_c.as_ptr() as *const C, len_c),
            )
        }
    }
}

impl<T: SoaLayout> Storage for SoaVecStorage<T> {
    fn len(&self) -> usize {
        self.len
    }
    fn capacity(&self) -> usize {
        self.capacity
    }
    fn clear(&mut self) {
        self.columns.iter_mut().for_each(|c| c.clear());
        self.len = 0;
    }

    fn remove_indices(&mut self, mut indices: Vec<usize>) {
        indices.sort_unstable_by(|a, b| b.cmp(a));
        indices.dedup();
        for i in indices {
            T::swap_remove_cols(&mut self.columns, T::STRIDES, i);
            self.len -= 1;
        }
    }
}

impl<T: SoaLayout> CpuStorage for SoaVecStorage<T> {
    fn new(capacity: usize) -> Self {
        let columns = T::STRIDES
            .iter()
            .map(|s| Vec::with_capacity(capacity * s))
            .collect();
        Self {
            columns,
            len: 0,
            capacity,
            _marker: PhantomData,
        }
    }
}

impl<T: SoaLayout> SoaCpuStorage for SoaVecStorage<T> {
    type Layout = T;

    fn columns_mut(&mut self) -> &mut [Vec<u8>] {
        &mut self.columns
    }

    fn increment_len(&mut self) {
        self.len += 1;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::storage::SoaLayout;

    #[derive(Default, PartialEq, Debug, Clone, Copy)]
    pub struct MockEntity {
        pub d64: f64,
        pub c8: u8,
    }

    impl SoaLayout for MockEntity {
        const STRIDES: &'static [usize] = &[std::mem::size_of::<f64>(), std::mem::size_of::<u8>()];

        fn push_cols(&self, cols: &mut [Vec<u8>]) {
            cols[0].extend_from_slice(&self.d64.to_ne_bytes());
            cols[1].extend_from_slice(&self.c8.to_ne_bytes());
        }

        fn read_cols(cols: &[Vec<u8>], index: usize) -> Self {
            let d64_start = index * Self::STRIDES[0];
            let d64_end = d64_start + Self::STRIDES[0];
            let d64 = f64::from_ne_bytes(cols[0][d64_start..d64_end].try_into().unwrap());

            let c8_start = index * Self::STRIDES[1];
            let c8 = cols[1][c8_start];
            Self { d64, c8 }
        }

        fn swap_remove_cols(cols: &mut [Vec<u8>], strides: &[usize], index: usize) {
            for (col, &stride) in cols.iter_mut().zip(strides.iter()) {
                let last_start = col.len() - stride;
                if index * stride != last_start {
                    let (target, source) = col.split_at_mut(last_start);
                    target[index * stride..index * stride + stride].copy_from_slice(source);
                }
                col.truncate(last_start);
            }
        }
    }

    // --- contract macro tests ---
    crate::test_cpu_storage!(SoaVecStorage<MockEntity>);
    crate::test_soa_layout!(
        MockEntity,
        MockEntity { d64: 1.0, c8: 1 },
        MockEntity { d64: 2.0, c8: 2 }
    );

    // --- remove_indices tests ---
    #[test]
    fn remove_indices_removes_correct_items() {
        let mut s = SoaVecStorage::<MockEntity>::new(8);
        for v in [1.0f64, 2.0, 3.0, 4.0, 5.0] {
            MockEntity { d64: v, c8: 0 }.push_cols(&mut s.columns);
            s.len += 1;
        }
        s.remove_indices(vec![1, 3]);
        assert_eq!(s.len(), 3);
        
        // Read back the remaining items and check
        let remaining: Vec<f64> = (0..s.len())
            .map(|i| MockEntity::read_cols(&s.columns, i).d64)
            .collect();
        assert!(!remaining.contains(&2.0));
        assert!(!remaining.contains(&4.0));
    }

    #[test]
    fn remove_indices_handles_empty() {
        let mut s = SoaVecStorage::<MockEntity>::new(4);
        MockEntity::default().push_cols(&mut s.columns);
        s.len += 1;
        s.remove_indices(vec![]);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn remove_indices_deduplicates() {
        let mut s = SoaVecStorage::<MockEntity>::new(4);
        MockEntity::default().push_cols(&mut s.columns);
        s.len += 1;
        MockEntity::default().push_cols(&mut s.columns);
        s.len += 1;
        s.remove_indices(vec![0, 0, 0]);
        assert_eq!(s.len(), 1);
    }
}