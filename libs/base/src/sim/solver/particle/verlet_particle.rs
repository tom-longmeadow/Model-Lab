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
    pub inv_mass: V::Scalar,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum VerletParticleColumns {
    Pos = 0,
    PosOld = 1,
    Acc = 2,
    Radius = 3,
    Color = 4,
    InvMass = 5,
}
 

impl<V> VerletParticle<V> 
where 
    V: Vector
{
    pub const STRIDE: u64 = std::mem::size_of::<Self>() as u64;

    #[inline]
    pub fn new(pos: V) -> Self { 
        Self {
            pos,  
            pos_old: pos,
            acc: V::ZERO,
            radius: V::Scalar::ONE,
            color: Color::WHITE,
            inv_mass: V::Scalar::ONE,
        }
    }

    #[inline]
    pub fn with_velocity(mut self, vel: V) -> Self {
        // Correctly uses your Vector trait Sub optimization bound
        self.pos_old = self.pos - vel;
        self
    }

    #[inline]
    pub fn with_radius(mut self, radius: V::Scalar, density: V::Scalar) -> Self {
        self.radius = radius;
 
        let mass = radius * radius * density; 
        let inv_mass = if mass > V::Scalar::ZERO {
            V::Scalar::ONE / mass
        } else {
            V::Scalar::ZERO // Infinite mass (immovable object)
        };

        self.inv_mass = inv_mass; 
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
        VerletParticle::new(V::ZERO)
    }
}