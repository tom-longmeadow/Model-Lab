use std::alloc::{Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;  
 
use crate::sim::storage::{ElementStorage, SoaColumn, SoaLayout, SoaView,  Storage};

pub struct Column {
    pub bytes: Vec<u8>,
    pub element_layout: Layout,
}

pub struct SoaVecStorage<L: SoaLayout> {
    pub columns: Vec<Column>,
    raw_columns: Vec<SoaColumn>, 
    len: usize,
    _marker: PhantomData<L>,
}

impl<L: SoaLayout> SoaVecStorage<L> {
    pub fn new(capacity: usize) -> Self {
        let mut columns = Vec::with_capacity(L::LAYOUTS.len());
        let mut raw_columns = Vec::with_capacity(L::LAYOUTS.len());

        for &layout in L::LAYOUTS {
            columns.push(Column {
                bytes: Vec::new(),
                element_layout: layout,
            });
            raw_columns.push(SoaColumn {
                ptr: NonNull::dangling(),
                cap: 0,
                element_layout: layout,
            });
        }

        let mut storage = Self {
            columns,
            raw_columns,
            len: 0,
            _marker: PhantomData,
        };

        if capacity > 0 {
            storage.grow_to(capacity);
        }
        storage
    }

    pub fn grow_to(&mut self, new_capacity: usize) {
        for col in &mut self.columns {
            let size = col.element_layout.size();
            if size > 0 {
                let current_cap = col.bytes.capacity() / size;
                if new_capacity > current_cap {
                    let additional_items = new_capacity - current_cap;
                    col.bytes.reserve(additional_items * size);
                }
            }
        }
        self.sync_raw_descriptors();
    }

    fn sync_raw_descriptors(&mut self) {
        let layout_count = L::LAYOUTS.len();
        let hot_columns = unsafe { self.columns.get_unchecked_mut(0..layout_count) };
        let hot_raw = unsafe { self.raw_columns.get_unchecked_mut(0..layout_count) };

        for i in 0..layout_count {
            let col = &mut hot_columns[i];
            let size = col.element_layout.size();
            let cap = if size == 0 { 0 } else { col.bytes.capacity() / size };
            
            let ptr = unsafe { NonNull::new_unchecked(col.bytes.as_mut_ptr()) };
            
            hot_raw[i] = SoaColumn {
                ptr,
                cap,
                element_layout: col.element_layout,
            };
        }
    }

    // FIXED: Made public so your view methods can guarantee perfect sync states
    pub unsafe fn update_backing_byte_lengths(&mut self, item_len: usize) {
        for col in &mut self.columns {
            let size = col.element_layout.size();
            unsafe { col.bytes.set_len(item_len * size) };
        }
    }

    #[inline(always)]
    pub fn view(&self) -> SoaView<'_, L> {
        SoaView::new(&self.raw_columns, self.len)
    }

    #[inline(always)]
    pub fn view_mut(&mut self) -> SoaView<'_, L> {
        // FIXED: Force full structural sync immediately before view extraction passes!
        self.sync_raw_descriptors();
        SoaView::new(&self.raw_columns, self.len)
    }
}

impl<L: SoaLayout> Storage for SoaVecStorage<L> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.raw_columns.first().map(|c| c.cap).unwrap_or(0)
    }

    fn clear(&mut self) {
        for i in (0..self.len).rev() {
            unsafe { L::drop_cols(&self.raw_columns, i); }
        }
        for col in &mut self.columns {
            col.bytes.clear();
        }
        self.len = 0;
        self.sync_raw_descriptors();
    }
}

