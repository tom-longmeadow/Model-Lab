use crate::{aabb::AABB, math::{Vector}, ui::layout::color::Color};
use crate::math::FloatScalar;

pub struct StreamConfig<V: Vector> {
    pub start_tick: u64,
    pub ticks_per_spawn: u64,
    pub particle_count: usize,
    pub max_particles: usize, 
    pub relative_position: V,
    pub velocity: V,
    pub radius: V::Scalar,
    pub density: V::Scalar,
    pub colors: &'static [Color], 
}

impl<V: Vector> StreamConfig<V> {
    pub fn new(
        start_tick: u64, 
        ticks_per_spawn: u64,  
        max_particles: usize, 
        relative_position: V, 
        velocity: V, 
        radius: V::Scalar, 
        colors: &'static [Color]
    ) -> Self {
        Self { 
            start_tick, 
            ticks_per_spawn, 
            particle_count: 0, 
            max_particles, 
            relative_position, 
            velocity, 
            radius, 
            density: V::Scalar::ONE,
            colors 
        }
    }

    /// Helper to evaluate if a particle should spawn on this tick
    pub fn should_spawn(&self, tick: u64) -> bool {
        self.particle_count < self.max_particles &&
            tick >= self.start_tick && 
            tick % self.ticks_per_spawn == 0
    }

    pub fn get_color(&self) -> Color{
        let percentage = self.particle_count as f32 / (self.max_particles - 1) as f32;
        Color::get_color_at_percentage(&self.colors, percentage) 
    } 

      pub fn get_spawn_position(&self, bounds: &AABB<V>) -> V { 
        let size = bounds.size(); 
        bounds.min.clone() + self.relative_position.clone().mul_elementwise(size)
    }
}
