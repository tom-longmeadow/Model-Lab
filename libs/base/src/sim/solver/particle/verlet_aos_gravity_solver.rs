 use std::hash::Hash; 
use crate::math::{FloatScalar, Vector}; 
use crate::sim::solver::Solver;
use crate::sim::solver::particle::environment::ParticleEnvironment;
use crate::sim::solver::particle::space::collision::CollisionRegistry; 
use crate::sim::solver::particle::verlet_aos_vec_storage::VerletParticleAosVecStorage;
use crate::sim::solver::particle::verlet_physics::VerletPhysics;
use crate::sim::storage::{AosCpuStorage, Storage};

pub struct VerletAosGravitySolver<V> 
where 
    V: Vector,
    V::Quantized: Hash + Eq, 
{ 
    pub scratch_pos:     Vec<V>, 
    pub scratch_pos_old: Vec<V>,  
    pub scratch_radii:   Vec<V::Scalar>, 
}

impl<V> VerletAosGravitySolver<V>
where 
    V: Vector, 
    V::Quantized: Hash + Eq,
{
    pub fn new( 
        initial_capacity: usize, 
    ) -> Self { 
        Self { 
            scratch_pos:     Vec::with_capacity(initial_capacity),
            scratch_pos_old: Vec::with_capacity(initial_capacity), 
            scratch_radii:   Vec::with_capacity(initial_capacity),
        }
    }
}

impl<V> Solver<VerletParticleAosVecStorage<V>, ParticleEnvironment<V>> for VerletAosGravitySolver<V>
where
    V: Vector,
    V::Quantized: Hash + Eq,
{
 
    fn init(&mut self, _storage: &mut VerletParticleAosVecStorage<V>, _environment: &mut ParticleEnvironment<V>) { }

    fn pre_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>, _dt: f64, _tick: u64, environment: &mut ParticleEnvironment<V>) {
        environment.state.update_jitter(_tick);
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

            environment.space.grid.set_cell_size(max_radius);  
            environment.tuning.update_physics(min_radius, max_radius);
        }
    }
    
    fn sub_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>,  dt: f64, environment: &ParticleEnvironment<V>) {
               
        // KINETICS PASS 
        let sub_step_dt = <V::Scalar as FloatScalar>::from_f64(dt);
        for p in storage.iter_mut() {
            environment.gravity.get();
            let mut acc: V = environment.gravity.get(); 
            VerletPhysics::update_kinetics(sub_step_dt, &environment, &mut p.pos, &mut p.pos_old, &mut acc);
        }

        let len = storage.len();
        let mut collisions = CollisionRegistry::new();
        
        self.scratch_pos.clear();
        self.scratch_pos_old.clear(); 

        for p in storage.iter() {
            self.scratch_pos.push(p.pos);
            self.scratch_pos_old.push(p.pos_old); 
        }

        // Run the heavy spatial partition / broadphase detection once
        VerletPhysics.detect_collisions(len, &self.scratch_radii, &self.scratch_pos, &mut collisions);

        // Relax ALL positional constraints simultaneously
        for _ in 0..environment.tuning.collision_iterations {
            for collision in &collisions.pairs {
                let a = collision.a_index;
                let b = collision.b_index;
                if a == b { continue; }

                let (pos_a, pos_b) = unsafe {
                    let pos_ptr = self.scratch_pos.as_mut_ptr();
                    (&mut *pos_ptr.add(a), &mut *pos_ptr.add(b))
                };

                let inv_mass = <V::Scalar as FloatScalar>::from_f64(1.0);

                // Keep this strictly focused on positions to avoid iteration multiplying
                VerletPhysics::resolve_particle_collisions(
                    &environment,
                    pos_a, 
                    pos_b, 
                    self.scratch_radii[a], 
                    self.scratch_radii[b], 
                    inv_mass, 
                    inv_mass,
                );
            } 
        }

        for collision in &collisions.pairs {
            let a = collision.a_index;
            let b = collision.b_index;
            if a == b { continue; }

            let (pos_a, pos_b, pos_old_a, pos_old_b) = unsafe {
                let pos_ptr = self.scratch_pos.as_mut_ptr();
                let pos_old_ptr = self.scratch_pos_old.as_mut_ptr(); // Assumes scratch_pos_old exists
                (
                    &*pos_ptr.add(a),
                    &*pos_ptr.add(b),
                    &mut *pos_old_ptr.add(a),
                    &mut *pos_old_ptr.add(b),
                )
            };

            let inv_mass = <V::Scalar as FloatScalar>::from_f64(1.0);

            VerletPhysics::apply_particle_restitution(
                &environment.tuning.physics,
                pos_a,
                pos_b,
                pos_old_a,
                pos_old_b,
                self.scratch_radii[a],
                self.scratch_radii[b],
                inv_mass,
                inv_mass,
            );
        }

   
        for i in 0..len {
            VerletPhysics::apply_position_constraints(
                sub_step_dt, &environment, 
                self.scratch_radii[i], &mut self.scratch_pos[i], &mut self.scratch_pos_old[i]
            );
        }

        // Apply the safety clamp over time and commit everything back to storage uniformly
        let sub_step_max_vel = environment.tuning.physics.max_velocity * sub_step_dt;
        let max_vel_squared = sub_step_max_vel * sub_step_max_vel;

        for (i, p) in storage.iter_mut().enumerate() {
            let pos = self.scratch_pos[i];
            let mut pos_old = self.scratch_pos_old[i];
            let vel: V = pos - pos_old;
            let vel_sq = vel.length_squared();

            if vel_sq > max_vel_squared {
                let vel_len = vel_sq.sqrt();  
                let clamped_vel = vel * (sub_step_max_vel / vel_len); 
                pos_old = pos - clamped_vel;
            }

            p.pos = pos;
            p.pos_old = pos_old;
        }
    }
    
    fn post_step(&mut self, _storage: &mut VerletParticleAosVecStorage<V>, _dt: f64, _environment: &ParticleEnvironment<V>) { }
     
}



 