impl<L: SoaLayout + 'static> ElementStorage for SoaVecStorage<L> {
    type Element = L;
    type View<'a> = SoaView<'a, L> where Self: 'a;
    type ViewMut<'a> = SoaView<'a, L> where Self: 'a; 

    #[inline(always)] fn view(&self) -> Self::View<'_> { self.view() }
    #[inline(always)] fn view_mut(&mut self) -> Self::ViewMut<'_> { self.view_mut() }

    fn push(&mut self, element: Self::Element) {
        let current_cap = self.capacity();
        if self.len >= current_cap {
            let new_cap = if current_cap == 0 { 4 } else { current_cap * 2 };
            self.grow_to(new_cap);
        }

        unsafe {
            self.update_backing_byte_lengths(self.len + 1);
        }
        
        self.sync_raw_descriptors();

        unsafe {
            L::push_cols(element, &mut self.raw_columns, self.len);
        }
        
        self.len += 1;
    }

    fn swap_remove(&mut self, index: usize) -> Self::Element {
        assert!(index < self.len, "Index out of bounds");
        unsafe {
            let item = L::read_cols(&self.raw_columns, index);
            let tail_idx = self.len - 1;

            if index != tail_idx {
                let layout_count = L::LAYOUTS.len();
                let hot_raw = unsafe { self.raw_columns.get_unchecked(0..layout_count) };
                
                for col in hot_raw {
                    let size = col.element_layout.size();
                    if size > 0 {
                        let base_ptr = col.ptr.as_ptr();
                        let src = base_ptr.add(tail_idx * size);
                        let dst = base_ptr.add(index * size);
                        std::ptr::copy_nonoverlapping(src, dst, size);
                    }
                }
            }

            self.len -= 1;
            
            // FIXED: Keep internal vector byte lengths perfectly in lockstep 
            // with your tracking variables, preventing memory view drift!
            self.update_backing_byte_lengths(self.len);
            self.sync_raw_descriptors();
            
            item
        }
    }
}

// // =========================================================================
// // 1. UNDERLYING REALLOCATION CONTAINERS
// // =========================================================================

// /// An abstraction representing a layout lane owning its backing memory block.
// pub struct Column {
//     pub bytes: Vec<u8>,
//     pub element_layout: Layout,
// }

// /// The generic, layout-agnostic Structure-of-Arrays linear vector storage engine.
// pub struct SoaVecStorage<L: SoaLayout> {
//     columns: Vec<Column>,
//     raw_columns: Vec<SoaColumn>, // Mirrored safe tracking pointers for views
//     len: usize,
//     _marker: PhantomData<L>,
// }

// impl<L: SoaLayout> SoaVecStorage<L> {
//     /// Creates a new SoA vector store initialized with an explicit allocation capacity.
//     pub fn new(capacity: usize) -> Self {
//         let mut columns = Vec::new();
//         let mut raw_columns = Vec::new();

//         // Dynamically instantiate independent lanes from static layout configuration
//         for &layout in L::LAYOUTS {
//             columns.push(Column {
//                 bytes: Vec::new(),
//                 element_layout: layout,
//             });
//             raw_columns.push(SoaColumn {
//                 ptr: NonNull::dangling(),
//                 cap: 0,
//                 element_layout: layout,
//             });
//         }

//         let mut storage = Self {
//             columns,
//             raw_columns,
//             len: 0,
//             _marker: PhantomData,
//         };

//         if capacity > 0 {
//             storage.grow_to(capacity);
//         }
//         storage
//     }

//     /// Grows all column capacities equally to accommodate structural adjustments.
//     pub fn grow_to(&mut self, new_capacity: usize) {
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

//     /// Re-maps low-level slice descriptors to actual vectors after reallocations.
//     fn sync_raw_descriptors(&mut self) {
//         for (i, col) in self.columns.iter_mut().enumerate() {
//             let size = col.element_layout.size();
//             let ptr = unsafe { NonNull::new_unchecked(col.bytes.as_mut_ptr()) };
            
//             self.raw_columns[i] = SoaColumn {
//                 ptr,
//                 cap: if size == 0 { 0 } else { col.bytes.capacity() / size },
//                 element_layout: col.element_layout,
//             };
//         }
//     }

//     /// Internal safe helper to ensure underlying byte allocations sync with tracked item totals.
//     unsafe fn update_backing_byte_lengths(&mut self, item_len: usize) {
//         for col in &mut self.columns {
//             let size = col.element_layout.size();
//             col.bytes.set_len(item_len * size);
//         }
//     }
// }

