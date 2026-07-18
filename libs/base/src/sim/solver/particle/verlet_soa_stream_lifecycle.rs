use crate::{math::Vector, sim::{lifecycle::{Lifecycle, stream_config::StreamConfig}, 
solver::particle::{environment::ParticleEnvironment, 
    lifecycle::{tick_stream_lifecycle}, verlet_soa_vec_storage::VerletParticleSoaVecStorage}}
};
 
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
    V: Vector + std::ops::Sub<Output = V> + 'static,  
    V::Scalar: 'static,
{
    fn tick(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, tick: u64, step_dt: f64, environment: &ParticleEnvironment<V>) {
        tick_stream_lifecycle(&mut self.config, storage, tick, step_dt, environment); 
    } 
}
