use std::{alloc::Layout, ptr, slice};

use crate::{math::{FloatScalar, Vector}, sim::{solver::{particle::{data_layout::ParticleDataLayout, verlet_particle::VerletParticle}, 
    soa_vec_storage::{ErgonomicSoaLayout,SoaVecStorage}}, storage::{RawColumn, SoaCpuStorage, SoaLayout, Storage}}, ui::layout::color::Color};
 
pub type VerletParticleSoaVecStorage<V> = SoaVecStorage<VerletParticle<V>>;
 
impl<V: Vector + 'static> ParticleDataLayout<V> for VerletParticleSoaVecStorage<V> {
    #[inline(always)]
    fn len(&self) -> usize {
        Storage::len(self)
    }

    #[inline(always)]
    fn radii(&self) -> &[V::Scalar] {
        unsafe { self.slice_uncheck(VerletCol::Radius) }
    }

    #[inline(always)]
    fn positions_mut(&mut self) -> &mut [V] {
        unsafe { self.slice_mut_uncheck(VerletCol::Pos) }
    }

    #[inline(always)]
    fn positions_and_old_mut(&mut self) -> (&mut [V], &mut [V]) {
        let len = Storage::len(self);
        unsafe {
            // Re-using the safe raw-pointer extraction pattern
            let pos_ptr = self.columns()[VerletCol::Pos as usize].ptr.cast::<V>();
            let old_ptr = self.columns()[VerletCol::PosOld as usize].ptr.cast::<V>();
            (
                slice::from_raw_parts_mut(pos_ptr, len),
                slice::from_raw_parts_mut(old_ptr, len),
            )
        }
    }

    fn commit_kinetics(&mut self, max_vel_squared: V::Scalar, sub_step_max_vel: V::Scalar) {
        let (pos_slice, old_slice) = self.positions_and_old_mut();
        for (pos, pos_old) in pos_slice.iter_mut().zip(old_slice.iter_mut()) {
            let vel = *pos - *pos_old;
            let vel_sq = vel.length_squared();
            if vel_sq > max_vel_squared {
                *pos_old = *pos - (vel * (sub_step_max_vel / vel_sq.sqrt()));
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum VerletCol {
    Pos = 0,
    PosOld = 1,
    Acc = 2,
    Radius = 3,
    Color = 4,
}

impl crate::sim::solver::soa_vec_storage::SoaProperty for VerletCol {
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
    ];

    #[inline]
    unsafe fn push_cols(&self, cols: &mut [RawColumn], index: usize) {
        unsafe {
            ptr::write(cols[VerletCol::Pos as usize].ptr.cast::<V>().add(index), self.pos);
            ptr::write(cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(index), self.pos_old);
            ptr::write(cols[VerletCol::Acc as usize].ptr.cast::<V>().add(index), self.acc);
            ptr::write(cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(index), self.radius);
            ptr::write(cols[VerletCol::Color as usize].ptr.cast::<Color>().add(index), self.color);
        }
    }

    #[inline]
    unsafe fn read_cols(cols: &[RawColumn], index: usize) -> Self {
        unsafe {
            Self {
                pos:     ptr::read(cols[VerletCol::Pos as usize].ptr.cast::<V>().add(index)),
                pos_old: ptr::read(cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(index)),
                acc:     ptr::read(cols[VerletCol::Acc as usize].ptr.cast::<V>().add(index)),
                radius:  ptr::read(cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(index)),
                color:   ptr::read(cols[VerletCol::Color as usize].ptr.cast::<Color>().add(index)),
            }
        }
    }

    #[inline]
    unsafe fn swap_remove_cols(cols: &mut [RawColumn], index: usize, current_len: usize) {
        let tail_index = current_len - 1;
        if index == tail_index { return; }
        
        unsafe {
            ptr::copy_nonoverlapping(cols[VerletCol::Pos as usize].ptr.cast::<V>().add(tail_index), cols[VerletCol::Pos as usize].ptr.cast::<V>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(tail_index), cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletCol::Acc as usize].ptr.cast::<V>().add(tail_index), cols[VerletCol::Acc as usize].ptr.cast::<V>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(tail_index), cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(index), 1);
            ptr::copy_nonoverlapping(cols[VerletCol::Color as usize].ptr.cast::<Color>().add(tail_index), cols[VerletCol::Color as usize].ptr.cast::<Color>().add(index), 1);
        }
    }
}

unsafe impl<V> ErgonomicSoaLayout for VerletParticle<V>
where
    V: Vector + 'static,
    V::Scalar: 'static,
{
    type Property = VerletCol;
}