// // =========================================================================
// // 2. MODERNIZED INTERFACE SYSTEM PROVISIONING (Views instead of raw slices)
// // =========================================================================

// impl<L: SoaLayout> SoaVecStorage<L> {
//     /// Generates a clean, read-only snapshot view of all fields.
//     #[inline(always)]
//     pub fn view(&self) -> SoaView<'_, L> {
//         SoaView::new(&self.raw_columns, self.len)
//     }

//     /// Generates a single mutable layout context controller.
//     #[inline(always)]
//     pub fn view_mut(&mut self) -> SoaViewMut<'_, L> {
//         SoaViewMut::new(&mut self.raw_columns, self.len)
//     }
// }

// // =========================================================================
// // 3. LIFETIME-BOUND INTEGRATION WITH CORE ENGINE TRAITS
// // =========================================================================

// impl<L: SoaLayout> Storage for SoaVecStorage<L> {
//     #[inline(always)]
//     fn len(&self) -> usize {
//         self.len
//     }

//     #[inline(always)]
//     fn capacity(&self) -> usize {
//         self.raw_columns.first().map(|c| c.cap).unwrap_or(0)
//     }

//     fn clear(&mut self) {
//         for i in (0..self.len).rev() {
//             unsafe {
//                 L::drop_cols(&self.raw_columns, i);
//             }
//         }
//         for col in &mut self.columns {
//             col.bytes.clear();
//         }
//         self.len = 0;
//         self.sync_raw_descriptors();
//     }
// }

// impl<L: SoaLayout> ElementStorage for SoaVecStorage<L> {
//     type Item = L;

//     #[inline(always)]
//     fn with_capacity(capacity: usize) -> Self {
//         Self::new(capacity)
//     }

//     fn push(&mut self, item: Self::Element) {
//         let current_cap = self.capacity();
//         if self.len >= current_cap {
//             let new_cap = if current_cap == 0 { 4 } else { current_cap * 2 };
//             self.grow_to(new_cap);
//         }

//         unsafe {
//             L::push_cols(item, &self.raw_columns, self.len);
//             self.update_backing_byte_lengths(self.len + 1);
//         }
//         self.len += 1;
//         self.sync_raw_descriptors();
//     }

//     fn swap_remove(&mut self, index: usize) -> Self::Element {
//         assert!(index < self.len, "Index out of bounds");
//         unsafe {
//             // Read target out to present as return item
//             let item = L::read_cols(&self.raw_columns, index);

//             // Shift trailing elements directly down to balance structural alignment
//             let tail_idx = self.len - 1;
//             if index != tail_idx {
//                 for i in 0..L::LAYOUTS.len() {
//                     let col = &self.raw_columns[i];
//                     if col.element_layout.size() > 0 {
//                         let base_ptr = col.ptr.as_ptr();
//                         let size = col.element_layout.size();
                        
//                         let src = base_ptr.add(tail_idx * size);
//                         let dst = base_ptr.add(index * size);
//                         std::ptr::copy_nonoverlapping(src, dst, size);
//                     }
//                 }
//             }

//             self.len -= 1;
//             self.update_backing_byte_lengths(self.len);
//             self.sync_raw_descriptors();
//             item
//         }
//     }
// }


// pub struct SoaStorageStream<'a, V: Vector> {
//     storage: &'a mut SoaVecStorage<VerletParticle<V>>,
// }

// impl<'a, V: Vector> SoaStorageStream<'a, V> {
//     pub fn new(storage: &'a mut SoaVecStorage<VerletParticle<V>>) -> Self {
//         Self { storage }
//     }
// }

