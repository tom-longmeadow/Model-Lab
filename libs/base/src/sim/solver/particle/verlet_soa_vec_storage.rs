use std::{alloc::Layout, ptr};

use crate::{math::Vector, sim::{solver::{particle::verlet_particle::VerletParticle, 
    soa_vec_storage::SoaVecStorage}, storage::{RawColumn, SoaLayout}}, ui::layout::color::Color};
 
pub type VerletParticleSoaVecStorage<V> = SoaVecStorage<VerletParticle<V>>;

#[repr(usize)]
enum VerletCol {
    Pos = 0,
    PosOld = 1,
    Acc = 2,
    Radius = 3,
    Color = 4,
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
        ptr::write(cols[VerletCol::Pos as usize].ptr.cast::<V>().add(index), self.pos);
        ptr::write(cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(index), self.pos_old);
        ptr::write(cols[VerletCol::Acc as usize].ptr.cast::<V>().add(index), self.acc);
        ptr::write(cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(index), self.radius);
        ptr::write(cols[VerletCol::Color as usize].ptr.cast::<Color>().add(index), self.color);
    }

    #[inline]
    unsafe fn read_cols(cols: &[RawColumn], index: usize) -> Self {
        Self {
            pos:     ptr::read(cols[VerletCol::Pos as usize].ptr.cast::<V>().add(index)),
            pos_old: ptr::read(cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(index)),
            acc:     ptr::read(cols[VerletCol::Acc as usize].ptr.cast::<V>().add(index)),
            radius:  ptr::read(cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(index)),
            color:   ptr::read(cols[VerletCol::Color as usize].ptr.cast::<Color>().add(index)),
        }
    }

    #[inline]
    unsafe fn swap_remove_cols(cols: &mut [RawColumn], index: usize, current_len: usize) {
        let tail_index = current_len - 1;
        if index == tail_index { return; }
        
        ptr::copy_nonoverlapping(cols[VerletCol::Pos as usize].ptr.cast::<V>().add(tail_index), cols[VerletCol::Pos as usize].ptr.cast::<V>().add(index), 1);
        ptr::copy_nonoverlapping(cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(tail_index), cols[VerletCol::PosOld as usize].ptr.cast::<V>().add(index), 1);
        ptr::copy_nonoverlapping(cols[VerletCol::Acc as usize].ptr.cast::<V>().add(tail_index), cols[VerletCol::Acc as usize].ptr.cast::<V>().add(index), 1);
        ptr::copy_nonoverlapping(cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(tail_index), cols[VerletCol::Radius as usize].ptr.cast::<V::Scalar>().add(index), 1);
        ptr::copy_nonoverlapping(cols[VerletCol::Color as usize].ptr.cast::<Color>().add(tail_index), cols[VerletCol::Color as usize].ptr.cast::<Color>().add(index), 1);
    }
}