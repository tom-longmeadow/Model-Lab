use crate::{math::{FloatScalar, Vector}, ui::layout::color::Color};

#[repr(C)]
#[derive(Debug, Clone, Copy)] 
pub struct VerletParticle<V> 
where 
    V: Vector
{
    pub pos:     V,
    pub pos_old: V,
    pub acc:     V,
    pub radius:  V::Scalar,
    pub color:   Color,
}

impl<V> VerletParticle<V> 
where 
    V: Vector,
    VerletParticle<V>: Sized, // Ensures size can be computed
{
    pub const STRIDE: u64 = std::mem::size_of::<Self>() as u64;
}

impl<V> VerletParticle<V> 
where 
    V: Vector
{
    #[inline]
    pub fn new(pos: V) -> Self {
        Self {
            pos,  
            pos_old: pos,
            acc: V::ZERO,
            radius: <V::Scalar as FloatScalar>::ZERO,
            color: Color::WHITE,
        }
    }

    #[inline]
    pub fn with_velocity(mut self, vel: V) -> Self {
        // Correctly uses your Vector trait Sub optimization bound
        self.pos_old = self.pos - vel;
        self
    }

    #[inline]
    pub fn with_radius(mut self, radius: V::Scalar) -> Self {
        self.radius = radius;
        self
    }

    #[inline]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl<V> Default for VerletParticle<V> 
where 
    V: Vector
{
    #[inline]
    fn default() -> Self {
        Self {
            pos: V::ZERO,
            pos_old: V::ZERO,
            acc: V::ZERO,
            radius: <V::Scalar as FloatScalar>::ZERO,
            color: Color::WHITE,
        }
    }
}