// impl<'a, V: Vector> ParticlePushStream<V> for SoaStorageStream<'a, V>
// where
//     V: 'static,
//     V::Scalar: 'static,
// {
//     #[inline(always)]
//     fn push_components(
//         &mut self,
//         pos: V,
//         pos_old: V,
//         acc: V,
//         radius: V::Scalar,
//         color: Color,
//         inv_mass: V::Scalar,
//     ) {
//         let current_cap = self.storage.capacity();
//         let current_len = self.storage.len;

//         // Auto-grow storage backing lanes if capacity bound is met
//         if current_len >= current_cap {
//             let new_cap = if current_cap == 0 { 4 } else { current_cap * 2 };
//             self.storage.grow_to(new_cap);
//         }

//         // Direct low-level unsafe pointer write into the sequential column offsets
//         unsafe {
//             let cols = &mut self.storage.raw_columns;
//             ptr::write(cols[VerletParticleColumns::Pos as usize].ptr.cast::<V>().add(current_len), pos);
//             ptr::write(cols[VerletParticleColumns::PosOld as usize].ptr.cast::<V>().add(current_len), pos_old);
//             ptr::write(cols[VerletParticleColumns::Acc as usize].ptr.cast::<V>().add(current_len), acc);
//             ptr::write(cols[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>().add(current_len), radius);
//             ptr::write(cols[VerletParticleColumns::Color as usize].ptr.cast::<Color>().add(current_len), color);
//             ptr::write(cols[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>().add(current_len), inv_mass);

//             self.storage.update_backing_byte_lengths(current_len + 1);
//         }

//         self.storage.len += 1;
//         self.storage.sync_raw_descriptors();
//     }
// }
// pub struct ParticleLifecycle<V: Vector, S: ParticleSpawner<V>> {
//     pub spawner: S,
//     pub phantom: std::marker::PhantomData<V>,
// }

// impl<V, S, A> Lifecycle<SoaVecStorage<VerletParticle<V>>, ParticleEnvironment<V, A>> for ParticleLifecycle<V, S> 
// where
//     V: Vector + std::ops::Sub<Output = V> + 'static,  
//     V::Scalar: 'static,
//     A: ParticleAttributes<V> + 'static,
//     S: ParticleSpawner<V>, 
// {
//     fn tick(
//         &mut self, 
//         storage: &mut SoaVecStorage<VerletParticle<V>>, 
//         tick: u64, 
//         step_dt: f64, 
//         environment: &ParticleEnvironment<V, A>
//     ) {
//         let current_count = storage.len();
//         let max_limit = storage.capacity(); 

//         // Create the direct pipeline bridge right on the loop execution stack
//         let mut stream = SoaStorageStream::new(storage);

//         // Drive execution down into the agnostic spawner engine
//         self.spawner.try_spawn(tick, max_limit, current_count, step_dt, environment, &mut stream); 
//     } 
// }
// pub struct SoaVecStorage<Entity: SoaLayout> {
//     pub(crate) columns: Vec<RawColumn>,
//     len: usize,
//     capacity: usize,
//     _marker: PhantomData<Entity>,
// }

// // 🟢 Completely generic layout adjustments
// impl<Entity: SoaLayout> SoaVecStorage<Entity> { 
//     fn grow_to(&mut self, new_capacity: usize) {
//         if new_capacity <= self.capacity {
//             return;
//         }

//         for (i, layout) in Entity::LAYOUTS.iter().enumerate() {
//             let col = &mut self.columns[i];
//             if layout.size() == 0 {
//                 col.cap = new_capacity;
//                 continue;
//             }

//             unsafe {
//                 let new_ptr = if col.cap == 0 {
//                     let new_layout = Layout::from_size_align_unchecked(
//                         layout.size() * new_capacity, 
//                         layout.align()
//                     );
//                     alloc(new_layout)
//                 } else {
//                     let old_layout = Layout::from_size_align_unchecked(
//                         layout.size() * col.cap, 
//                         layout.align()
//                     );
//                     realloc(col.ptr, old_layout, layout.size() * new_capacity)
//                 };

