use std::{alloc::Layout, marker::PhantomData, ptr, slice};

use crate::{math::Vector, sim::{solver::{particle::verlet_particle::{VerletParticle, VerletParticleColumns}, 
    soa_vec_storage::{ErgonomicSoaLayout, SoaProperty, SoaVecStorage}}, storage::{RawColumn, SoaLayout, Storage}}, ui::layout::color::Color};
 
pub type VerletParticleSoaVecStorage<V> = SoaVecStorage<VerletParticle<V>>;
  
impl SoaProperty for VerletParticleColumns {
    #[inline(always)]
    fn column_index(&self) -> usize {
        *self as usize
    }
}

unsafe impl<V> SoaLayout for VerletParticle<V> 
where 
    V: Vector + 'static,
    V::Scalar: 'static,
{
    const LAYOUTS: &'static [Layout] = &[
        Layout::new::<V>(),          // Pos
        Layout::new::<V>(),          // PosOld
        Layout::new::<V>(),          // Acc
        Layout::new::<V::Scalar>(),  // Radius
        Layout::new::<Color>(),      // Color
        Layout::new::<V::Scalar>(),  // Inv_Mass
    ];

    #[inline]
    unsafe fn push_cols(&self, cols: &mut [RawColumn], index: usize) {
        unsafe {
            ptr::write(cols[VerletParticleColumns::Pos as usize].ptr.cast::<V>().add(index), self.pos);
            ptr::write(cols[VerletParticleColumns::PosOld as usize].ptr.cast::<V>().add(index), self.pos_old);
            ptr::write(cols[VerletParticleColumns::Acc as usize].ptr.cast::<V>().add(index), self.acc);
            ptr::write(cols[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>().add(index), self.radius);
            ptr::write(cols[VerletParticleColumns::Color as usize].ptr.cast::<Color>().add(index), self.color);
            ptr::write(cols[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>().add(index), self.inv_mass);
        }
    }

    #[inline]
    unsafe fn read_cols(cols: &[RawColumn], index: usize) -> Self {
        unsafe {
            Self {
                pos:      ptr::read(cols[VerletParticleColumns::Pos as usize].ptr.cast::<V>().add(index)),
                pos_old:  ptr::read(cols[VerletParticleColumns::PosOld as usize].ptr.cast::<V>().add(index)),
                acc:      ptr::read(cols[VerletParticleColumns::Acc as usize].ptr.cast::<V>().add(index)),
                radius:   ptr::read(cols[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>().add(index)),
                color:    ptr::read(cols[VerletParticleColumns::Color as usize].ptr.cast::<Color>().add(index)),
                inv_mass: ptr::read(cols[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>().add(index)),
            }
        }
    }

    #[inline]
    unsafe fn swap_remove_cols(cols: &mut [RawColumn], index: usize, current_len: usize) {
        let tail_index = current_len - 1;
        if index == tail_index { return; }
        
        unsafe {
            ptr::copy_nonoverlapping(cols[VerletParticleColumns::Pos as usize].ptr.cast::<V>().add(tail_index), cols[VerletParticleColumns::Pos as usize].ptr.cast::<V>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletParticleColumns::PosOld as usize].ptr.cast::<V>().add(tail_index), cols[VerletParticleColumns::PosOld as usize].ptr.cast::<V>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletParticleColumns::Acc as usize].ptr.cast::<V>().add(tail_index), cols[VerletParticleColumns::Acc as usize].ptr.cast::<V>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>().add(tail_index), cols[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletParticleColumns::Color as usize].ptr.cast::<Color>().add(tail_index), cols[VerletParticleColumns::Color as usize].ptr.cast::<Color>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>().add(tail_index), cols[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>().add(index), 1);
        }
    }
}

unsafe impl<V> ErgonomicSoaLayout for VerletParticle<V>
where
    V: Vector + 'static,
    V::Scalar: 'static,
{
    type Property = VerletParticleColumns;
}

// ============================================================================
// ZERO-COST SCALAR STRIDING SLICES
// ============================================================================

/// Acts exactly like a slice, but allows stepping through interleaved vector layouts
pub struct ComponentSliceMut<'a, S> {
    pub ptr: *mut S,
    len: usize,
    _marker: PhantomData<&'a mut [S]>,
}

impl<'a, S: Copy> ComponentSliceMut<'a, S> {
    #[inline(always)]
    pub unsafe fn from_raw_parts_mut(ptr: *mut S, len: usize) -> Self {
        Self { ptr, len, _marker: PhantomData }
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.len }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// Index-get optimized for compiler pointer arithmetic register tracking
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> S {
        // Strides by 2 elements to jump across [X, Y] vector component boundaries
        unsafe { *self.ptr.add(index * 2) }
    }

    /// Index-set optimized for compiler pointer arithmetic register tracking
    #[inline(always)]
    pub unsafe fn set_unchecked(&self, index: usize, value: S) {
        unsafe { *self.ptr.add(index * 2) = value };
    }
}

/// Same structural concept for read-only scalar properties
pub struct ComponentSlice<'a, S> {
    pub ptr: *const S,
    len: usize,
    _marker: PhantomData<&'a [S]>,
}

