use std::{alloc::Layout, ptr, slice};

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
 
pub trait ErgonomicSoaCpuStorageExt<Entity: ErgonomicSoaLayout> {
    fn get_physics_components_mut<V: Vector>(
        &mut self,
    ) -> (
        &mut [V],        // 🟢 FIXED: Continuous, direct mutable vector slice for positions
        &mut [V],        // 🟢 FIXED: Continuous, direct mutable vector slice for historical steps
        &mut [V],        // 🟢 FIXED: Continuous, direct mutable vector slice for forces
        &mut [V::Scalar], // Clean flat primitive array slice for radii
        &mut [V::Scalar], // Clean flat primitive array slice for masses
    );
}

impl<Entity: ErgonomicSoaLayout> ErgonomicSoaCpuStorageExt<Entity> for SoaVecStorage<Entity> {
    #[inline(always)]
    fn get_physics_components_mut<V: Vector>(
        &mut self,
    ) -> (
        &mut [V],
        &mut [V],
        &mut [V],
        &mut [V::Scalar],
        &mut [V::Scalar],
    ) {
        let len = self.len();
        let c = &self.columns;

        // Extract raw column pointer elements
        let pos_base  = c[VerletParticleColumns::Pos as usize].ptr.cast::<V>();
        let old_base  = c[VerletParticleColumns::PosOld as usize].ptr.cast::<V>();
        let acc_base  = c[VerletParticleColumns::Acc as usize].ptr.cast::<V>();
        let rad_base  = c[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>();
        let mass_base = c[VerletParticleColumns::InvMass as usize].ptr.cast::<V::Scalar>();

        // 🟢 FIXED: Return standard slices which are completely free of aliasing overhead 
        // and allow LLVM to generate optimized assembly instructions
        unsafe {
            (
                slice::from_raw_parts_mut(pos_base, len),
                slice::from_raw_parts_mut(old_base, len),
                slice::from_raw_parts_mut(acc_base, len),
                slice::from_raw_parts_mut(rad_base, len),
                slice::from_raw_parts_mut(mass_base, len),
            )
        }
    }
}