use std::{alloc::Layout, ptr};

use crate::{math::Vector, sim::{solver::{particle::verlet_particle::{VerletParticle}, 
soa_vec_storage::SoaVecStorage}, storage::{SoaColumn, SoaLayout, SoaProperty}}, ui::layout::color::Color};


pub type VerletParticleSoaVecStorage<V> = SoaVecStorage<VerletParticle<V>>;
   
unsafe impl<V> SoaLayout for VerletParticle<V> 
where 
    V: Vector + 'static,
    V::Scalar: 'static,
{
    // FIXED: Swapped dynamic slice reference out for an exact-size fixed array expression.
    // This allows the compiler to treat the sizes and counts as absolute compile-time constants.
    const LAYOUTS: &'static [Layout] = {
        // Enforced inside a strict const evaluation block
        &[
            Layout::new::<V>(),          // Slot 0: Pos
            Layout::new::<V>(),          // Slot 1: PosOld
            Layout::new::<V>(),          // Slot 2: Acc
            Layout::new::<V::Scalar>(),  // Slot 3: Radius
            Layout::new::<Color>(),      // Slot 4: Color
            Layout::new::<V::Scalar>(),  // Slot 5: InvMass
        ]
    };

    #[inline(always)]
    unsafe fn push_cols(item: Self, cols: &mut [SoaColumn], index: usize) {
        // FIXED: Upfront exact-size slicing explicitly tells LLVM that the columns array 
        // contains at least 6 elements, completely eliminating runtime bounds checks inside this function.
        let hot_cols = unsafe { cols.get_unchecked(0..6) };

        unsafe {
            ptr::write(hot_cols[0].ptr.as_ptr().cast::<V>().add(index), item.pos);
            ptr::write(hot_cols[1].ptr.as_ptr().cast::<V>().add(index), item.pos_old);
            ptr::write(hot_cols[2].ptr.as_ptr().cast::<V>().add(index), item.acc);
            ptr::write(hot_cols[3].ptr.as_ptr().cast::<V::Scalar>().add(index), item.radius);
            ptr::write(hot_cols[4].ptr.as_ptr().cast::<Color>().add(index), item.color);
            ptr::write(hot_cols[5].ptr.as_ptr().cast::<V::Scalar>().add(index), item.inv_mass);
        }
    }

    #[inline(always)]
    unsafe fn read_cols(cols: &[SoaColumn], index: usize) -> Self {
        // FIXED: Upfront exact-size slicing explicitly tells LLVM that the columns array 
        // contains at least 6 elements, completely eliminating runtime bounds checks inside this function.
        let hot_cols = unsafe { cols.get_unchecked(0..6) };

        unsafe {
            Self {
                pos:      ptr::read(hot_cols[0].ptr.as_ptr().cast::<V>().add(index)),
                pos_old:  ptr::read(hot_cols[1].ptr.as_ptr().cast::<V>().add(index)),
                acc:      ptr::read(hot_cols[2].ptr.as_ptr().cast::<V>().add(index)),
                radius:   ptr::read(hot_cols[3].ptr.as_ptr().cast::<V::Scalar>().add(index)),
                color:    ptr::read(hot_cols[4].ptr.as_ptr().cast::<Color>().add(index)),
                inv_mass: ptr::read(hot_cols[5].ptr.as_ptr().cast::<V::Scalar>().add(index)),
            }
        }
    }

    #[inline(always)]
    unsafe fn drop_cols(cols: &[SoaColumn], index: usize) {
        // FIXED: Upfront exact-size slicing explicitly tells LLVM that the columns array 
        // contains at least 6 elements, completely eliminating runtime bounds checks inside this function.
        let hot_cols = unsafe { cols.get_unchecked(0..6) };

        unsafe {
            ptr::drop_in_place(hot_cols[0].ptr.as_ptr().cast::<V>().add(index));
            ptr::drop_in_place(hot_cols[1].ptr.as_ptr().cast::<V>().add(index));
            ptr::drop_in_place(hot_cols[2].ptr.as_ptr().cast::<V>().add(index));
            ptr::drop_in_place(hot_cols[3].ptr.as_ptr().cast::<V::Scalar>().add(index));
            ptr::drop_in_place(hot_cols[4].ptr.as_ptr().cast::<Color>().add(index));
            ptr::drop_in_place(hot_cols[5].ptr.as_ptr().cast::<V::Scalar>().add(index));
        }
    }
}

// =========================================================================
// ZERO-COST STRUCTURAL PROPERTY DEFINITIONS
// =========================================================================

pub struct SoaPos;
pub struct SoaPosOld;
pub struct AccField;
pub struct SoaRadius;
pub struct SoaColor;
pub struct SoaInvMass;

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaPos {
    type Type = V;
    const INDEX: usize = 0; // Inlined direct integer literal values
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaPosOld {
    type Type = V;
    const INDEX: usize = 1; // Inlined direct integer literal values
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for AccField {
    type Type = V;
    const INDEX: usize = 2; // Inlined direct integer literal values
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaRadius {
    type Type = V::Scalar;
    const INDEX: usize = 3; // Inlined direct integer literal values
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaColor {
    type Type = Color;
    const INDEX: usize = 4; // Inlined direct integer literal values
}

impl<V: Vector + 'static> SoaProperty<VerletParticle<V>> for SoaInvMass {
    type Type = V::Scalar;
    const INDEX: usize = 5; // Inlined direct integer literal values
}