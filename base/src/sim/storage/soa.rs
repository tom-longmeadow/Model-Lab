use std::alloc::{alloc, dealloc, Layout};

pub struct SoaColumn {
    ptr:      *mut u8,
    len:      usize,
    capacity: usize,
    stride:   usize,   // size of one element in bytes
}

impl SoaColumn {
    pub fn new<T>(capacity: usize) -> Self {
        let stride  = std::mem::size_of::<T>();
        let layout  = Layout::array::<T>(capacity).unwrap();
        let ptr     = unsafe { alloc(layout) };
        Self { ptr, len: 0, capacity, stride }
    }

    pub fn as_slice<T>(&self) -> &[T] {
        assert_eq!(self.stride, std::mem::size_of::<T>());
        unsafe { std::slice::from_raw_parts(self.ptr as *const T, self.len) }
    }

    pub fn as_slice_mut<T>(&mut self) -> &mut [T] {
        assert_eq!(self.stride, std::mem::size_of::<T>());
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut T, self.len) }
    }

    pub fn push<T>(&mut self, value: T) {
        assert_eq!(self.stride, std::mem::size_of::<T>());
        assert!(self.len < self.capacity);
        unsafe {
            let dst = self.ptr.add(self.len * self.stride) as *mut T;
            dst.write(value);
        }
        self.len += 1;
    }

    pub fn swap_remove(&mut self, index: usize) {
        assert!(index < self.len);
        let last = self.len - 1;
        if index != last {
            unsafe {
                let a = self.ptr.add(index * self.stride);
                let b = self.ptr.add(last  * self.stride);
                std::ptr::swap_nonoverlapping(a, b, self.stride);
            }
        }
        self.len -= 1;
    }

    pub fn len(&self)    -> usize { self.len }
    pub fn stride(&self) -> usize { self.stride }
}

impl Drop for SoaColumn {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(
            self.capacity * self.stride,
            std::mem::align_of::<u64>(),   // safe over-alignment
        ).unwrap();
        unsafe { dealloc(self.ptr, layout) };
    }
}


pub struct SoaData {
    columns:  Vec<SoaColumn>,
    len:      usize,
    capacity: usize,
}

impl SoaData {
    pub fn new(capacity: usize) -> Self {
        Self { columns: Vec::new(), len: 0, capacity }
    }

    pub fn add_column<T>(&mut self) -> usize {
        let index = self.columns.len();
        self.columns.push(SoaColumn::new::<T>(self.capacity));
        index
    }

    pub fn col<T>(&self, index: usize) -> &[T] {
        self.columns[index].as_slice::<T>()
    }

    pub fn col_mut<T>(&mut self, index: usize) -> &mut [T] {
        self.columns[index].as_slice_mut::<T>()
    }

    pub fn len(&self) -> usize { self.len }
}

/// User implements this to define column layout and item mapping.
pub trait SoaLayout: Sized {
    /// Called once on construction — add columns via soa.add_column::<T>()
    fn setup(soa: &mut SoaData);

    /// Push one item into the correct columns
    fn push(item: Self, soa: &mut SoaData);
}

pub struct SoaStorage<L: SoaLayout> {
    soa: SoaData,
    _marker: std::marker::PhantomData<L>,
}

impl<L: SoaLayout> SoaStorage<L> {
    /// Direct column access — solver uses this
    pub fn col<T>(&self, index: usize) -> &[T] {
        self.soa.col::<T>(index)
    }

    pub fn col_mut<T>(&mut self, index: usize) -> &mut [T] {
        self.soa.col_mut::<T>(index)
    }

    pub fn push(&mut self, item: L) {
        L::push(item, &mut self.soa);
        self.soa.len += 1;
    }

    pub fn swap_remove(&mut self, index: usize) {
        for col in &mut self.soa.columns {
            col.swap_remove(index);
        }
        self.soa.len -= 1;
    }

    pub fn clear(&mut self) {
        for col in &mut self.soa.columns {
            col.len = 0;
        }
        self.soa.len = 0;
    }
}

impl<L: SoaLayout> crate::sim::storage::Storage for SoaStorage<L> {
    type Item = L;

    fn new(capacity: usize) -> Self {
        let mut soa = SoaData::new(capacity);
        L::setup(&mut soa);
        Self { soa, _marker: std::marker::PhantomData }
    }

    fn len(&self)      -> usize { self.soa.len }
    fn capacity(&self) -> usize { self.soa.capacity }
}

pub struct Particle { pub x: f32, pub y: f32, pub z: f32, pub kind: u32 }

pub enum ParticleCol { X = 0, Y = 1, Z = 2, Kind = 3 }

impl SoaLayout for Particle {
    fn setup(soa: &mut SoaData) {
        soa.add_column::<f32>();  // X
        soa.add_column::<f32>();  // Y
        soa.add_column::<f32>();  // Z
        soa.add_column::<u32>();  // Kind
    }

    fn push(item: Self, soa: &mut SoaData) {
        soa.columns[ParticleCol::X    as usize].push::<f32>(item.x);
        soa.columns[ParticleCol::Y    as usize].push::<f32>(item.y);
        soa.columns[ParticleCol::Z    as usize].push::<f32>(item.z);
        soa.columns[ParticleCol::Kind as usize].push::<u32>(item.kind);
    }
}

// concrete storage type
type ParticleStorage = SoaStorage<Particle>;

// solver accesses columns directly
// let xs: &[f32] = storage.col::<f32>(ParticleCol::X as usize);
// let ks: &[u32] = storage.col::<u32>(ParticleCol::Kind as usize);

 
// pub struct SoaData<T>{
//     pub data: Vec<SoaArray<T>>,
// }


 