 use crate::{math::DVec2, sim::solver::tuning::ParticlePhysicsTuning};

pub struct VerletSolver;
impl VerletSolver {


    #[inline(always)]
    pub fn update_kinetics(
        tuning: &ParticlePhysicsTuning,
        pos: &mut f64,
        pos_old: &mut f64,
        dt: f64,
        acc: &mut f64,
    ) {
        let temp_pos = *pos;
        
        // 1. Calculate the implicit velocity vector (displacement)
        let displacement = temp_pos - *pos_old;

        // 2. Calculate frame-rate independent damping factor
        // tuning.global_damping = 0.0 means no drag. Higher numbers mean more drag.
        let damping_factor = (-tuning.global_damping * dt).exp();

        // 3. Apply Verlet integration: Next = Current + (Displacement * Damping) + (Acc * dt^2)
        let next_pos = temp_pos + (displacement * damping_factor) + (*acc * dt * dt);

        *pos = next_pos;
        *pos_old = temp_pos;
        *acc = 0.0;
    }

   
    #[inline(always)]
    fn apply_position_constraints(
        tuning: &ParticlePhysicsTuning,
        min: f64,
        max: f64,
        radius: f64,
        pos: &mut f64,
        pos_old: &mut f64,
    ) {
        // --- MINIMUM BOUNDARY ---
        let min_allowed = min + radius;
        if *pos < min_allowed {
            let velocity = *pos - *pos_old; 
            if velocity <= 0.0 {
                *pos = min_allowed;

                if velocity.abs() < tuning.velocity_bounce_threshold {
                    *pos_old = min_allowed;
                } else {
                    // FIX: Add the flipped velocity. 
                    // Since velocity is negative, subtracting a negative would be wrong.
                    // We want pos_old to be LESS than min_allowed so the next frame moves right.
                    *pos_old = min_allowed - (velocity.abs() * tuning.restitution);
                }
            }
        }

        // --- MAXIMUM BOUNDARY ---
        let max_allowed = max - radius;
        if *pos > max_allowed {
            let velocity = *pos - *pos_old; 
            if velocity >= 0.0 {
                *pos = max_allowed;

                if velocity.abs() < tuning.velocity_bounce_threshold {
                    *pos_old = max_allowed;
                } else {
                    // FIX: We want pos_old to be GREATER than max_allowed so the next frame moves left.
                    *pos_old = max_allowed + (velocity.abs() * tuning.restitution);
                }
            }
        }
    }


    #[inline(always)]
    pub fn apply_position_constraints_2d(
        tuning: &ParticlePhysicsTuning,
        min_bound: DVec2, // (min_x, min_y)
        max_bound: DVec2, // (max_x, max_y)
        radius: f64,
        pos: &mut DVec2,
        pos_old: &mut DVec2,
    ) {
        // 1. Capture initial positions to detect if a boundary correction happens
        let initial_pos = *pos;
        let velocity = *pos - *pos_old;

        // 2. Run the X-axis constraints using your original 1D function
        Self::apply_position_constraints(
            tuning,
            min_bound.x,
            max_bound.x,
            radius,
            &mut pos.x,
            &mut pos_old.x,
        );

        // If X position changed, a wall was hit -> Apply friction to the Y velocity
        if pos.x != initial_pos.x {
            let tangent_friction = velocity.y * tuning.friction;
            pos_old.y += tangent_friction;
        }

        // 3. Run the Y-axis constraints using your original 1D function
        Self::apply_position_constraints(
            tuning,
            min_bound.y,
            max_bound.y,
            radius,
            &mut pos.y,
            &mut pos_old.y,
        );

        // If Y position changed, a floor/ceiling was hit -> Apply friction to the X velocity
        if pos.y != initial_pos.y {
            let tangent_friction = velocity.x * tuning.friction;
            pos_old.x += tangent_friction;
        }
    }
    

    /// Forces a particle to stay within the bounds instantly. 
    /// Call this loop across all particles when the window resizes if you want to snap them to the edge.
    #[inline(always)]
    pub fn clamp_to_bounds_1d(pos: &mut f64, min: f64, max: f64, radius: f64) {
        let min_allowed = min + radius;
        let max_allowed = max - radius;

        if *pos < min_allowed {
            *pos = min_allowed;
        } else if *pos > max_allowed {
            *pos = max_allowed;
        }
    }

