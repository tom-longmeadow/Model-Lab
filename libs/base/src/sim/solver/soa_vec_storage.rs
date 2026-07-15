 use std::alloc::{ dealloc, Layout};
use std::marker::PhantomData;
use std::ptr;

use crate::sim::storage::{CpuStorage, RawColumn, SoaCpuStorage, SoaLayout, Storage};
pub struct SoaVecStorage<Entity: SoaLayout> {
    columns: Vec<RawColumn>,
    len: usize,
    capacity: usize,
    _marker: PhantomData<Entity>,
}

impl<Entity: SoaLayout> SoaVecStorage<Entity> { 
    fn grow_to(&mut self, new_capacity: usize) {
        // FIXED: Restored the proper guard condition and brace structure
        if new_capacity <= self.capacity {
            return;
        }

        // Loop through and reallocate memory for each column layout
        for (i, layout) in Entity::LAYOUTS.iter().enumerate() {
            let col = &mut self.columns[i];
            if layout.size() == 0 {
                col.cap = new_capacity;
                continue;
            }

            unsafe {
                let new_ptr = if col.cap == 0 {
                    let new_layout = std::alloc::Layout::from_size_align_unchecked(
                        layout.size() * new_capacity, 
                        layout.align()
                    );
                    std::alloc::alloc(new_layout)
                } else {
                    let old_layout = std::alloc::Layout::from_size_align_unchecked(
                        layout.size() * col.cap, 
                        layout.align()
                    );
                    std::alloc::realloc(col.ptr, old_layout, layout.size() * new_capacity)
                };

                if new_ptr.is_null() {
                    std::alloc::handle_alloc_error(std::alloc::Layout::from_size_align_unchecked(
                        layout.size() * new_capacity, 
                        layout.align()
                    ));
                }
                col.ptr = new_ptr;
                col.cap = new_capacity;
            }
        }
        self.capacity = new_capacity;
    }
 
    pub fn swap_remove(&mut self, index: usize) -> Entity {
        assert!(index < self.len, "Index out of bounds");
        unsafe {
            let item = Entity::read_cols(&self.columns, index);
            Entity::swap_remove_cols(&mut self.columns, index, self.len);
            self.len -= 1;
            item
        }
    }
}

impl<Entity: SoaLayout> Storage for SoaVecStorage<Entity> {
    fn len(&self) -> usize { 
        self.len 
    }
    
    fn capacity(&self) -> usize { 
        self.capacity 
    }
    
    fn clear(&mut self) {
        for i in (0..self.len).rev() {
            unsafe { 
                drop(Entity::read_cols(&self.columns, i)); 
            }
        }
        self.len = 0;
    }
}

impl<Entity: SoaLayout> CpuStorage for SoaVecStorage<Entity> {
    fn new(capacity: usize) -> Self {
        let columns = Entity::LAYOUTS.iter().map(|&layout| RawColumn {
            ptr: ptr::null_mut(),
            cap: 0,
            element_layout: layout,
        }).collect();

        let mut storage = Self { 
            columns, 
            len: 0, 
            capacity: 0, 
            _marker: PhantomData 
        };
        
        if capacity > 0 { 
            storage.grow_to(capacity); 
        }
        storage
    }
}

impl<Entity: SoaLayout> SoaCpuStorage for SoaVecStorage<Entity> {
    type Layout = Entity;
    fn columns(&self) -> &[RawColumn] { &self.columns }
    fn columns_mut(&mut self) -> &mut [RawColumn] { &mut self.columns }
    fn set_len(&mut self, new_len: usize) { self.len = new_len; }
}

impl<Entity: SoaLayout> Drop for SoaVecStorage<Entity> {
    fn drop(&mut self) {
        // Drop elements first so active resources inside fields drop cleanly.
        self.clear();
        
        // Deallocate the actual underlying raw memory heaps.
        for (i, layout) in Entity::LAYOUTS.iter().enumerate() {
            let col = &self.columns[i];
            if col.cap > 0 && layout.size() > 0 && !col.ptr.is_null() {
                unsafe {
                    let old_layout = Layout::from_size_align_unchecked(
                        layout.size() * col.cap, 
                        layout.align()
                    );
                    dealloc(col.ptr, old_layout);
                }
            }
        }
    }
}
