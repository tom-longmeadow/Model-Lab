use crate::{math::Vector, sim::{lifecycle::{Lifecycle, stream_config::StreamConfig}, 
solver::particle::{environment::ParticleEnvironment, verlet_aos_vec_storage::VerletParticleAosVecStorage, 
    verlet_particle::VerletParticle}, storage::AosCpuStorage}};
 
 

pub struct AosStreamLifecycle<V: Vector> {
    pub config: StreamConfig<V>,
}

impl<V: Vector> AosStreamLifecycle<V> {
    pub fn new(config: StreamConfig<V>) -> Self {
        Self { config }
    }
}

impl<V: Vector> Lifecycle<VerletParticleAosVecStorage<V>, ParticleEnvironment<V>> for AosStreamLifecycle<V> 
where
    V: std::ops::Sub<Output = V>, 
{
    fn tick(&mut self, storage: &mut VerletParticleAosVecStorage<V>, tick: u64, environment: &ParticleEnvironment<V>) {
        
        if self.config.should_spawn(tick) {
            let position = self.config.get_spawn_position(&environment.space.bounds);
            let color = self.config.get_color();
             
            let p = VerletParticle::new(position)
                .with_velocity(self.config.velocity.clone()) 
                .with_radius(self.config.radius.clone()) 
                .with_color(color);
             
            storage.push(p);
            self.config.particle_count += 1;
        } 
    } 
}