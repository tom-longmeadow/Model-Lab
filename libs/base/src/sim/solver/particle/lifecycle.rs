pub mod fountain;
pub mod ball;
use crate::{aabb::AABB, math::Vector};



 
#[derive(Clone, Debug)]
pub struct Stream<V: Vector> {
    // Timing Gates
    start_tick: u64,
    ticks_per_spawn: u64, 
    
    // Core Particle Footprint Configurations
    relative_position: V,
    pub velocity: V,
    pub radius: V::Scalar,
    pub density: V::Scalar, 
   
}


impl<V: Vector> Stream<V> {
    /// Creates a new continuous particle injection stream profile.
    #[inline]
    pub fn new(
        start_tick: u64,
        ticks_per_spawn: u64, 
        relative_position: V,
        velocity: V,
        radius: V::Scalar,
        density: V::Scalar,
    ) -> Self {
        Self {
            start_tick,
            ticks_per_spawn, 
            relative_position,
            velocity,
            radius,
            density,
        }
    }
}

impl<V: Vector> Stream<V> {
    /// Checks the temporal constraint. Returns `true` if this stream should fire.
    #[inline(always)]
    pub fn should_emit(&self, tick: u64) -> bool {
        tick >= self.start_tick && (tick - self.start_tick) % self.ticks_per_spawn == 0
    }
 
   #[inline(always)]
    pub fn get_position(&self, bounds: &AABB<V>) -> V {
        let size = bounds.size(); 
        bounds.min.clone() + self.relative_position.clone().mul_elementwise(size)
    }
 
}


 