use crate::sim::storage::{SoaStorage, Storage};

/// Describes the column layout of a type that can live in SoaVecStorage.
/// Implement this on your entity struct — not on the storage.
pub trait SoaLayout: Sized {
    /// Byte stride of each column — one entry per field.
    const STRIDES: &'static [usize];

    /// Push all fields into their respective byte columns.
    fn push_cols(&self, cols: &mut [Vec<u8>]);

    /// Reconstruct Self from byte columns at index.
    fn read_cols(cols: &[Vec<u8>], index: usize) -> Self;

    /// Swap-remove at index — keeps all columns in sync.
    fn swap_remove_cols(cols: &mut [Vec<u8>], strides: &[usize], index: usize);
}

/// Generic Struct-of-Arrays storage.
/// T declares its own column layout via SoaLayout.
/// Storage manages raw byte columns — knows nothing about field semantics.
pub struct SoaVecStorage<T: SoaLayout> {
    /// One Vec<u8> per field — length is len * stride for that column.
    columns:  Vec<Vec<u8>>,
    strides:  Vec<usize>,
    len:      usize,
    capacity: usize,
    _marker:  std::marker::PhantomData<T>,
}

impl<T: SoaLayout> SoaVecStorage<T> {
    /// Direct column slice — use for SIMD.
    /// SAFETY: caller must request the correct C for the column index.
    /// Use the col:: constants defined on the entity type to uphold this.
   pub unsafe fn col_raw<C>(&self, col: usize) -> &[C] {
        let bytes = &self.columns[col];
        // SAFETY: caller must request the correct C for the column index.
        unsafe {
            std::slice::from_raw_parts(bytes.as_ptr() as *const C, bytes.len() / self.strides[col])
        }
    }

   pub unsafe fn col_raw_mut<C>(&mut self, col: usize) -> &mut [C] {
        let len   = self.columns[col].len() / self.strides[col];
        let bytes = &mut self.columns[col];
        // SAFETY: caller must request the correct C for the column index.
        unsafe {
            std::slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut C, len)
        }
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

/********************/ 
/*      TESTS       */ 
/********************/ 
 
#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Default)]
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

}