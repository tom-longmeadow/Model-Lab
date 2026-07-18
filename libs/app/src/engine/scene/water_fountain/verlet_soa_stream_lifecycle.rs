use base::{math::Vector, sim::{lifecycle::{Lifecycle, stream_config::StreamConfig}, 
solver::particle::{environment::ParticleEnvironment, 
    verlet_soa_vec_storage::VerletParticleSoaVecStorage}}};

use crate::engine::scene::water_fountain::scene_config::water_fountain_lifcycle_tick;

 
 
pub struct WaterFountainSoaStreamLifecycle<V: Vector> {
    pub config: StreamConfig<V>,
}

impl<V: Vector> WaterFountainSoaStreamLifecycle<V> {
    pub fn new(config: StreamConfig<V>) -> Self {
        Self { config }
    }
}

impl<V> Lifecycle<VerletParticleSoaVecStorage<V>, ParticleEnvironment<V>> for WaterFountainSoaStreamLifecycle<V> 
where
    V: Vector + std::ops::Sub<Output = V> + 'static,  
    V::Scalar: 'static,
{
    fn tick(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, tick: u64, step_dt: f64, environment: &ParticleEnvironment<V>) {
        water_fountain_lifcycle_tick(&mut self.config, storage, tick, step_dt, environment); 
    } 
}

 

