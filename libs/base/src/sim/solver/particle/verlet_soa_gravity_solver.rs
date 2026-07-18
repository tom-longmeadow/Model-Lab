use std::hash::Hash; 
use crate::math::{FloatScalar, Vector}; 
use crate::sim::solver::Solver; 
use crate::sim::solver::particle::environment::ParticleEnvironment; 
use crate::sim::solver::particle::space::collision_registry::CollisionRegistry;
use crate::sim::solver::particle::verlet_particle::VerletParticleColumns; 
use crate::sim::solver::particle::verlet_physics::VerletPhysics;
use crate::sim::solver::particle::verlet_soa_vec_storage::{ErgonomicSoaCpuStorageExt, VerletParticleSoaVecStorage}; 
use crate::sim::storage::{Storage};  

use crate::sim::solver::particle::space::grid::UniformGrid;
use crate::ui::layout::color::Color;

pub struct VerletSoaGravitySolver<V> 
where 
    V: Vector,
    V::Quantized: Hash + Eq + Copy,
{  
    pub grid: UniformGrid<V>,        // 🟢 OWNED INSTANCE: Matches the unified AoS design layout
    pub registry: CollisionRegistry, 
    pub color_by_velocity: bool
}

impl<V> VerletSoaGravitySolver<V>
where 
    V: Vector,
    V::Quantized: Hash + Eq + Copy,
{
    pub fn new(initial_capacity: usize, default_cell_size: V::Scalar, color_by_velocity: bool) -> Self { 
        Self { 
            grid: UniformGrid::new(default_cell_size),
            registry: CollisionRegistry::with_capacity(initial_capacity), 
            color_by_velocity

        }
    }
}

impl<V: Vector + 'static> Solver<VerletParticleSoaVecStorage<V>, ParticleEnvironment<V>> for VerletSoaGravitySolver<V>
where
    V::Quantized: Hash + Eq + Copy,
{
    fn init(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>, _environment: &mut ParticleEnvironment<V>) { }

    fn pre_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, _tick: u64, step_dt:f64, environment: &mut ParticleEnvironment<V>) {
        environment.state.update_jitter(_tick);
        
        let len = Storage::len(storage);
        if len == 0 { return; }

        
        

        if self.color_by_velocity{
             
            let pos_base = storage.columns[VerletParticleColumns::Pos as usize].ptr.cast::<V>();
            let pos = unsafe { std::slice::from_raw_parts(pos_base, len) };

            let old_base = storage.columns[VerletParticleColumns::PosOld as usize].ptr.cast::<V>();
            let old = unsafe { std::slice::from_raw_parts(old_base, len) };

            // 1. Cast to a mutable pointer (*mut Color)
            let color_base = storage.columns[VerletParticleColumns::Color as usize].ptr.cast::<Color>();
            // 2. Use from_raw_parts_mut to get a mutable slice reference (&mut [Color])
            let color = unsafe { std::slice::from_raw_parts_mut(color_base, len) };
    
              
            let min_speed_threshold = 0.0_f64;   // Anything moving slower than this is "still" (Dark Blue)
            let max_expected_speed = 160.0_f64;  // Your cap for Foam White

            for i in 0..len {
                let p = pos[i];
                let p_old = old[i];
                
                let velocity = p - p_old;
                let speed = velocity.length().to_f64().abs() / step_dt;
                
                // 1. Subtract the threshold to clear out the micro-jitter
                let adjusted_speed = (speed - min_speed_threshold).max(0.0);
                
                // 2. Map the remaining range smoothly from [0.0, max - min]
                let speed_range = max_expected_speed - min_speed_threshold;
                let percentage = (adjusted_speed / speed_range).clamp(0.0, 1.0) as f32;

                let c = Color::get_color_at_percentage(&Color::WATER, percentage);
                
                color[i] = c; 
            }
        }
        

        let rad_base = storage.columns[VerletParticleColumns::Radius as usize].ptr.cast::<V::Scalar>();
        let radii = unsafe { std::slice::from_raw_parts(rad_base, len) };

        type S<V> = <V as Vector>::Scalar; 
        let mut min_radius = S::<V>::INFINITY;
        let mut max_radius = S::<V>::NEG_INFINITY;

        for &r in radii.iter() {
            if r < min_radius { min_radius = r; }
            if r > max_radius { max_radius = r; }
        }

        // Base cell size scales off the max collision DIAMETER (2 * radius)
        let max_diameter = max_radius + max_radius;
        self.grid.set_cell_size(max_diameter);  
        
        environment.tuning.update_physics(min_radius, max_radius);
    }
    
    fn sub_step(&mut self, storage: &mut VerletParticleSoaVecStorage<V>, sub_step_dt: f64, environment: &ParticleEnvironment<V>) {
        let len = Storage::len(storage);
        if len == 0 { return; }

        let v_dt = V::Scalar::from_f64(sub_step_dt);
        let gravity_acc = environment.gravity.get();

        // 🟢 FIXED: Unpack using our updated native vector slice extension trait
        let (positions, positions_old, accelerations, radii, inv_masses) = 
            storage.get_physics_components_mut::<V>();

        // 1. APPLY FORCE INTEGRATION (Unified vector math removes messy per-component indexing loops)
        for acc in accelerations.iter_mut() {
            *acc = gravity_acc;
        }

        // 2. EXECUTED PIPELINE DELEGATION
        // A. Update positions and velocities via our clean, non-aliasing contiguous array lanes
        VerletPhysics::soa_update_kinetics(
            positions, 
            positions_old, 
            accelerations, 
            v_dt, 
            environment
        );

        // 🟢 FIXED: Populate the grid directly using native types. No array reconstruction or f64 conversions!
        self.grid.clear();
        let cached_cell_size = self.grid.cell_size;

        for (i, &pos) in positions.iter().enumerate() {
            let cell_key = pos.quantize_into(cached_cell_size);
            self.grid.insert(cell_key, i);
        }

        // 🏎️ CRITICAL FIX: Match the signature of your new merged spatial direct solver.
        // This does the work of both old B & C steps combined!
        unsafe {
            self.grid.soa_resolve_collisions_spatial_direct(
                positions, 
                positions_old, 
                inv_masses, 
                radii, 
                v_dt,
                environment
            );
        }

        // D. Bounded wall limits and containment boundaries
        VerletPhysics::soa_apply_position_constraints(
            positions, 
            positions_old, 
            radii, 
            v_dt, 
            environment
        );

        // ⚠️ ATTENTION: Step E requires an approach adjustment (See below)
    }
 
    
    fn post_step(&mut self, _storage: &mut VerletParticleSoaVecStorage<V>,  _environment: &ParticleEnvironment<V>) { }
}