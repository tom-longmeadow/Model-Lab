 
use crate::math::{FloatScalar, Vector}; 
use crate::sim::solver::Solver;
use crate::sim::solver::particle::environment::ParticleEnvironment; 
use crate::sim::solver::particle::space::grid::UniformGrid;
use crate::sim::solver::particle::verlet_aos_vec_storage::{VerletParticleAosVecStorage};
use crate::sim::solver::particle::verlet_physics::{VerletPhysics};
use crate::sim::storage::{AosCpuStorage};
use crate::prelude::solver::particle::space::collision_registry::CollisionRegistry;
use std::hash::Hash;

pub struct VerletAosGravitySolver<V> 
where 
    V: Vector,
    V::Quantized: Hash + Eq + Copy, 
{  
    pub grid: UniformGrid<V>,        // 🟢 OWNED INSTANCE: Eliminates borrow-checker conflicts!
    pub registry: CollisionRegistry, 
}

impl<V> VerletAosGravitySolver<V>
where 
    V: Vector,
    V::Quantized: Hash + Eq + Copy, 
{
    pub fn new(initial_capacity: usize, default_cell_size: V::Scalar) -> Self { 
        Self { 
            grid: UniformGrid::new(default_cell_size),
            registry: CollisionRegistry::with_capacity(initial_capacity), 
        }
    }
}

impl<V: Vector + 'static> Solver<VerletParticleAosVecStorage<V>, ParticleEnvironment<V>> for VerletAosGravitySolver<V>
where
    V::Quantized: std::hash::Hash + Eq + Copy,
{
    fn init(&mut self, _storage: &mut VerletParticleAosVecStorage<V>, _environment: &mut ParticleEnvironment<V>) { }

    fn pre_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>, _tick: u64, environment: &mut ParticleEnvironment<V>) {
        environment.state.update_jitter(_tick);
        
        let particles = storage.as_slice();
        if particles.is_empty() { return; }

        type S<V> = <V as Vector>::Scalar; 
        let mut min_radius = S::<V>::INFINITY;
        let mut max_radius = S::<V>::NEG_INFINITY;

        for p in particles.iter() {
            if p.radius < min_radius { min_radius = p.radius; }
            if p.radius > max_radius { max_radius = p.radius; }
        }

        // 🟢 FIXED: Base cell size must scale off max collision DIAMETER (2 * radius) 
        // to guarantee neighbor cells overlap enough to register contacts!
        let max_diameter = max_radius + max_radius;
        self.grid.set_cell_size(max_diameter);  
        
        environment.tuning.update_physics(min_radius, max_radius);
    }
    
    fn sub_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>, sub_step_dt: f64, environment: &ParticleEnvironment<V>) {
        let particles = storage.as_slice_mut();
        if particles.is_empty() { return; }

        let v_dt = V::Scalar::from_f64(sub_step_dt);
        let gravity_acc: V = environment.gravity.get(); 

        // 1. APPLY FORCE INTEGRATION (Fused iterator approach)
        for p in particles.iter_mut() {
            p.acc = gravity_acc;
        }

        // 2. EXECUTED PIPELINE DELEGATION
        // A. Update positions and velocities with your damping factors
        VerletPhysics::aos_update_kinetics(particles, v_dt, environment);

        // 🟢 FIXED: Clear and re-populate with exact cell size reference cache
        self.grid.clear();
        let cell_size = self.grid.cell_size;
        for (index, p) in particles.iter().enumerate() {
            let cell_key = p.pos.quantize_into(cell_size);
            self.grid.insert(cell_key, index);
        }

        // B. Narrowphase: Populate the registry and run relaxation constraints
        self.registry.clear();
        self.grid.aos_find_collisions(particles, &mut self.registry);

        VerletPhysics::aos_resolve_collisions( 
            particles, 
            &self.registry, 
            environment
        );

        // C. Bounded Wall Environments & Open Air Slide Friction Dampening
        VerletPhysics::aos_apply_position_constraints(particles, v_dt, environment);

        // D. Post-Loop Velocity History Restitution Bouncing
        VerletPhysics::aos_apply_particle_restitution(&self.registry, particles, environment);
    }
    
    fn post_step(&mut self, _storage: &mut VerletParticleAosVecStorage<V>, _environment: &ParticleEnvironment<V>) { }
}
 