// // impl<V> VerletAosGravitySolver<V>
// // where 
// //     V: Vector, 
// //     V::Quantized: Hash + Eq,
// // {
// //     pub fn new(
// //         substep_count: u64,
// //         collision_iterations: u64,
// //         gravity: V, 
// //         insets: Insets<V>, 
// //         initial_capacity: usize,
// //     ) -> Self {

// //         // Note: cell_size will be dynamically updated by pre_step loop later.
// //         let initial_cell_size: V::Scalar = <V::Scalar as FloatScalar>::from_f64(1.0);
// //         let grid = UniformGrid::<V>::new(initial_cell_size);
// //         let bounds = AABB::<V>::default(); 
// //         let tuning = PhysicsTuning::<V::Scalar>::new(
// //             initial_cell_size, initial_cell_size, collision_iterations);
        
// //         Self {
// //             substep_count,
// //             collision_iterations,
// //             gravity,
// //             tuning,
// //             bounds,
// //             insets,
// //             grid,
// //             scratch_pos: Vec::with_capacity(initial_capacity),
// //             scratch_pos_old: Vec::with_capacity(initial_capacity), 
// //             scratch_radii: Vec::with_capacity(initial_capacity),
// //         }
// //     }
// // }
 

// // impl<V> Solver<VerletParticleAosVecStorage<V>> for VerletAosGravitySolver<V>
// // where 
// //     V: Vector,
// //     V::Quantized: Hash + Eq,
// // {

// //     type Bounds = AABB<V>;

// //     fn substep_count(&self) -> u64 { self.substep_count }

// //     fn init(&mut self, _storage: &mut VerletParticleAosVecStorage<V>) { }

// //     fn post_step(&mut self, _storage: &mut VerletParticleAosVecStorage<V>, _dt: f64) {  }
    
