use crate::{aabb::AABB,  math::Vector, sim::{lifecycle::{Lifecycle, stream_config::StreamConfig}, 
solver::{particle::{verlet_aos_vec_storage::VerletParticleAosVecStorage, verlet_particle::VerletParticle}}, storage::AosCpuStorage}};
 
 

pub struct AosStreamLifecycle<V: Vector> {
    pub config: StreamConfig<V>,
}

impl<V: Vector> AosStreamLifecycle<V> {
    pub fn new(config: StreamConfig<V>) -> Self {
        Self { config }
    }
}

impl<V: Vector> Lifecycle<VerletParticleAosVecStorage<V>> for AosStreamLifecycle<V> 
where
    V: std::ops::Sub<Output = V>, 
{
    // Fixes the trait mismatch error by providing the concrete type
    type Bounds = AABB<V>;

    fn tick(&mut self, storage: &mut VerletParticleAosVecStorage<V>, tick: u64, bounds: &Self::Bounds) {
        if self.config.should_spawn(tick) {
            let position = self.config.get_spawn_position(bounds);
            let color = self.config.get_color();
            
            // Your builder API works perfectly here since copy/clone is handled
            let p = VerletParticle::new(position)
                .with_velocity(self.config.velocity.clone()) 
                .with_radius(self.config.radius.clone()) 
                .with_color(color);
             
            storage.push(p);
            self.config.particle_count += 1;
        } 
    }
}