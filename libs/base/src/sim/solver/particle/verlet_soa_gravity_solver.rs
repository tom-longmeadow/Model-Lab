use std::hash::Hash; 
use crate::math::{FloatScalar, Vector}; 
use crate::sim::solver::Solver; 
use crate::sim::solver::particle::environment::ParticleEnvironment; 
use crate::sim::solver::particle::space::collision_registry::CollisionRegistry;
use crate::sim::solver::particle::verlet_particle::VerletParticleColumns;
use crate::sim::solver::particle::verlet_physics::{ VerletPhysics};
use crate::sim::solver::particle::verlet_soa_vec_storage::VerletParticleSoaVecStorage; 
use crate::sim::storage::{Storage}; 
use crate::sim::solver::particle::verlet_soa_vec_storage::ErgonomicSoaCpuStorageExt;
pub struct VerletSoaGravitySolver {  
    // 🟢 Decoupled registry index pair cache matching the AoS design
    pub registry: CollisionRegistry,
}

impl VerletSoaGravitySolver {
    pub fn new(initial_capacity: usize) -> Self { 
        Self { 
            registry: CollisionRegistry::with_capacity(initial_capacity), 
        }
    }
}

impl<V: Vector + 'static> Solver<VerletParticleSoaVecStorage<V>, ParticleEnvironment<V>> for VerletSoaGravitySolver
where
    V::Quantized: Hash + Eq + Copy,
{
    fn init(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>, _environment: &mut ParticleEnvironment<V>) { }

    fn pre_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, _dt: f64, _tick: u64, environment: &mut ParticleEnvironment<V>) {
        environment.state.update_jitter(_tick);
        
        let len = Storage::len(storage);
        if len == 0 { return; }

        let radii = storage.as_slice(VerletParticleColumns::Radius);

        type S<V> = <V as Vector>::Scalar; 
        let mut min_radius = S::<V>::INFINITY;
        let mut max_radius = S::<V>::NEG_INFINITY;

        for &r in radii.iter() {
            if r < min_radius { min_radius = r; }
            if r > max_radius { max_radius = r; }
        }

        environment.space.grid.set_cell_size(max_radius);  
        environment.tuning.update_physics(min_radius, max_radius);
    }
    
    fn sub_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, dt: f64, environment: &ParticleEnvironment<V>) {
        let len = Storage::len(storage);
        if len == 0 { return; }

        let sub_step_dt = <V::Scalar as FloatScalar>::from_f64(dt);
        
        // Extract raw individual scalar components out of the master gravity force vector
        let gravity_x = environment.gravity.get().component(0);
        let gravity_y = environment.gravity.get().component(1);

        // 🟢 FIXED: Unpack the layout into 8 distinct scalar component strided fields!
        // This completely matches what your VerletPhysics::soa_* workers expect.
        let (pos_x, pos_y, old_x, old_y, acc_x, acc_y, radii, inv_masses) = 
            storage.get_physics_components_mut::<V>();

        // 1. APPLY FORCE INTEGRATION (Gravity components injected straight into memory lanes)
        for i in 0..len {
            unsafe {
                acc_x.set_unchecked(i, acc_x.get_unchecked(i) + gravity_x);
                acc_y.set_unchecked(i, acc_y.get_unchecked(i) + gravity_y);
            }
        }

        // 2. EXECUTED PIPELINE DELEGATION
        unsafe {
            // A. Updated Kinetics pass (Takes 8 arguments)
            VerletPhysics::soa_update_kinetics(
                &pos_x, &pos_y, 
                &old_x, &old_y, 
                &acc_x, &acc_y, 
                sub_step_dt, environment
            );

            // B. Narrowphase spatial grid collision checks (Takes 7 arguments)
            VerletPhysics::soa_detect_and_resolve_collisions(
                &environment.space.grid, 
                &pos_x, &pos_y, 
                inv_masses, 
                radii, 
                &mut self.registry, 
                environment
            );

            // C. Bounded wall limits and containment boundaries (Takes 7 arguments)
            VerletPhysics::soa_apply_position_constraints(
                &pos_x, &pos_y, 
                &old_x, &old_y, 
                radii, 
                sub_step_dt, environment
            );

            // D. Restitution bounce history register writes (Takes 8 arguments)
            VerletPhysics::soa_apply_particle_restitution(
                &self.registry, 
                &pos_x, &pos_y, 
                &old_x, &old_y, 
                inv_masses, 
                radii, environment
            );
        }
    }
    
    fn post_step(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>, _dt: f64, _environment: &ParticleEnvironment<V>) { }
}
  
