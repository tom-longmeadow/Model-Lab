use base::{aabb::AABB, insets::Insets, math::{FloatScalar, Vector, VectorMask}, 
    sim::{solver::{Solver, particle::{partition::{collision::CollisionRegistry, grid::{GridKey, UniformGrid}}, tuning::ParticlePhysicsTuning, verlet_physics::VerletPhysics}}, storage::{AosCpuStorage, Storage}}};
use std::hash::Hash;
use crate::simulation::particle::verlet_aos_vec_storage::AosVecStorage;

 
pub struct VerletAosGravitySolver<V> 
where 
    V: Vector,
    V::Quantized: Hash + Eq, 
{
    // 1. Simulation Stepping Control
    pub substep_count: u64,
    pub collision_iterations: u64,
    pub gravity: V,
    
    // 2. The Unified Engine Configuration
    pub tuning: ParticlePhysicsTuning<V::Scalar>,
    
    // 3. Environment & Spatial Partitioning
    pub bounds: AABB<V>,
    pub insets: Insets<V>,
    pub grid: UniformGrid<V>, // Constrained directly to your UniformGrid type

    // 4. Flat Cache-Friendly Scratch Buffers  
    pub scratch_pos: Vec<V>, 
    pub scratch_pos_old: Vec<V>, 
    pub scratch_radii: Vec<V::Scalar>,
}

impl<V> VerletAosGravitySolver<V>
where 
    V: Vector, 
    V::Quantized: Hash + Eq,
{
    /// Creates a fresh solver instance matching your dimension variants
    pub fn new(
        substep_count: u64,
        collision_iterations: u64,
        gravity: V, 
        insets: Insets<V>, 
        initial_capacity: usize,
    ) -> Self {

        // Initialize your spatial partitioning grid. 
        // Note: cell_size will be dynamically updated by your pre_step loop later.
        let initial_cell_size: V::Scalar = <V::Scalar as FloatScalar>::from_f64(1.0);
        let grid = UniformGrid::<V>::new(initial_cell_size);
        let bounds = AABB::<V>::default(); 
        let tuning = ParticlePhysicsTuning::<V::Scalar>::new(initial_cell_size, collision_iterations);
        
        Self {
            substep_count,
            collision_iterations,
            gravity,
            tuning,
            bounds,
            insets,
            grid,
            scratch_pos: Vec::with_capacity(initial_capacity),
            scratch_pos_old: Vec::with_capacity(initial_capacity),
            scratch_radii: Vec::with_capacity(initial_capacity),
        }
    }
}
 

