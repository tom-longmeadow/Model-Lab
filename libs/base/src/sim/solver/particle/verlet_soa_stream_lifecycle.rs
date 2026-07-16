use crate::{math::Vector, sim::{lifecycle::{Lifecycle, stream_config::StreamConfig}, 
solver::particle::{environment::ParticleEnvironment,
    verlet_particle::VerletParticle, verlet_soa_vec_storage::VerletParticleSoaVecStorage}, storage::CpuStorage}};
 
 pub struct SoaStreamLifecycle<V: Vector> {
    pub config: StreamConfig<V>,
}

impl<V: Vector> SoaStreamLifecycle<V> {
    pub fn new(config: StreamConfig<V>) -> Self {
        Self { config }
    }
}

impl<V> Lifecycle<VerletParticleSoaVecStorage<V>, ParticleEnvironment<V>> for SoaStreamLifecycle<V> 
where
    V: Vector + 'static,
    V::Scalar: 'static,
{
    fn tick(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, tick: u64, environment: &ParticleEnvironment<V>) {
        if self.config.should_spawn(tick) {
            let position = self.config.get_spawn_position(&environment.space.bounds);
            let color = self.config.get_color();
             
            // Constructs the logical item layout using the builder methods
            let p = VerletParticle::new(position)
                .with_velocity(self.config.velocity.clone()) 
                .with_radius(self.config.radius.clone()) 
                .with_color(color);
             
            // Appends the entity directly into the columns using our custom SoA vector storage implementation
            storage.push(p);
            self.config.particle_count += 1;
        } 
    } 
}