// //     fn pre_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>, _dt: f64, _tick: u64, bounds: &AABB<V>) {


// //         self.tuning.update_jitter(_tick);
// //         if storage.len() != self.scratch_radii.len() {
// //             self.scratch_radii.clear();

// //             type S<V> = <V as Vector>::Scalar; 
// //             let mut min_radius = S::<V>::INFINITY;
// //             let mut max_radius = S::<V>::NEG_INFINITY;

// //             for p in storage.as_slice().iter() {
// //                 self.scratch_radii.push(p.radius);

// //                 if p.radius < min_radius { min_radius = p.radius; }
// //                 if p.radius > max_radius { max_radius = p.radius; }
// //             }

// //             self.grid.set_cell_size(max_radius);  
// //             self.tuning = PhysicsTuning::new(min_radius, max_radius, self.collision_iterations);
// //         } 

       
// //         let new_bounds = AABB::from_insets(bounds, &self.insets);  
// //         // if self.bounds.min != new_bounds.min || self.bounds.max != new_bounds.max { 
// //         //         let bounds_initialized = self.bounds.max.cmpgt(self.bounds.min).any();

// //         //         if bounds_initialized {
// //         //         for p in storage.iter_mut() { 
// //         //             VerletPhysics::scale_to_bounds(
// //         //                  &self.tuning, 
// //         //                  &mut p.pos,
// //         //                 &mut p.pos_old,
// //         //                 self.bounds.min,
// //         //                 self.bounds.max,
// //         //                 new_bounds.min,
// //         //                 new_bounds.max,
// //         //             );
// //         //         }
// //         //     }
// //         // }
 
// //         self.bounds = new_bounds;
         
       
// //     }

// //     fn sub_step(&mut self, storage: &mut VerletParticleAosVecStorage<V>, dt: f64) {
// //         // KINETICS PASS 
// //         let sub_step_dt = <V::Scalar as FloatScalar>::from_f64(dt);
// //         for p in storage.iter_mut() {
// //             let mut acc: V = V::ZERO; 
// //             acc += self.gravity;
// //             VerletPhysics::update_kinetics(sub_step_dt, &self.tuning, &mut p.pos, &mut p.pos_old, &mut acc);
// //         }

// //         let len = storage.len();
// //         let mut collisions = CollisionRegistry::new();
        
// //         self.scratch_pos.clear();
// //         self.scratch_pos_old.clear(); 

// //         for p in storage.iter() {
// //             self.scratch_pos.push(p.pos);
// //             self.scratch_pos_old.push(p.pos_old); 
// //         }

// //         // Run the heavy spatial partition / broadphase detection once
// //         VerletPhysics.detect_collisions(len, &self.scratch_radii, &self.scratch_pos, &mut collisions);

// //         // Relax ALL positional constraints simultaneously
// //         for _ in 0..self.collision_iterations {
// //             for collision in &collisions.pairs {
// //                 let a = collision.a_index;
// //                 let b = collision.b_index;
// //                 if a == b { continue; }

// //                 let (pos_a, pos_b) = unsafe {
// //                     let pos_ptr = self.scratch_pos.as_mut_ptr();
// //                     (&mut *pos_ptr.add(a), &mut *pos_ptr.add(b))
// //                 };

// //                 let inv_mass = <V::Scalar as FloatScalar>::from_f64(1.0);

// //                 // Keep this strictly focused on positions to avoid iteration multiplying
// //                 VerletPhysics::resolve_particle_collisions(
// //                     &self.tuning, 
// //                     pos_a, 
// //                     pos_b, 
// //                     self.scratch_radii[a], 
// //                     self.scratch_radii[b], 
// //                     inv_mass, 
// //                     inv_mass,
// //                 );
// //             }

