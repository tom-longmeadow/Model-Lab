use crate::sim::storage::{SoaLayout, SoaStorage, Storage};

/// Generic CPU-side Struct-of-Arrays storage backed by `Vec<u8>` byte columns.
/// `T` declares its own column layout via [`SoaLayout`].
/// Storage manages raw byte columns — knows nothing about field semantics.
/// Typed safe column access lives in the physics sub-traits
/// ([`SoaNewtonianStorage`], [`SoaVerletStorage`]).
pub struct SoaVecStorage<T: SoaLayout> {
    /// One `Vec<u8>` per field — length is `len * stride` for that column.
    columns:  Vec<Vec<u8>>,
    strides:  Vec<usize>,
    len:      usize,
    capacity: usize,
    _marker:  std::marker::PhantomData<T>,
}

impl<T: SoaLayout> SoaVecStorage<T> {
    /// Typed immutable slice over one byte column.
    ///
    /// # Safety
    /// `C` must match the actual element type for `col`.
    /// Only called from physics sub-trait impls which know the correct type.
    #[allow(dead_code)]
    pub(crate) unsafe fn col_raw<C>(&self, col: usize) -> &[C] {
        let bytes = &self.columns[col];
        unsafe {
            std::slice::from_raw_parts(
                bytes.as_ptr() as *const C,
                bytes.len() / self.strides[col],
            )
        }
    }

    /// Typed mutable slice over one byte column.
    ///
    /// # Safety
    /// `C` must match the actual element type for `col`.
    /// Only called from physics sub-trait impls which know the correct type.
    #[allow(dead_code)]
    pub(crate) unsafe fn col_raw_mut<C>(&mut self, col: usize) -> &mut [C] {
        let len   = self.columns[col].len() / self.strides[col];
        let bytes = &mut self.columns[col];
        unsafe {
            std::slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut C, len)
        }
    }

    /// Borrow columns `a` and `b` mutably and column `c` immutably in one call.
    /// All three indices must be distinct.
    ///
    /// # Safety
    /// `C` must match the actual element type for all three columns.
    /// `a`, `b`, `c` must be distinct valid column indices.
    #[allow(dead_code)]
    pub(crate) unsafe fn col3_mut_mut_ref<C>(
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
        let len_a = col_a.len() / self.strides[a];
        let len_b = col_b.len() / self.strides[b];
        let len_c = col_c.len() / self.strides[c];
        unsafe {(
            std::slice::from_raw_parts_mut(col_a.as_mut_ptr() as *mut C, len_a),
            std::slice::from_raw_parts_mut(col_b.as_mut_ptr() as *mut C, len_b),
            std::slice::from_raw_parts    (col_c.as_ptr()     as *const C, len_c),
        )}
    }
}

impl<T: SoaLayout> Storage for SoaVecStorage<T> {
    type Item = T;

    fn new(capacity: usize) -> Self {
        let strides: Vec<usize> = T::STRIDES.to_vec();
        let columns = strides.iter()
            .map(|s| Vec::with_capacity(capacity * s))
            .collect();
        Self { columns, strides, len: 0, capacity, _marker: std::marker::PhantomData }
    }

    fn len(&self)      -> usize { self.len }
    fn capacity(&self) -> usize { self.capacity }

    fn push(&mut self, item: T) {
        item.push_cols(&mut self.columns);
        self.len += 1;
    }

    fn swap_remove(&mut self, index: usize) -> T {
        let item = T::read_cols(&self.columns, index);
        T::swap_remove_cols(&mut self.columns, &self.strides, index);
        self.len -= 1;
        item
    }

    fn clear(&mut self) {
        self.columns.iter_mut().for_each(|c| c.clear());
        self.len = 0;
    }
}

impl<T: SoaLayout> SoaStorage for SoaVecStorage<T> {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::sim::storage::SoaLayout;
    use super::*;

    #[derive(Default, PartialEq, Debug)]
    pub struct MockEntity {
        pub d64: f64,
        pub c8:  u8,
    }

    impl SoaLayout for MockEntity {
        const STRIDES: &'static [usize] = &[
            std::mem::size_of::<f64>(),
            std::mem::size_of::<u8>(),
        ];

        fn push_cols(&self, cols: &mut [Vec<u8>]) {
            cols[0].extend_from_slice(&self.d64.to_ne_bytes());
            cols[1].extend_from_slice(&self.c8.to_ne_bytes());
        }

        fn read_cols(cols: &[Vec<u8>], index: usize) -> Self {
            let d64 = f64::from_ne_bytes(cols[0][index * 8..][..8].try_into().unwrap());
            let c8  = cols[1][index];
            Self { d64, c8 }
        }

        fn swap_remove_cols(cols: &mut [Vec<u8>], strides: &[usize], index: usize) {
            for (col, &stride) in cols.iter_mut().zip(strides.iter()) {
                let last_start = col.len() - stride;
                let tgt_start  = index * stride;
                col.copy_within(last_start.., tgt_start);
                col.truncate(last_start);
            }
        }
    }

    crate::test_storage!(SoaVecStorage<MockEntity>, MockEntity);

    #[test]
    fn round_trip_push_read() {
        let mut s = SoaVecStorage::<MockEntity>::new(4);
        s.push(MockEntity { d64: 1.5, c8: 10 });
        s.push(MockEntity { d64: 2.5, c8: 20 });
        assert_eq!(s.len(), 2);
        let item = MockEntity::read_cols(&s.columns, 1);
        assert_eq!(item, MockEntity { d64: 2.5, c8: 20 });
    }

    #[test]
    fn swap_remove_keeps_data_consistent() {
        let mut s = SoaVecStorage::<MockEntity>::new(4);
        s.push(MockEntity { d64: 1.0, c8: 1 });
        s.push(MockEntity { d64: 2.0, c8: 2 });
        s.push(MockEntity { d64: 3.0, c8: 3 });
        let removed = s.swap_remove(0);
        assert_eq!(removed, MockEntity { d64: 1.0, c8: 1 });
        assert_eq!(s.len(), 2);
        // last item moved to slot 0
        let new_first = MockEntity::read_cols(&s.columns, 0);
        assert_eq!(new_first, MockEntity { d64: 3.0, c8: 3 });
    }
}
