use crate::sim::storage::{AosStorage, ElementStorage, Storage};

 pub struct AosVecStorage<Element> {
    items: Vec<Element>,
}

impl<Element> AosVecStorage<Element> {
    #[inline(always)]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl<Element> Storage for AosVecStorage<Element> {
    #[inline(always)]
    fn len(&self) -> usize { 
        self.items.len() 
    }
    
    #[inline(always)]
    fn capacity(&self) -> usize { 
        self.items.capacity() 
    }
    
    #[inline(always)]
    fn clear(&mut self) { 
        self.items.clear(); 
    } 
}

impl<Element: 'static> ElementStorage for AosVecStorage<Element> {
    type Element = Element;  
    
    // Wire the associated view types directly to standard safe Rust slices
    type View<'a> = &'a [Element] where Self: 'a;
    type ViewMut<'a> = &'a mut [Element] where Self: 'a;

    #[inline(always)]
    fn view(&self) -> Self::View<'_> {
        &self.items
    }

    #[inline(always)]
    fn view_mut(&mut self) -> Self::ViewMut<'_> {
        &mut self.items
    }

    #[inline(always)]
    fn push(&mut self, element: Self::Element) {  
        self.items.push(element);
    }

    #[inline(always)]
    fn swap_remove(&mut self, index: usize) -> Self::Element {
        self.items.swap_remove(index)
    }
}

impl<Element: 'static> AosStorage for AosVecStorage<Element> {
    #[inline(always)]
    fn as_slice(&self) -> &[Self::Element] {
        &self.items
    }

    #[inline(always)]
    fn as_slice_mut(&mut self) -> &mut [Self::Element] {
        &mut self.items
    }
}