//                 if new_ptr.is_null() {
//                     handle_alloc_error(Layout::from_size_align_unchecked(
//                         layout.size() * new_capacity, 
//                         layout.align()
//                     ));
//                 }
//                 col.ptr = new_ptr;
//                 col.cap = new_capacity;
//             }
//         }
//         self.capacity = new_capacity;
//     }

//     // ======================================================================
//     // 🟢 NEW ERGONOMIC SLICING API: Accepting any enum implementing SoaProperty
//     // ======================================================================

//     /// Safely retrieve an immutable array slice via any enum type matching your index layout.
//     #[inline(always)]
//     pub fn slice<T, P: SoaProperty>(&self, prop: P) -> &[T] {
//         Entity::slice_from_cols(
//             &self.columns, 
//             prop.column_index(), 
//             self.len
//         )
//     }

//     /// Safely retrieve a mutable array slice via any enum type matching your index layout.
//     #[inline(always)]
//     pub fn slice_mut<T, P: SoaProperty>(&mut self, prop: P) -> &mut [T] {
//         Entity::slice_from_cols_mut(
//             &mut self.columns, 
//             prop.column_index(), 
//             self.len
//         )
//     }
// }

// impl<Entity: SoaLayout> Storage for SoaVecStorage<Entity> {
//     #[inline(always)]
//     fn len(&self) -> usize { 
//         self.len 
//     }
    
//     #[inline(always)]
//     fn capacity(&self) -> usize { 
//         self.capacity 
//     }
    
//     fn clear(&mut self) {
//         for i in (0..self.len).rev() {
//             unsafe { 
//                 drop(Entity::read_cols(&self.columns, i)); 
//             }
//         }
//         self.len = 0;
//     }
// }

// impl<Entity: SoaLayout> CpuStorage for SoaVecStorage<Entity> {
//     type Item = Entity;

//     fn new(capacity: usize) -> Self {
//         let columns = Entity::LAYOUTS.iter().map(|&layout| RawColumn {
//             ptr: ptr::null_mut(),
//             cap: 0,
//             element_layout: layout,
//         }).collect();

//         let mut storage = Self { 
//             columns, 
//             len: 0, 
//             capacity: 0, 
//             _marker: PhantomData 
//         };
        
//         if capacity > 0 { 
//             storage.grow_to(capacity); 
//         }
//         storage
//     }

//     fn push(&mut self, item: Self::Item) {
//         if self.len >= self.capacity {
//             let new_cap = if self.capacity == 0 { 4 } else { self.capacity * 2 };
//             self.grow_to(new_cap);
//         }
//         unsafe {
//             item.push_cols(&mut self.columns, self.len);
//         }
//         self.len += 1;
//     }

//     fn swap_remove(&mut self, index: usize) -> Self::Item {
//         assert!(index < self.len, "Index out of bounds");
//         unsafe {
//             let item = Entity::read_cols(&self.columns, index);
//             Entity::swap_remove_cols(&mut self.columns, index, self.len);
//             self.len -= 1;
//             item
//         }
//     }
// }

// impl<Entity: SoaLayout> SoaCpuStorage for SoaVecStorage<Entity> {
//     type Layout = Entity;
//     #[inline(always)]
//     fn columns(&self) -> &[RawColumn] { &self.columns }
//     #[inline(always)]
//     fn columns_mut(&mut self) -> &mut [RawColumn] { &mut self.columns }
//     #[inline(always)]
//     fn set_len(&mut self, new_len: usize) { self.len = new_len; }
// }

// impl<Entity: SoaLayout> Drop for SoaVecStorage<Entity> {
//     fn drop(&mut self) {
//         self.clear();
        
//         for (i, layout) in Entity::LAYOUTS.iter().enumerate() {
//             let col = &self.columns[i];
//             if col.cap > 0 && layout.size() > 0 && !col.ptr.is_null() {
//                 unsafe {
//                     let old_layout = Layout::from_size_align_unchecked(
//                         layout.size() * col.cap, 
//                         layout.align()
//                     );
//                     dealloc(col.ptr, old_layout);
//                 }
//             }
//         }
//     }
// }
 