impl<V> Solver<AosVecStorage<V>> for VerletAosGravitySolver<V>
where 
    V: Vector,
    V::Quantized: Hash + Eq,
{

    type Bounds = AABB<V>;

    fn substep_count(&self) -> u64 { self.substep_count }

    fn init(&mut self, _storage: &mut AosVecStorage<V>) { }

    fn post_step(&mut self, _storage: &mut AosVecStorage<V>, _dt: f64) {  }
    
    fn pre_step(&mut self, storage: &mut AosVecStorage<V>, _dt: f64, _tick: u64, bounds: &AABB<V>) {

        // 1. Construct the new constraint area
        let new_bounds = AABB::from_insets(bounds, &self.insets);
        
        // 2. DETECT CHANGE: Check if this is a subsequent frame and the window size actually changed
        // (Ensure you have a way to know if self.bounds was already initialized, or check if min/max differ)
        if self.bounds.min != new_bounds.min || self.bounds.max != new_bounds.max {
    
         
                let bounds_initialized = self.bounds.max.cmpgt(self.bounds.min).any();

                if bounds_initialized {
                for p in storage.iter_mut() {
                    // Both axes updated gracefully in a single assignment step
                    VerletPhysics::scale_to_bounds(
                        &mut p.pos,
                        &mut p.pos_old,
                        self.bounds.min,
                        self.bounds.max,
                        new_bounds.min,
                        new_bounds.max,
                    );
                }
            }
        }

        // 3. Commit the new bounds safely
        self.bounds = new_bounds;
        
        // 4. Your existing scratch/tuning synchronization logic...
        if storage.len() != self.scratch_radii.len() {
            self.scratch_radii.clear();

            type S<V> = <V as Vector>::Scalar; 
            let mut min_radius = S::<V>::INFINITY;
            let mut max_radius = S::<V>::NEG_INFINITY;

            for p in storage.as_slice().iter() {
                self.scratch_radii.push(p.radius);

                if p.radius < min_radius { min_radius = p.radius; }
                if p.radius > max_radius { max_radius = p.radius; }
            }

            self.grid.set_cell_size(max_radius);  

            self.tuning = ParticlePhysicsTuning::new(min_radius, self.collision_iterations);
        } 
    }

    fn sub_step(&mut self, storage: &mut AosVecStorage<V>, dt: f64) {


        // --- PHASE 1: KINETICS PASS ---
let scalar_dt = <V::Scalar as FloatScalar>::from_f64(dt);

for p in storage.iter_mut() {
    let mut acc: V = V::ZERO; 
    acc += self.gravity;
    VerletPhysics::update_kinetics(&self.tuning, &mut p.pos, &mut p.pos_old, scalar_dt, &mut acc);
}

// --- PHASE 2 & 3: UNIFIED CONSTRAINT & COLLISION LOOP ---
let mut collisions = CollisionRegistry::new();
let len = storage.len();

// 1. Synchronize scratch arrays from storage
self.scratch_pos.clear();
self.scratch_pos_old.clear();
for p in storage.iter() {
    self.scratch_pos.push(p.pos);
    self.scratch_pos_old.push(p.pos_old);
}

// 2. Run the heavy spatial partition / broadphase detection once
VerletPhysics.detect_collisions(len, &self.scratch_radii, &self.scratch_pos, &mut collisions);

// 3. Relax ALL positional constraints simultaneously
for _ in 0..self.collision_iterations {
    // Inter-particle constraints
    for collision in &collisions.pairs {
        let a = collision.a_index;
        let b = collision.b_index;
        if a == b { continue; }

        let (pos_a, pos_b, pos_old_a, pos_old_b) = unsafe {
            let pos_ptr = self.scratch_pos.as_mut_ptr();
            let old_ptr = self.scratch_pos_old.as_mut_ptr();
            (
                &mut *pos_ptr.add(a),
                &mut *pos_ptr.add(b),
                &mut *old_ptr.add(a),
                &mut *old_ptr.add(b),
            )
        };

        let inv_mass = <V::Scalar as FloatScalar>::from_f64(1.0);

        VerletPhysics::resolve_particle_collisions(
            &self.tuning, pos_a, pos_old_a, pos_b, pos_old_b,
            self.scratch_radii[a], self.scratch_radii[b], inv_mass, inv_mass,
        );
    }

    // CRITICAL FIX: Evaluate boundaries DURING relaxation so particles push against each other *and* the wall
    for i in 0..len {
        VerletPhysics::apply_position_constraints(
            &self.tuning, self.bounds.min, self.bounds.max, 
            self.scratch_radii[i], &mut self.scratch_pos[i], &mut self.scratch_pos_old[i]
        );
    }
}

// 4. Apply restitution phase using the completely relaxed positions
for collision in &collisions.pairs {
    let a = collision.a_index;
    let b = collision.b_index;

    let (pos_old_a, pos_old_b) = if a < b {
        let (left, right) = self.scratch_pos_old.as_mut_slice().split_at_mut(b);
        (&mut left[a], &mut right[0])
    } else {
        continue;
    };

    VerletPhysics::apply_collision_restitution(
        &self.tuning, &self.scratch_pos[a], &self.scratch_pos[b],
        pos_old_a, pos_old_b, self.scratch_radii[a], self.scratch_radii[b],
    );
}

// 5. Commit everything back to storage uniformly
for (i, p) in storage.iter_mut().enumerate() {
    p.pos = self.scratch_pos[i];
    p.pos_old = self.scratch_pos_old[i]; // Ensure old positions (velocity) carry over!
}

        // // --- PHASE 1: KINETICS PASS ---
        // let scalar_dt = <V::Scalar as FloatScalar>::from_f64(dt);

        // for p in storage.iter_mut() {
        //     let mut acc: V = V::ZERO; 
        //     acc += self.gravity;

        //     VerletPhysics::update_kinetics(&self.tuning, &mut p.pos, &mut p.pos_old, scalar_dt, &mut acc);
             
        // }

        // // --- PHASE 2: PARTICLE-TO-PARTICLE COLLISIONS VIA REGISTRY ---
        // let mut collisions = CollisionRegistry::new();
        // let len = storage.len();

        // // 1. Synchronize scratch position and radius data once from storage
        // self.scratch_pos.clear();
        // self.scratch_pos_old.clear();
        // for p in storage.iter() {
        //     self.scratch_pos.push(p.pos);
        //     self.scratch_pos_old.push(p.pos_old);
        // }

        // // 2. Run the heavy O(N^2) detection loop exactly once
        // VerletPhysics.detect_collisions(len, &self.scratch_radii, &self.scratch_pos, &mut collisions);

        // // 3. Relax constraints over multiple iterations using the updated pure function
        // for _ in 0..self.collision_iterations {
        //     for collision in &collisions.pairs {
        //         let a = collision.a_index;
        //         let b = collision.b_index;

        //         // Ensure we never alias the exact same index
        //         if a == b { continue; }

        //         // Bypassing the borrow checker safely using raw pointer offsets
        //         let (pos_a, pos_b, pos_old_a, pos_old_b) = unsafe {
        //             let pos_ptr = self.scratch_pos.as_mut_ptr();
        //             let old_ptr = self.scratch_pos_old.as_mut_ptr();

        //             (
        //                 &mut *pos_ptr.add(a),
        //                 &mut *pos_ptr.add(b),
        //                 &mut *old_ptr.add(a),
        //                 &mut *old_ptr.add(b),
        //             )
        //         };

        //         let inv_mass_a = <V::Scalar as FloatScalar>::from_f64(1.0);
        //         let inv_mass_b = <V::Scalar as FloatScalar>::from_f64(1.0);

        //         VerletPhysics::resolve_particle_collisions(
        //             &self.tuning, 
        //             pos_a,
        //             pos_old_a,
        //             pos_b,
        //             pos_old_b,
        //             self.scratch_radii[a],
        //             self.scratch_radii[b],
        //             inv_mass_a,  
        //             inv_mass_b,
        //         );
        //     }
        // }

        //  // 3b. Relax constraints over multiple iterations using the updated pure function
        // for collision in &collisions.pairs {
        //     let a = collision.a_index;
        //     let b = collision.b_index;

        //     // Use standard Vec method .as_mut_slice() and index right[0] safely
        //     let (pos_old_a, pos_old_b) = if a < b {
        //         let (left, right) = self.scratch_pos_old.as_mut_slice().split_at_mut(b);
        //         (&mut left[a], &mut right[0]) // Fixed: right[0] targets original index b
        //     } else {
        //         continue;
        //     };

        //     VerletPhysics::apply_collision_restitution(
        //         &self.tuning,
        //         &self.scratch_pos[a],
        //         &self.scratch_pos[b],
        //         pos_old_a,
        //         pos_old_b,
        //         self.scratch_radii[a],
        //         self.scratch_radii[b],
        //     );
        // }

        // // 4. Commit final relaxed positions back to storage
        // for (i, p) in storage.iter_mut().enumerate() {
        //     p.pos = self.scratch_pos[i];
        // }

        // // --- PHASE 3: BOUNDARY CONSTRAINTS PASS ---
        // for p in storage.iter_mut() {
        //     VerletPhysics::apply_position_constraints(&self.tuning, self.bounds.min, self.bounds.max, p.radius, &mut p.pos, &mut p.pos_old); 
        // }
    }
}
          

        // // --- PHASE 1: KINETICS & FORCES PASS ---
        // for p in storage.iter_mut() {
        //     let mut acc_x = 0.0;
        //     let mut acc_y = -self.gravity;

        //     VerletSolver::update_kinetics_1d(&self.tuning, &mut p.pos.x, &mut p.pos_old.x, dt, &mut acc_x);
        //     VerletSolver::update_kinetics_1d(&self.tuning, &mut p.pos.y, &mut p.pos_old.y, dt, &mut acc_y);
        // }

        // // --- PHASE 2: PURE POSITION CONSTRAINT RELAXATION LOOP ---
        // // This loop now ONLY fixes overlaps. No velocity changes or pos_old modifications happen here.
        // for _iteration in 0..self.collision_iterations {
            
        //     // Re-sync flat grid tracking locations
        //     self.scratch_pos.clear();
        //     for p in storage.iter() {
        //         self.scratch_pos.push(p.pos);
        //     }
            
        //     self.grid.populate(&self.scratch_pos);
        //     let mut registry = CollisionRegistry::<DVec2>::new();
        //     self.grid.find_collisions(&self.scratch_pos, &self.scratch_radii, &mut registry);
            
        //     // Handle standalone wall constraints if no particles touch
        //     if registry.pairs.is_empty() {
        //         for p in storage.iter_mut() {
        //             VerletSolver::apply_axis_position_constraints_1d(&self.tuning, self.bounds.min.x, self.bounds.max.x, p.radius, &mut p.pos.x);
        //             VerletSolver::apply_axis_position_constraints_1d(&self.tuning, self.bounds.min.y, self.bounds.max.y, p.radius, &mut p.pos.y);
        //         }
        //         break; 
        //     }
            
        //     let particles = storage.as_slice_mut(); 

        //     for collision in registry.pairs {
        //         let (idx_a, idx_b) = (collision.a_index, collision.b_index);
        //         if idx_a >= particles.len() || idx_b >= particles.len() { continue; }
                
        //         let (particle_a, particle_b) = if idx_a < idx_b {
        //             let (left, right) = particles.split_at_mut(idx_b);
        //             (&mut left[idx_a], &mut right[0])
        //         } else {
        //             let (left, right) = particles.split_at_mut(idx_a);
        //             (&mut right[0], &mut left[idx_b])
        //         };

        //         // Recalculate true physical distance based on altered positions
        //         let delta = particle_b.pos - particle_a.pos;
        //         let dist_sq = delta.dot(delta);
        //         let min_dist = particle_a.radius + particle_b.radius;
                
        //         if dist_sq >= min_dist * min_dist { continue; }
                
        //         let dist = dist_sq.sqrt();
        //         let penetration = min_dist - dist;
        //         let normal = if dist > 1e-10 { delta / dist } else { DVec2::new(1.0, 0.0) };
                
        //         let inv_mass_a = 1.0;
        //         let inv_mass_b = 1.0;

        //         // POSITION SOLVERS PER AXIS (No velocity logic)
        //         VerletSolver::resolve_particle_position_1d(
        //             &self.tuning,
        //             &mut particle_a.pos.x, inv_mass_a, 
        //             &mut particle_b.pos.x, inv_mass_b, 
        //             normal.x, penetration,
        //         );

        //         VerletSolver::resolve_particle_position_1d(
        //             &self.tuning,
        //             &mut particle_a.pos.y, inv_mass_a, 
        //             &mut particle_b.pos.y, inv_mass_b, 
        //             normal.y, penetration,
        //         );
        //     }
        
        //     // Keep boundaries running inside the relaxation loop purely for positions
        //     for p in storage.iter_mut() {
        //         VerletSolver::apply_axis_position_constraints_1d(&self.tuning, self.bounds.min.x, self.bounds.max.x, p.radius, &mut p.pos.x);
        //         VerletSolver::apply_axis_position_constraints_1d(&self.tuning, self.bounds.min.y, self.bounds.max.y, p.radius, &mut p.pos.y);
        //     } 
        // }

        // // --- PHASE 3: FINAL POST-LOOP VELOCITY AND RESTITUTION PASS ---
        // // Run this exactly ONCE per sub-step after all position shifting finishes.
        
        // // Step A: Final Wall Bounces
        // for p in storage.iter_mut() {
        //     VerletSolver::resolve_final_wall_velocity_1d(&self.tuning, self.bounds.min.x, self.bounds.max.x, p.radius, &mut p.pos.x, &mut p.pos_old.x, dt);
        //     VerletSolver::resolve_final_wall_velocity_1d(&self.tuning, self.bounds.min.y, self.bounds.max.y, p.radius, &mut p.pos.y, &mut p.pos_old.y, dt);
        // }

        // // Step B: Final Particle-to-Particle Impulse Bounces
        // self.scratch_pos.clear();
        // for p in storage.iter() {
        //     self.scratch_pos.push(p.pos);
        // }
        // self.grid.populate(&self.scratch_pos);
        // let mut final_registry = CollisionRegistry::<DVec2>::new();
        // self.grid.find_collisions(&self.scratch_pos, &self.scratch_radii, &mut final_registry);

        // let particles = storage.as_slice_mut();
        // for collision in final_registry.pairs {
        //     let (idx_a, idx_b) = (collision.a_index, collision.b_index);
        //     if idx_a >= particles.len() || idx_b >= particles.len() { continue; }
            
        //     let (particle_a, particle_b) = if idx_a < idx_b {
        //         let (left, right) = particles.split_at_mut(idx_b);
        //         (&mut left[idx_a], &mut right[0])
        //     } else {
        //         let (left, right) = particles.split_at_mut(idx_a);
        //         (&mut right[0], &mut left[idx_b])
        //     };

        //     let delta = particle_b.pos - particle_a.pos;
        //     let dist_sq = delta.dot(delta);
        //     let min_dist = particle_a.radius + particle_b.radius;
            
        //     if dist_sq >= min_dist * min_dist { continue; }
            
        //     let dist = dist_sq.sqrt();
        //     let normal = if dist > 1e-10 { delta / dist } else { DVec2::new(1.0, 0.0) };
            
        //     let inv_mass_a = 1.0;
        //     let inv_mass_b = 1.0;
        //     let total_inv_mass = inv_mass_a + inv_mass_b;

        //     // Evaluate actual final post-relaxation velocities
        //     let vel_a = (particle_a.pos - particle_a.pos_old) / dt;
        //     let vel_b = (particle_b.pos - particle_b.pos_old) / dt;
        //     let vel_along_normal = (vel_a - vel_b).dot(normal);

        //     // Apply energy modifications only if particles are moving toward each other
        //     if vel_along_normal > 0.0 {
        //         if vel_along_normal > self.tuning.velocity_bounce_threshold {
        //             // Dynamic Energetic Bounce: Run standard restitution math
        //             let impulse_vel = -(1.0 + self.tuning.restitution) * vel_along_normal / total_inv_mass;
                    
        //             VerletSolver::apply_particle_velocity_impulse_1d(
        //                 &mut particle_a.pos_old.x, inv_mass_a,
        //                 &mut particle_b.pos_old.x, inv_mass_b,
        //                 normal.x, impulse_vel, dt,
        //             );

        //             VerletSolver::apply_particle_velocity_impulse_1d(
        //                 &mut particle_a.pos_old.y, inv_mass_a,
        //                 &mut particle_b.pos_old.y, inv_mass_b,
        //                 normal.y, impulse_vel, dt,
        //             );
        //         } else {
        //             // Resting Contact Mode: Eliminate relative velocity along normal entirely.
        //             // Re-aligns history positions to lock the entities into an unified, stable stack.
        //             let target_vel_a = vel_a - normal * vel_along_normal * (inv_mass_a / total_inv_mass);
        //             let target_vel_b = vel_b + normal * vel_along_normal * (inv_mass_b / total_inv_mass);

        //             particle_a.pos_old = particle_a.pos - (target_vel_a * dt);
        //             particle_b.pos_old = particle_b.pos - (target_vel_b * dt);
        //         }
        //     }
        // }
//     }
// }

 