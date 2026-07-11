use base::{math::{Bounds, DVec2, FloatExt}, sim::{lifecycle::Lifecycle, storage::AosCpuStorage}, ui::layout::color::Color};

use crate::simulation::verlet_2d::{particle::Particle, aos_vec_storage::AosVecStorage};


pub struct StreamLifecycle {
    start_tick: u64,
    ticks_per_spawn: u64,
    particle_count: usize,
    max_particles: usize, 
    velocity: DVec2,
    radius: f64,
    color: Color,
}

impl StreamLifecycle {
    pub fn new(start_tick: u64, ticks_per_spawn: u64,  max_particles: usize, 
        velocity: DVec2, radius: f64,  color: Color) -> Self {
        Self { start_tick, ticks_per_spawn, particle_count: 0, max_particles,  velocity, radius, color}
    }
}

impl Lifecycle<AosVecStorage> for StreamLifecycle {
    fn tick(&mut self, storage: &mut AosVecStorage, tick: u64, bounds: &Bounds) {

        if self.particle_count < self.max_particles &&
            tick >= self.start_tick && 
            tick % self.ticks_per_spawn == 0 
        {
            let x = bounds.min.x.lerp(bounds.max.x, 0.25);
            let y = bounds.max.y;

            let position = DVec2::new(x,y);
            let p = Particle::new(position)
                                .with_velocity(self.velocity)
                                .with_radius(self.radius)
                                .with_color(self.color);
             
            storage.push(p);
            self.particle_count += 1;
        } 
    }
}