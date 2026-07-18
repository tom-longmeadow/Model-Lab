use crate::{math::Vector, sim::{lifecycle::{Lifecycle, stream_config::StreamConfig}, 
solver::particle::{environment::ParticleEnvironment, 
    lifecycle::{tick_stream_lifecycle}, verlet_aos_vec_storage::VerletParticleAosVecStorage}}
};
 
 

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
     V: Vector + std::ops::Sub<Output = V> + 'static,  
    V::Scalar: 'static,
{
    fn tick(&mut self, storage: &mut VerletParticleAosVecStorage<V>, tick: u64, step_dt: f64, environment: &ParticleEnvironment<V>) {
        
        tick_stream_lifecycle(&mut self.config, storage, tick, step_dt, environment); 
    } 
}

 