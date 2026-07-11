use base::{math::{Bounds, DVec2}, sim::{solver::{Solver, integrator::Verlet, partition::{collision::CollisionRegistry, grid::UniformGrid2D}, verlet::{self, RectConstraint}}, storage::{AosCpuStorage, Storage}}};

use crate::simulation::verlet_2d::aos_vec_storage::AosVecStorage;

pub struct GravitySolver{
    substep_count: u64,
    collision_iterations: usize,
    bounds: RectConstraint,
    gravity: f64,
    inset: f64,
    restitution: f64,
    grid: UniformGrid2D,
    // Persistent scratch buffers (reused each frame, no allocation)
    scratch_positions: Vec<DVec2>,
    scratch_radii: Vec<f64>,
    penetration_tolerance: f64,
}

impl GravitySolver
{
    pub fn new(substep_count: u64, bounds: &Bounds, restitution: f64, gravity: f64, inset: f64) -> Self {
        Self { 
            substep_count,
            collision_iterations: 5,  // Multiple passes to fully separate overlapping particles
            bounds: RectConstraint::from_bounds_with_inset(bounds, inset, restitution),
            gravity,
            inset,
            restitution,
            grid: UniformGrid2D::new(0.0),
            scratch_positions: Vec::new(),
            scratch_radii: Vec::new(),
            penetration_tolerance: 1.0,
        }
    }
}
 
 

impl Solver<AosVecStorage> for GravitySolver {

    fn substep_count(&self) -> u64 { self.substep_count }

    fn init(&mut self, storage: &mut AosVecStorage) {

        
       
    }
    
    fn pre_step(&mut self, storage: &mut AosVecStorage, _dt: f64, _tick: u64, bounds: &Bounds) {
        self.bounds = RectConstraint::from_bounds_with_inset(bounds, self.inset, self.restitution);
        
        if storage.len() != self.scratch_radii.len() {
            self.scratch_radii.clear();
            for p in storage.iter() {
                self.scratch_radii.push(p.radius);
            }

            let mut min_radius = f64::INFINITY;
            let mut max_radius = f64::NEG_INFINITY;

            for &radius in &self.scratch_radii {
                if radius < min_radius { min_radius = radius; }
                if radius > max_radius { max_radius = radius; }
            }

            self.grid.set_cell_size(max_radius);
            self.penetration_tolerance = min_radius * 0.001;
        } 
    }

    fn sub_step(&mut self, storage: &mut AosVecStorage, dt: f64) {
        
        // Integrate
        for p in storage.iter_mut() {
            p.acc.y = -self.gravity;
            Verlet::step(&mut p.pos.x, &mut p.pos_old.x, p.acc.x, dt);
            Verlet::step(&mut p.pos.y, &mut p.pos_old.y, p.acc.y, dt);
        }

        // // Apply boundary constraints immediately after integration
        // for p in storage.iter_mut() {
        //     self.bounds.apply(&mut p.pos.x, &mut p.pos_old.x, &mut p.pos.y, &mut p.pos_old.y, true);
        // }
        
        // Extract data into scratch buffers  
        self.scratch_positions.clear(); 
        for p in storage.iter() {
            self.scratch_positions.push(p.pos); 
        }
 
        
         for iteration in 0..self.collision_iterations {

            // Only apply restitution on the first pass.
            // Subsequent passes are purely positional correction — applying (1+e)
            // repeatedly would multiply outgoing velocity by (1+e)^n, adding energy.
            let iter_restitution = if iteration == 0 { self.restitution } else { 0.0 };


            // Update positions for grid
            self.scratch_positions.clear();
            for p in storage.iter() {
                self.scratch_positions.push(p.pos);
            }
            
            // Detect collisions
            self.grid.populate(&self.scratch_positions);
            let mut registry = CollisionRegistry::<DVec2>::new();
            self.grid.find_collisions(&self.scratch_positions, &self.scratch_radii, &mut registry);
            
            if registry.pairs.is_empty() {
                break;  // Early exit if no collisions
            }
            
            // Resolve collisions
            let particles = storage.as_slice_mut();
            let mut resolved_count = 0;

            for collision in registry.pairs {
                // Skip tiny overlaps to prevent jitter from numerical precision
                if collision.penetration < self.penetration_tolerance {
                    continue;
                }
                
                let (idx_a, idx_b) = (collision.a_index, collision.b_index);
                
                // Safe mutable access to two different particles
                if idx_a >= particles.len() || idx_b >= particles.len() {
                    continue;
                }
                
                // Get mutable references to both particles
                let (particle_a, particle_b) = if idx_a < idx_b {
                    let (left, right) = particles.split_at_mut(idx_b);
                    (&mut left[idx_a], &mut right[0])
                } else {
                    let (left, right) = particles.split_at_mut(idx_a);
                    (&mut right[0], &mut left[idx_b])
                };

                // Recompute collision normal and penetration with CURRENT positions
                // (previous resolutions may have moved particles since detection)
                let delta = particle_b.pos - particle_a.pos;
                let dist_sq = delta.dot(delta);
                let min_dist = particle_a.radius + particle_b.radius;
                let min_dist_sq = min_dist * min_dist;
                
                // Skip if no longer colliding (previous resolutions may have separated them)
                if dist_sq >= min_dist_sq {
                    continue;
                }
                
                let dist = dist_sq.sqrt();
                let penetration = min_dist - dist;
                
                // Skip tiny overlaps only after a few iterations
                if iteration > 2 && penetration < self.penetration_tolerance {
                    continue;
                }
                
                // Compute fresh normal (handle zero-distance case)
                let normal = if dist > 1e-10 {
                    delta / dist
                } else {
                    // Particles at exactly same position - push apart in arbitrary direction
                    DVec2::new(self.penetration_tolerance * 100.0, 0.0)
                };
                
                // Assume equal mass (inv_mass = 1.0) for now
                verlet::resolve_collision(
                    &mut particle_a.pos, &mut particle_a.pos_old, 1.0,
                    &mut particle_b.pos, &mut particle_b.pos_old, 1.0,
                    normal, penetration, iter_restitution,
                );
                
                resolved_count += 1;  
            }

            // Apply bounds inside the loop so wall contacts are visible
            // to subsequent collision passes. Only apply restitution on
            // first pass — same reasoning as iter_restitution above.
            let use_restitution = iteration == 0;
            for p in storage.iter_mut() {
                self.bounds.apply(&mut p.pos.x, &mut p.pos_old.x, &mut p.pos.y, &mut p.pos_old.y, use_restitution);
            }

          
            
            // If very few collisions resolved, we can exit early
            if resolved_count == 0 {
                break;
            }

        }
        
       // Final bounds pass with restitution to catch anything remaining
        for p in storage.iter_mut() {
            self.bounds.apply(&mut p.pos.x, &mut p.pos_old.x, &mut p.pos.y, &mut p.pos_old.y, true);
        }
        
        // Dampen very slow particles to prevent jitter
        const VELOCITY_SLEEP_THRESHOLD: f64 = 1e-4;  // Stop particles moving slower than this
        for p in storage.iter_mut() {
            let vel = p.pos - p.pos_old;
            let speed_sq = vel.dot(vel);
            if speed_sq < VELOCITY_SLEEP_THRESHOLD * VELOCITY_SLEEP_THRESHOLD {
                p.pos_old = p.pos;  // Set velocity to zero
            }
        }
    }

     
}