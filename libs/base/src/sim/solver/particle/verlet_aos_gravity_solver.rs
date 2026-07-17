 
use crate::math::{FloatScalar, Vector}; 
use crate::sim::solver::Solver;
use crate::sim::solver::particle::environment::ParticleEnvironment; 
use crate::sim::solver::particle::verlet_aos_vec_storage::{VerletParticleAosVecStorage};
use crate::sim::solver::particle::verlet_physics::{VerletPhysics};
use crate::sim::storage::{AosCpuStorage};
use crate::prelude::solver::particle::space::collision_registry::CollisionRegistry;

pub struct VerletAosGravitySolver
{  
    pub registry: CollisionRegistry, 
}

impl VerletAosGravitySolver {
    pub fn new(initial_capacity: usize) -> Self { 
        Self { 
            registry: CollisionRegistry::with_capacity(initial_capacity), 
        }
    }
}

impl<V: Vector + 'static> Solver<VerletParticleAosVecStorage<V>, ParticleEnvironment<V>> for VerletAosGravitySolver
where
    V::Quantized: std::hash::Hash + Eq + Copy,
{
    fn init(&mut self, _storage: &mut VerletParticleAosVecStorage<V>, _environment: &mut ParticleEnvironment<V>) { }

    fn pre_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>, _dt: f64, _tick: u64, environment: &mut ParticleEnvironment<V>) {
        environment.state.update_jitter(_tick);
        
        let particles = storage.as_slice();
        if particles.is_empty() { return; }

        // Scan radii to update the broadphase grid sizing dynamically
        type S<V> = <V as Vector>::Scalar; 
        let mut min_radius = S::<V>::INFINITY;
        let mut max_radius = S::<V>::NEG_INFINITY;

        for p in particles.iter() {
            if p.radius < min_radius { min_radius = p.radius; }
            if p.radius > max_radius { max_radius = p.radius; }
        }

        environment.space.grid.set_cell_size(max_radius);  
        environment.tuning.update_physics(min_radius, max_radius);
    }
    
    fn sub_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>, dt: f64, environment: &ParticleEnvironment<V>) {
        let particles = storage.as_slice_mut();
        if particles.is_empty() { return; }

        let sub_step_dt = <V::Scalar as FloatScalar>::from_f64(dt);
        let gravity_acc: V = environment.gravity.get(); 

        // 1. APPLY FORCE INTEGRATION (Gravity acceleration mapping)
        for p in particles.iter_mut() {
            p.acc = p.acc + gravity_acc;
        }

        // 2. EXECUTED PIPELINE DELEGATION
        // Safe, clean wrappers call into encapsulated VerletPhysics workers
        unsafe {
            // A. Update positions and velocities with your damping factors
            VerletPhysics::aos_update_kinetics(particles, sub_step_dt, environment);

            // B. Narrowphase: Broadphase scanning + internal dynamic constraint loop execution
             VerletPhysics::aos_detect_and_resolve_collisions(
                &environment.space.grid, 
                particles, 
                &mut self.registry, 
                environment
            );

            // C. Bounded Wall Environments & Open Air Slide Friction Dampening
             VerletPhysics::aos_apply_position_constraints(particles, sub_step_dt, environment);

            // D. Post-Loop Velocity History Restitution Bouncing
             VerletPhysics::aos_apply_particle_restitution(&self.registry, particles, environment);
        }
    }
    
    fn post_step(&mut self, _storage: &mut VerletParticleAosVecStorage<V>, _dt: f64, _environment: &ParticleEnvironment<V>) { }
}
 