    /// Proportions a particle's position from an old window size to a new window size.
    /// Call this across all particles during a resize to prevent layout squishing issues.
     #[inline(always)]
    pub fn scale_to_bounds(
        pos: &mut f64, 
        pos_old: &mut f64,
        old_min: f64, 
        old_max: f64, 
        new_min: f64, 
        new_max: f64
    ) {
        let old_range = old_max - old_min;
        if old_range <= 0.0 { return; } // Prevent division by zero

        // Calculate relative position factor (0.0 to 1.0)
        let pct = (*pos - old_min) / old_range;
        let pct_old = (*pos_old - old_min) / old_range;

        let new_range = new_max - new_min;
        *pos = new_min + (pct * new_range);
        *pos_old = new_min + (pct_old * new_range);
    }

    /// 2D wrapper that routes both dimensions through the 1D scale logic.
    #[inline(always)]
    pub fn scale_to_bounds_2d(
        pos: &mut DVec2, 
        pos_old: &mut DVec2,
        old_min: DVec2, 
        old_max: DVec2, 
        new_min: DVec2, 
        new_max: DVec2
    ) {
        // Process X Axis
        Self::scale_to_bounds(
            &mut pos.x, 
            &mut pos_old.x, 
            old_min.x, 
            old_max.x, 
            new_min.x, 
            new_max.x
        );

        // Process Y Axis
        Self::scale_to_bounds(
            &mut pos.y, 
            &mut pos_old.y, 
            old_min.y, 
            old_max.y, 
            new_min.y, 
            new_max.y
        );
    }

     
    /// Applies velocity-based restitution and surface friction to a colliding pair exactly once.
    /// Run this loop once across your active collision registry BEFORE the relaxation loop.
    #[inline(always)]
    pub fn apply_collision_restitution(
        tuning: &ParticlePhysicsTuning,
        pos_a: &DVec2,
        pos_b: &DVec2,
        pos_old_a: &mut DVec2,
        pos_old_b: &mut DVec2,
        radius_a: f64,
        radius_b: f64,
    ) {
        let delta = *pos_b - *pos_a;
        let distance_sq = delta.dot(delta);
        let min_dist = radius_a + radius_b;

        if distance_sq < min_dist * min_dist && distance_sq > 0.0 {
            let distance = distance_sq.sqrt();
            let normal = delta / distance;

            // 1. Calculate implicit velocities
            let vel_a = *pos_a - *pos_old_a;
            let vel_b = *pos_b - *pos_old_b;

            // 2. Find relative velocity
            let relative_vel = vel_b - vel_a;
            let vel_along_normal = relative_vel.dot(normal);

            // 3. Only process if they are moving TOWARD each other or sliding against each other
            if vel_along_normal < 0.0 {
                // --- RESTITUTION (Normal Axis) ---
                let speed = vel_along_normal.abs();
                let bounce_factor = if speed < tuning.velocity_bounce_threshold {
                    0.0 
                } else {
                    tuning.restitution
                };

                let restitution_impulse = normal * (vel_along_normal * (1.0 + bounce_factor) * 0.5);

                // --- FRICTION (Tangent Axis) ---
                // Extract the component of relative velocity moving along the surface
                let tangent_vel = relative_vel - (normal * vel_along_normal);
                
                // Assume tuning.friction is a standard coefficient (e.g., 0.1 for ice, 0.8 for rubber)
                // Divide by 2.0 to distribute the friction impulse evenly between both particles
                let friction_impulse = tangent_vel * (tuning.friction * 0.5);

                // --- APPLY IMPULSES TO VERLET STATE ---
                // Restitution pushes them apart; Friction opposes the sliding direction
                *pos_old_a += restitution_impulse + friction_impulse;
                *pos_old_b -= restitution_impulse + friction_impulse;
            }
        }
    }


    // /// 1. THE KINETIC PIPELINE (Unchanged)
    // #[inline(always)]
    // pub fn update_kinetics_1d(
    //     tuning: &ParticlePhysicsTuning,
    //     pos: &mut f64,
    //     pos_old: &mut f64,
    //     dt: f64,
    //     acc: &mut f64,
    // ) -> bool {
    //     let temp_pos = *pos;
    //     let dynamic_displacement = (temp_pos - *pos_old) * tuning.global_damping;
    //     let next_pos = temp_pos + dynamic_displacement + (*acc * dt * dt);

    //     let total_displacement = next_pos - temp_pos;
    //     let current_speed = total_displacement.abs() / dt;
    //     let mut speed_was_clamped = false;

    //     if current_speed > tuning.max_velocity && current_speed > 0.0 {
    //         let max_allowed_displacement = tuning.max_velocity * dt;
    //         let direction = total_displacement.signum();
            
    //         *pos = temp_pos + (direction * max_allowed_displacement);
    //         speed_was_clamped = true;
    //     } else {
    //         *pos = next_pos;
    //     }

    //     *pos_old = temp_pos;
    //     *acc = 0.0;

    //     speed_was_clamped
    // }