// //             // // --- NEW STEP: Pure geometric boundary containment ---
// //             // // Keeps particles inside the window while they push against each other
// //             // // without injecting artificial wall bounce velocities!
// //             // for i in 0..len {
// //             //     VerletPhysics::clamp_position_bounds(
// //             //         self.bounds.min, self.bounds.max, 
// //             //         self.scratch_radii[i], &mut self.scratch_pos[i]
// //             //     );
// //             // }
// //         }

// //         for collision in &collisions.pairs {
// //             let a = collision.a_index;
// //             let b = collision.b_index;
// //             if a == b { continue; }

// //             let (pos_a, pos_b, pos_old_a, pos_old_b) = unsafe {
// //                 let pos_ptr = self.scratch_pos.as_mut_ptr();
// //                 let pos_old_ptr = self.scratch_pos_old.as_mut_ptr(); // Assumes scratch_pos_old exists
// //                 (
// //                     &*pos_ptr.add(a),
// //                     &*pos_ptr.add(b),
// //                     &mut *pos_old_ptr.add(a),
// //                     &mut *pos_old_ptr.add(b),
// //                 )
// //             };

// //             let inv_mass = <V::Scalar as FloatScalar>::from_f64(1.0);

// //             VerletPhysics::apply_particle_restitution(
// //                 &self.tuning,
// //                 pos_a,
// //                 pos_b,
// //                 pos_old_a,
// //                 pos_old_b,
// //                 self.scratch_radii[a],
// //                 self.scratch_radii[b],
// //                 inv_mass,
// //                 inv_mass,
// //             );
// //         }

// //         // // Apply restitution phase using completely relaxed positions + unpolluted velocities
// //         // for collision in &collisions.pairs {
// //         //     let a = collision.a_index;
// //         //     let b = collision.b_index;

// //         //     let (pos_old_a, pos_old_b) = if a < b {
// //         //         let (left, right) = self.scratch_pos_old.as_mut_slice().split_at_mut(b);
// //         //         (&mut left[a], &mut right[0])
// //         //     } else {
// //         //         continue;
// //         //     };

// //         //     VerletPhysics::apply_collision_restitution(
// //         //         sub_step_dt, 
// //         //         &self.tuning, 
// //         //         &self.scratch_pos[a],           
// //         //         &self.scratch_pos[b],           
// //         //         &self.scratch_pos_unrelaxed[a], 
// //         //         &self.scratch_pos_unrelaxed[b], 
// //         //         pos_old_a, 
// //         //         pos_old_b, 
// //         //         self.scratch_radii[a], 
// //         //         self.scratch_radii[b],
// //         //     );
// //         // }

// //         // --- NEW STEP: Run full wall constraint pass exactly ONCE here ---
// //         // This processes real bounces and locks resting velocities to zero against the floor
// //         for i in 0..len {
// //             VerletPhysics::apply_position_constraints(
// //                 sub_step_dt, &self.tuning, self.bounds.min, self.bounds.max, 
// //                 self.scratch_radii[i], &mut self.scratch_pos[i], &mut self.scratch_pos_old[i]
// //             );
// //         }

// //         // Apply the safety clamp over time and commit everything back to storage uniformly
// //         let sub_step_max_vel = self.tuning.max_velocity * sub_step_dt;
// //         let max_vel_squared = sub_step_max_vel * sub_step_max_vel;

// //         for (i, p) in storage.iter_mut().enumerate() {
// //             let pos = self.scratch_pos[i];
// //             let mut pos_old = self.scratch_pos_old[i];
// //             let vel: V = pos - pos_old;
// //             let vel_sq = vel.length_squared();

// //             if vel_sq > max_vel_squared {
// //                 let vel_len = vel_sq.sqrt();  
// //                 let clamped_vel = vel * (sub_step_max_vel / vel_len); 
// //                 pos_old = pos - clamped_vel;
// //             }

// //             p.pos = pos;
// //             p.pos_old = pos_old;
// //         }
// //     }
 
// // }
          
 
 