impl<'a, S: Copy> ComponentSlice<'a, S> {
    #[inline(always)]
    pub unsafe fn from_raw_parts(ptr: *const S, len: usize) -> Self {
        Self { ptr, len, _marker: PhantomData }
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.len }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.len == 0 }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> S {
        unsafe { *self.ptr.add(index * 2) }
    }
}

// ============================================================================
// ERGONOMIC HIGH-PERFORMANCE ENUM-DRIVEN EXTENSION TRAIT
// ============================================================================

pub trait ErgonomicSoaCpuStorageExt<Entity: ErgonomicSoaLayout> {
    /// Safe, zero-cost read-only slice retrieval for flat single-scalar columns
    fn as_slice<T>(&self, prop: Entity::Property) -> &[T];

    /// Safe, zero-cost mutable slice retrieval for flat single-scalar columns
    fn as_mut_slice<T>(&mut self, prop: Entity::Property) -> &mut [T];

    /// Safely unpacks all individual vector columns into raw read-only scalar 
    /// strided slices using your explicit `VerletParticleColumns` enum map.
    fn get_physics_components<V: Vector>(
        &self,
    ) -> (
        ComponentSlice<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSlice<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        &[V::Scalar],              
        &[V::Scalar],              
    );

    /// Safely unpacks all individual vector columns into raw mutable scalar 
    /// strided slices using your explicit `VerletParticleColumns` enum map.
    fn get_physics_components_mut<V: Vector>(
        &mut self,
    ) -> (
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        &mut [V::Scalar],             
        &mut [V::Scalar],             
    );
}

impl<Entity: ErgonomicSoaLayout> ErgonomicSoaCpuStorageExt<Entity> for SoaVecStorage<Entity> {
    #[inline(always)]
    fn as_slice<T>(&self, prop: Entity::Property) -> &[T] {
        unsafe { self.slice_uncheck(prop) }
    }

    #[inline(always)]
    fn as_mut_slice<T>(&mut self, prop: Entity::Property) -> &mut [T] {
        unsafe { self.slice_mut_uncheck(prop) }
    }

    #[inline(always)]
    fn get_physics_components<V: Vector>(
        &self,
    ) -> (
        ComponentSlice<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSlice<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        &[V::Scalar],
        &[V::Scalar],
    ) {
        let len = self.len();
        let c = &self.columns;

        let pos_base  = c[VerletParticleColumns::Pos as usize].ptr.cast::<V::Scalar>();
        let rad_base  = c[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>();
        let mass_base = c[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>();

        unsafe {
            (
                ComponentSlice::from_raw_parts(pos_base.add(0), len), 
                ComponentSlice::from_raw_parts(pos_base.add(1), len), 
                slice::from_raw_parts(rad_base, len),                 
                slice::from_raw_parts(mass_base, len),                
            )
        }
    }

    #[inline(always)]
    fn get_physics_components_mut<V: Vector>(
        &mut self,
    ) -> (
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        ComponentSliceMut<'_, V::Scalar>, // 🟢 FIXED: Added anonymous lifetime marker
        &mut [V::Scalar],
        &mut [V::Scalar],
    ) {
        let len = self.len();
        let c = &self.columns;

        let pos_base  = c[VerletParticleColumns::Pos as usize].ptr.cast::<V::Scalar>();
        let old_base  = c[VerletParticleColumns::PosOld as usize].ptr.cast::<V::Scalar>();
        let acc_base  = c[VerletParticleColumns::Acc as usize].ptr.cast::<V::Scalar>();
        let rad_base  = c[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>();
        let mass_base = c[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>();

        unsafe {
            (
                ComponentSliceMut::from_raw_parts_mut(pos_base.add(0), len), 
                ComponentSliceMut::from_raw_parts_mut(pos_base.add(1), len), 
                ComponentSliceMut::from_raw_parts_mut(old_base.add(0), len), 
                ComponentSliceMut::from_raw_parts_mut(old_base.add(1), len), 
                ComponentSliceMut::from_raw_parts_mut(acc_base.add(0), len), 
                ComponentSliceMut::from_raw_parts_mut(acc_base.add(1), len), 
                slice::from_raw_parts_mut(rad_base, len),                    
                slice::from_raw_parts_mut(mass_base, len),                   
            )
        }
    }
}