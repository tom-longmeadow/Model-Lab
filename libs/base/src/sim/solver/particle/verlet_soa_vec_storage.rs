use std::{alloc::Layout, ptr};

use crate::{math::Vector, sim::{solver::{particle::verlet_particle::{VerletParticle, VerletParticleColumns}, 
soa_vec_storage::SoaVecStorage}, storage::{SoaColumn, SoaLayout, SoaProperty}}, ui::layout::color::Color};


pub type VerletParticleSoaVecStorage<V> = SoaVecStorage<VerletParticle<V>>;
   
 
unsafe impl<V> SoaLayout for VerletParticle<V> 
where 
    V: Vector + 'static,
    V::Scalar: 'static,
{
    // The static schema mapping column slots to physical byte shapes
    const LAYOUTS: &'static [Layout] = &[
        Layout::new::<V>(),          // Slot 0: Pos
        Layout::new::<V>(),          // Slot 1: PosOld
        Layout::new::<V>(),          // Slot 2: Acc
        Layout::new::<V::Scalar>(),  // Slot 3: Radius
        Layout::new::<Color>(),      // Slot 4: Color
        Layout::new::<V::Scalar>(),  // Slot 5: InvMass
    ];

    #[inline(always)]
    unsafe fn push_cols(item: Self, cols: &mut [SoaColumn], index: usize) {
        unsafe {
            ptr::write(cols[VerletParticleColumns::Pos as usize].ptr.as_ptr().cast::<V>().add(index), item.pos);
            ptr::write(cols[VerletParticleColumns::PosOld as usize].ptr.as_ptr().cast::<V>().add(index), item.pos_old);
            ptr::write(cols[VerletParticleColumns::Acc as usize].ptr.as_ptr().cast::<V>().add(index), item.acc);
            ptr::write(cols[VerletParticleColumns::Radius as usize].ptr.as_ptr().cast::<V::Scalar>().add(index), item.radius);
            ptr::write(cols[VerletParticleColumns::Color as usize].ptr.as_ptr().cast::<Color>().add(index), item.color);
            ptr::write(cols[VerletParticleColumns::InvMass as usize].ptr.as_ptr().cast::<V::Scalar>().add(index), item.inv_mass);
        }
    }

    #[inline(always)]
    unsafe fn read_cols(cols: &[SoaColumn], index: usize) -> Self {
        unsafe {
            Self {
                pos:      ptr::read(cols[VerletParticleColumns::Pos as usize].ptr.as_ptr().cast::<V>().add(index)),
                pos_old:  ptr::read(cols[VerletParticleColumns::PosOld as usize].ptr.as_ptr().cast::<V>().add(index)),
                acc:      ptr::read(cols[VerletParticleColumns::Acc as usize].ptr.as_ptr().cast::<V>().add(index)),
                radius:   ptr::read(cols[VerletParticleColumns::Radius as usize].ptr.as_ptr().cast::<V::Scalar>().add(index)),
                color:    ptr::read(cols[VerletParticleColumns::Color as usize].ptr.as_ptr().cast::<Color>().add(index)),
                inv_mass: ptr::read(cols[VerletParticleColumns::InvMass as usize].ptr.as_ptr().cast::<V::Scalar>().add(index)),
            }
        }
    }

    #[inline(always)]
    unsafe fn drop_cols(cols: &[SoaColumn], index: usize) {
        unsafe {
            ptr::drop_in_place(cols[VerletParticleColumns::Pos as usize].ptr.as_ptr().cast::<V>().add(index));
            ptr::drop_in_place(cols[VerletParticleColumns::PosOld as usize].ptr.as_ptr().cast::<V>().add(index));
            ptr::drop_in_place(cols[VerletParticleColumns::Acc as usize].ptr.as_ptr().cast::<V>().add(index));
            ptr::drop_in_place(cols[VerletParticleColumns::Radius as usize].ptr.as_ptr().cast::<V::Scalar>().add(index));
            ptr::drop_in_place(cols[VerletParticleColumns::Color as usize].ptr.as_ptr().cast::<Color>().add(index));
            ptr::drop_in_place(cols[VerletParticleColumns::InvMass as usize].ptr.as_ptr().cast::<V::Scalar>().add(index));
        }
    }
}

 
pub struct SoaPos;
pub struct SoaPosOld;
pub struct AccField;
pub struct SoaRadius;
pub struct SoaColor;
pub struct SoaInvMass;

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaPos {
    type Type = V;
    const INDEX: usize = VerletParticleColumns::Pos as usize;
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaPosOld {
    type Type = V;
    const INDEX: usize = VerletParticleColumns::PosOld as usize;
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for AccField {
    type Type = V;
    const INDEX: usize = VerletParticleColumns::Acc as usize;
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaRadius {
    type Type = V::Scalar;
    const INDEX: usize = VerletParticleColumns::Radius as usize;
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaColor {
    type Type = Color;
    const INDEX: usize = VerletParticleColumns::Color as usize;
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaInvMass {
    type Type = V::Scalar;
    const INDEX: usize = VerletParticleColumns::InvMass as usize;
}  