use std::hash::Hash;
use crate::math::{FloatScalar, Vector}; 
use crate::sim::solver::Solver;
use crate::sim::solver::particle::data_layout::ParticleDataLayout;
use crate::sim::solver::particle::environment::ParticleEnvironment; 
use crate::sim::solver::particle::verlet_physics::{VerletCoreEngine, VerletPhysics};
use crate::sim::solver::particle::verlet_soa_vec_storage::VerletParticleSoaVecStorage;
use crate::sim::storage::{Storage};

pub struct VerletSoaGravitySolver<V> 
where 
    V: Vector,
    V::Quantized: Hash + Eq, 
{ 
    // Kept ONLY for caching broadphase and grid cell sizing metrics
    pub scratch_radii: Vec<V::Scalar>, 
}

impl<V> VerletSoaGravitySolver<V>
where 
    V: Vector, 
    V::Quantized: Hash + Eq,
{
    pub fn new(initial_capacity: usize) -> Self { 
        Self { 
            // Pos and PosOld scratches are completely gone!
            scratch_radii: Vec::with_capacity(initial_capacity),
        }
    }
}
impl<V: Vector + 'static> Solver<VerletParticleSoaVecStorage<V>, ParticleEnvironment<V>> for VerletSoaGravitySolver<V>
where
    V::Quantized: std::hash::Hash + Eq,
{
    fn init(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>, _environment: &mut ParticleEnvironment<V>) { }

    fn pre_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, _dt: f64, _tick: u64, environment: &mut ParticleEnvironment<V>) {
        environment.state.update_jitter(_tick);
        
        let len = Storage::len(storage);
        if len != self.scratch_radii.len() {
            self.scratch_radii.clear();

            type S<V> = <V as Vector>::Scalar; 
            let mut min_radius = S::<V>::INFINITY;
            let mut max_radius = S::<V>::NEG_INFINITY;

            // Zero-overhead immutable reference straight to the Radius column memory
            let radius_slice = storage.radii();

            for &radius in radius_slice.iter() {
                self.scratch_radii.push(radius);

                if radius < min_radius { min_radius = radius; }
                if radius > max_radius { max_radius = radius; }
            }

            environment.space.grid.set_cell_size(max_radius);  
            environment.tuning.update_physics(min_radius, max_radius);
        }
    }
    
    fn sub_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, dt: f64, environment: &ParticleEnvironment<V>) {
        let len = Storage::len(storage);
        if len == 0 { return; }

        let sub_step_dt = <V::Scalar as FloatScalar>::from_f64(dt);
        let gravity_acc: V = environment.gravity.get(); 

        // 1. KINETICS PASS (Completely In-Place via unified slice tuple extraction)
        let (pos_slice, old_slice) = storage.positions_and_old_mut();
        
        for (pos, pos_old) in pos_slice.iter_mut().zip(old_slice.iter_mut()) {
            let mut acc = gravity_acc;
            VerletPhysics::update_kinetics(sub_step_dt, environment, pos, pos_old, &mut acc);
        }
 
        VerletCoreEngine::execute_sub_step(storage, dt, environment, &self.scratch_radii);
    }
    
    fn post_step(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>, _dt: f64, _environment: &ParticleEnvironment<V>) { }
}