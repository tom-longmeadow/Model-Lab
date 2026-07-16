use crate::sim::storage::{AosCpuStorage, CpuStorage, Storage};

 

pub struct AosVecStorage<Item> {
    items: Vec<Item>,
}

impl<Item> AosVecStorage<Item> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl<Item> Storage for AosVecStorage<Item> {
    fn len(&self) -> usize { 
        self.items.len() 
    }
    
    fn capacity(&self) -> usize { 
        self.items.capacity() 
    }
    
    fn clear(&mut self) { 
        self.items.clear(); 
    } 
}

impl<Item> CpuStorage for AosVecStorage<Item> {
    type Item = Item; // FIXED: Associated type moves to CpuStorage

    fn new(capacity: usize) -> Self {
        Self { items: Vec::with_capacity(capacity) }
    }

    fn push(&mut self, item: Self::Item) { // FIXED: push moves to CpuStorage
        self.items.push(item);
    }

    fn swap_remove(&mut self, index: usize) -> Self::Item { // FIXED: swap_remove moves to CpuStorage
        self.items.swap_remove(index)
    }
}

impl<Item> AosCpuStorage for AosVecStorage<Item> {
    fn as_slice(&self) -> &[Self::Item] {
        &self.items
    }

    fn as_slice_mut(&mut self) -> &mut [Self::Item] {
        &mut self.items
    }
}