    // /// 2. PURE POSITION CONSTRAINT PIPELINE (Unchanged)
    // #[inline(always)]
    // pub fn apply_axis_position_constraints_1d(
    //     tuning: &ParticlePhysicsTuning,
    //     min: f64,
    //     max: f64,
    //     radius: f64,
    //     pos: &mut f64,
    // ) {
    //     let pen_min = min - (*pos - radius); 
    //     if pen_min > tuning.penetration_slop {
    //         *pos += 1.0 * (pen_min - tuning.penetration_slop);
    //     }

    //     let pen_max = (*pos + radius) - max; 
    //     if pen_max > tuning.penetration_slop {
    //         *pos += -1.0 * (pen_max - tuning.penetration_slop);
    //     }
    // }

    // /// 3. STEP A: PARTICLE POSITION SEPARATION (Unchanged)
    // #[inline(always)]
    // pub fn resolve_particle_position_1d(
    //     tuning: &ParticlePhysicsTuning,
    //     pos_a: &mut f64,
    //     inv_mass_a: f64,
    //     pos_b: &mut f64, 
    //     inv_mass_b: f64,
    //     normal_component: f64, 
    //     penetration: f64,      
    // ) -> bool {
    //     if penetration <= tuning.penetration_slop { return false; }
    //     let corrected_penetration = penetration - tuning.penetration_slop;

    //     let total_inv_mass = inv_mass_a + inv_mass_b;
    //     if total_inv_mass <= 0.0 { return false; }

    //     let correction_scalar = (corrected_penetration / total_inv_mass) * tuning.position_correction_bias;
    //     let correction_a = normal_component * (correction_scalar * inv_mass_a);
    //     let correction_b = normal_component * (correction_scalar * inv_mass_b);

    //     *pos_a -= correction_a;
    //     *pos_b += correction_b;

    //     true
    // }

    // /// 4. FINAL POST-LOOP VELOCITY RESOLVER (Uses your custom bounce threshold)
    // /// If speed is below the threshold, it zeroes out velocity along the wall normal.
    // #[inline(always)]
    // pub fn resolve_final_wall_velocity_1d(
    //     tuning: &ParticlePhysicsTuning,
    //     min: f64,
    //     max: f64,
    //     radius: f64,
    //     pos: &mut f64,
    //     pos_old: &mut f64,
    //     dt: f64,
    // ) -> bool {
    //     let left_bound = min + radius;
    //     let right_bound = max - radius;
        
    //     let mut normal = 0.0;
    //     let mut is_colliding = false;

    //     if *pos <= left_bound + tuning.penetration_slop {
    //         normal = 1.0;
    //         is_colliding = true;
    //     } else if *pos >= right_bound - tuning.penetration_slop {
    //         normal = -1.0;
    //         is_colliding = true;
    //     }

    //     if *pos < left_bound { *pos = left_bound; }
    //     if *pos > right_bound { *pos = right_bound; }

    //     if !is_colliding { return false; }

    //     let implicit_vel = (*pos - *pos_old) / dt;
    //     let relative_velocity = -(implicit_vel * normal);

    //     if relative_velocity > 0.0 {
    //         if relative_velocity > tuning.velocity_bounce_threshold {
    //             // Dynamic Energetic Bounce: Keep restitution bounciness
    //             let new_relative_velocity = relative_velocity * tuning.restitution;
    //             let new_implicit_vel = implicit_vel + normal * (relative_velocity + new_relative_velocity);
    //             *pos_old = *pos - (new_implicit_vel * dt);
    //         } else {
    //             // Resting Contact: HARD STOP. Match wall speed completely to kill the jitter
    //             let resting_implicit_vel = implicit_vel + normal * relative_velocity;
    //             *pos_old = *pos - (resting_implicit_vel * dt);
    //         }
    //         return true;
    //     }

    //     false
    // }

    // /// STEP B: PARTICLE VELOCITY IMPULSE APPLIER (Safely applies velocity corrections)
    // /// Cleans up history changes without compounding tiny floating point errors.
    // #[inline(always)]
    // pub fn apply_particle_velocity_impulse_1d(
    //     pos_old_a: &mut f64,
    //     inv_mass_a: f64,
    //     pos_old_b: &mut f64,
    //     inv_mass_b: f64,
    //     normal_component: f64, 
    //     impulse_vel: f64,      
    //     dt: f64,
    // ) -> bool {
    //     // Stop tiny micro-adjustments from triggering floating point thrashing
    //     if impulse_vel == 0.0 { return false; }

    //     *pos_old_a -= (normal_component * impulse_vel * inv_mass_a) * dt;
    //     *pos_old_b += (normal_component * impulse_vel * inv_mass_b) * dt;

    //     true
    // }
}
