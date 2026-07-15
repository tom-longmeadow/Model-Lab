
// =========================================================================
// 2. VERLET SOLVER IMPLEMENTATION
// =========================================================================

use crate::{math::{FloatScalar, Vector, VectorMask}, sim::solver::particle::{partition::collision::CollisionRegistry, tuning::ParticlePhysicsTuning}};
 
pub struct VerletPhysics; 
impl VerletPhysics {

 

     #[inline(always)]
    pub fn update_kinetics<V>(
        tuning: &ParticlePhysicsTuning<V::Scalar>,
        pos: &mut V,
        pos_old: &mut V,
        dt: V::Scalar,
        acc: &mut V,
    ) where 
        V: Vector
    {
        let temp_pos = *pos;
        
        // 1. Calculate the implicit velocity vector (displacement)
        let displacement = temp_pos - *pos_old;

        // 2. Calculate frame-rate independent damping factor 
        let damping_val = -tuning.global_damping * dt;
        let damping_factor = damping_val.exp();

        // 3. Apply Verlet integration with correct Vector * Scalar ordering
        let dt_sq = dt * dt;
        
        // Enforce (Vector * Scalar) layout to match your trait boundaries
        let damped_displacement = displacement * damping_factor;
        let accelerated_displacement = *acc * dt_sq;
        
        let next_pos = temp_pos + damped_displacement + accelerated_displacement;

        *pos = next_pos;
        *pos_old = temp_pos;
        *acc = V::ZERO;
    }
     

    /// Proportions a particle's position from an old window size to a new window size.
    /// Bit-perfect translation of the 1D algorithm handling zero ranges gracefully.
    /// Proportions a particle's position from an old window size to a new window size.
    /// Bit-perfect translation of the 1D algorithm handling zero ranges gracefully.
    #[inline(always)]
    pub fn scale_to_bounds<V>(
        pos: &mut V, 
        pos_old: &mut V,
        old_min: V, 
        old_max: V, 
        new_min: V, 
        new_max: V
    ) where 
        V: Vector
    {
        let old_range = old_max - old_min;
        let zero = V::ZERO;

        // 1. Identify valid axes (Strictly equivalent to 1D: old_range > 0.0)
        let valid_mask = old_range.cmpgt(zero);

        // 2. Prevent division-by-zero crashes on hardware
        // Where valid_mask is false (range <= 0.0), replace with 1.0. 
        // The calculated pct for these dead lanes will be wrong, but it is discarded later.
        let one_vector = V::splat(<V::Scalar as FloatScalar>::ONE);
        let safe_div = V::select(valid_mask, old_range, one_vector);

        // 3. Compute proportions elementwise
        let new_range = new_max - new_min;
        let pct = (*pos - old_min).div_elementwise(safe_div);
        let pct_old = (*pos_old - old_min).div_elementwise(safe_div);

        let candidate_pos = new_min + pct.mul_elementwise(new_range);
        let candidate_pos_old = new_min + pct_old.mul_elementwise(new_range);

        // 4. Conditional write-back (Only apply changes if old_range > 0.0)
        *pos = V::select(valid_mask, candidate_pos, *pos);
        *pos_old = V::select(valid_mask, candidate_pos_old, *pos_old);
    }

 

    #[inline(always)]
    pub fn apply_position_constraints<V>(
        tuning: &ParticlePhysicsTuning<V::Scalar>,
        min_bound: V,
        max_bound: V,
        radius: V::Scalar,
        pos: &mut V,
        pos_old: &mut V,
    ) where 
        V: Vector
    {


        // 1. Calculate the implicit velocity vector
        let velocity = *pos - *pos_old;
        let zero = V::ZERO;

        // 2. Define safety boundaries accounting for the radius
        let r_vec = V::splat(radius);
        let min_allowed = min_bound + r_vec;
        let max_allowed = max_bound - r_vec;

        // 3. Force the current position strictly inside the boundaries
        let clamped_pos = pos.max(min_allowed).min(max_allowed);

        // 4. Directional Collision Detection
        let violates_min = pos.cmplt(min_allowed);
        let moving_into_min = velocity.cmplt(zero); 
        let hit_min = V::mask_and(violates_min, moving_into_min);

        let violates_max = pos.cmpgt(max_allowed);
        let moving_into_max = velocity.cmpgt(zero); 
        let hit_max = V::mask_and(violates_max, moving_into_max);

        // Combine to find exactly which axes suffered an impact this frame
        let collided = V::mask_or(hit_min, hit_max);

        // 5. Rest Threshold & Bounce Processing
        let v_abs = V::select(velocity.cmplt(zero), zero - velocity, velocity);
        let thresh_vec = V::splat(tuning.velocity_bounce_threshold);
        let is_resting = v_abs.cmplt(thresh_vec);

        // --- FIX STARTS HERE ---
        // Explicitly identify lanes that are BOTH colliding and below the rest threshold
        let hard_stop_mask = V::mask_and(collided, is_resting);

        // Determine restitution factor lane-by-lane:
        let rest_vec = V::splat(tuning.restitution);
        let one_vec = V::splat(<V::Scalar as FloatScalar>::ONE);

        // If colliding and bouncing: multiplier is -restitution. 
        // If resting, it will be overridden by the hard stop mask below.
        let bounce_multiplier = zero - rest_vec;
        let velocity_multiplier = V::select(collided, bounce_multiplier, one_vec);

        let mut post_bounce_velocity = velocity.mul_elementwise(velocity_multiplier);
        
        // Hard clamp the velocity to zero for resting collisions to kill numerical jitter
        post_bounce_velocity = V::select(hard_stop_mask, zero, post_bounce_velocity);
        // --- FIX ENDS HERE ---

        // 6. Corner Friction: Apply friction to axes that DID NOT collide,
        // but ONLY if at least one other axis DID collide.
        let f_vec = V::splat(<V::Scalar as FloatScalar>::ONE - tuning.friction);

        let friction_multiplier = if collided.any() {
            V::select(collided, one_vec, f_vec)
        } else {
            one_vec
        };
        let final_velocity = post_bounce_velocity.mul_elementwise(friction_multiplier);

        // 7. Write back to state
        *pos = clamped_pos;
        
        // Force pos_old to exactly match clamped_pos on resting axes, 
        // ensuring zero velocity carries over to the next frame.
        *pos_old = clamped_pos - final_velocity;
        
        // // 1. Calculate the implicit velocity vector
        // let velocity = *pos - *pos_old;
        // let zero = V::ZERO;

        // // 2. Define safety boundaries accounting for the radius
        // let r_vec = V::splat(radius);
        // let min_allowed = min_bound + r_vec;
        // let max_allowed = max_bound - r_vec;

        // // 3. Force the current position strictly inside the boundaries
        // let clamped_pos = pos.max(min_allowed).min(max_allowed);

        // // 4. Directional Collision Detection
        // let violates_min = pos.cmplt(min_allowed);
        // let moving_into_min = velocity.cmplt(zero); 
        // let hit_min = V::mask_and(violates_min, moving_into_min);

        // let violates_max = pos.cmpgt(max_allowed);
        // let moving_into_max = velocity.cmpgt(zero); 
        // let hit_max = V::mask_and(violates_max, moving_into_max);

        // // Combine to find exactly which axes suffered an impact this frame
        // let collided = V::mask_or(hit_min, hit_max);

        // // 5. Rest Threshold & Bounce Processing
        // let v_abs = V::select(velocity.cmplt(zero), zero - velocity, velocity);
        // let thresh_vec = V::splat(tuning.velocity_bounce_threshold);
        // let is_resting = v_abs.cmplt(thresh_vec);

        // // Determine restitution factor lane-by-lane:
        // // - If colliding and resting: multiplier is 0.0 (stop)
        // // - If colliding and bouncing: multiplier is -restitution (reverse)
        // // - If not colliding: multiplier is 1.0 (maintain velocity)
        // let rest_vec = V::splat(tuning.restitution);
        // let one_vec = V::splat(<V::Scalar as FloatScalar>::ONE);

        // let bounce_multiplier = V::select(is_resting, zero, zero - rest_vec);
        // let velocity_multiplier = V::select(collided, bounce_multiplier, one_vec);

        // let post_bounce_velocity = velocity.mul_elementwise(velocity_multiplier);

        // // 6. Corner Friction: Apply friction to axes that DID NOT collide,
        // // but ONLY if at least one other axis DID collide.
        // let f_vec = V::splat(<V::Scalar as FloatScalar>::ONE - tuning.friction);

        // let friction_multiplier = if collided.any() {
        //     V::select(collided, one_vec, f_vec)
        // } else {
        //     one_vec
        // };
        // let final_velocity = post_bounce_velocity.mul_elementwise(friction_multiplier);

        // // 7. Write back to state
        // *pos = clamped_pos;
        // *pos_old = clamped_pos - final_velocity;

 
    }

   
 
        #[inline(always)]
        pub fn apply_collision_restitution<V>(
            tuning: &ParticlePhysicsTuning<V::Scalar>,
            pos_a: &V,
            pos_b: &V,
            pos_old_a: &mut V,
            pos_old_b: &mut V,
            radius_a: V::Scalar,
            radius_b: V::Scalar,
        ) where 
            V: Vector 
        {
            // Create local type aliases for cleaner math expressions
            type S<V> = <V as Vector>::Scalar;

            let delta = *pos_b - *pos_a;
            let distance_sq = delta.dot(delta);
            let min_dist = radius_a + radius_b;

            // Use your FloatScalar constants and comparison bounds safely
            if distance_sq < min_dist * min_dist && distance_sq > S::<V>::ZERO {
                let distance = distance_sq.sqrt();
                let normal = delta / distance;

                // 1. Calculate implicit velocities
                let vel_a = *pos_a - *pos_old_a;
                let vel_b = *pos_b - *pos_old_b;

                // 2. Find relative velocity
                let relative_vel = vel_b - vel_a;
                let vel_along_normal = relative_vel.dot(normal);

                // 3. Only process if they are moving TOWARD each other
                if vel_along_normal < S::<V>::ZERO {
                    // --- RESTITUTION (Normal Axis) ---
                    let speed = vel_along_normal.abs();
                    
                    // Map f64 tuning thresholds into your generic Precision scalar
                    let bounce_threshold = tuning.velocity_bounce_threshold;
                    let restitution_coeff = tuning.restitution;

                    let bounce_factor = if speed < bounce_threshold {
                        S::<V>::ZERO 
                    } else {
                        restitution_coeff
                    };

                    // Using 0.5 and 1.0 from FloatScalar conversions
                    let half = S::<V>::from_f64(0.5);
                    let one = S::<V>::ONE;
                    
                    let restitution_scalar = vel_along_normal * (one + bounce_factor) * half;
                    let restitution_impulse = normal * restitution_scalar;

                    // --- FRICTION (Tangent Axis) ---
                    // This tangent calculation is completely dimension-agnostic! 
                    // In 3D, it automatically isolates the 2D tangent sliding plane.
                    let tangent_vel = relative_vel - (normal * vel_along_normal);
                    
                    let friction_coeff = tuning.friction;
                    let friction_impulse = tangent_vel * (friction_coeff * half);

                    // --- APPLY IMPULSES TO VERLET STATE ---
                    let total_impulse = restitution_impulse + friction_impulse;
                    *pos_old_a += total_impulse;
                    *pos_old_b -= total_impulse;
                }
            }
        }
 
     

    /// Generic collision detector pushing cleanly to the abstract spatial registry.
    pub fn detect_collisions<V>(
        &self,
        len: usize,
        scratch_radii: &[V::Scalar],
        scratch_pos: &[V], 
        registry: &mut CollisionRegistry<V>
    ) where 
        V: Vector 
    {
        registry.clear();
        let zero = <V::Scalar as FloatScalar>::ZERO;
        
        for i in 0..len {
            let radius_a = scratch_radii[i];
            let pos_a = scratch_pos[i];

            for j in (i + 1)..len {
                let radius_b = scratch_radii[j];
                let pos_b = scratch_pos[j];

                let delta = pos_b - pos_a; 
                let distance_sq = delta.dot(delta);
                let min_dist = radius_a + radius_b;

                // Explicitly bringing FloatScalar methods into standard evaluation order
                if distance_sq < (min_dist * min_dist) && distance_sq > zero {
                    let distance = distance_sq.sqrt();
                    let penetration = min_dist - distance;
                    
                    // Matches V / V::Scalar layout perfectly
                    let normal = delta / distance; 

                    registry.push(i, j, normal, penetration);
                }
            }
        }
    }

    

    /// Generic relaxation-pass loop resolving overlaps using Baumgarte adjustments.
   #[inline(always)]
    pub fn resolve_particle_collisions<V>(
        tuning: &ParticlePhysicsTuning<V::Scalar>,
        pos_a: &mut V,
        pos_old_a: &mut V, // Pass pos_old as well!
        pos_b: &mut V,
        pos_old_b: &mut V, // Pass pos_old as well!
        radius_a: V::Scalar,
        radius_b: V::Scalar,
        inv_mass_a: V::Scalar,
        inv_mass_b: V::Scalar,
    ) where 
        V: Vector 
    {
        let delta = *pos_b - *pos_a;
        let distance_sq = delta.dot(delta);
        let min_dist = radius_a + radius_b;
        let min_dist_sq = min_dist * min_dist;
        let zero = <V::Scalar as FloatScalar>::ZERO;

        if distance_sq < min_dist_sq && distance_sq > zero {
            let distance = distance_sq.sqrt();
            let penetration = min_dist - distance;
            let slop = tuning.penetration_slop;
            
            if penetration > slop {
                let total_inv_mass = inv_mass_a + inv_mass_b;
                if total_inv_mass > zero {
                    let normal = delta / distance;
                    
                    // --- 1. POSITIONAL RESOLUTION (Pushes them apart) ---
                    let bias = tuning.penetration_correction_bias;
                    let corrected_penetration = (penetration - slop) * bias;
                    let mass_normalized_penetration = corrected_penetration / total_inv_mass;

                    let separation_a = normal * (mass_normalized_penetration * inv_mass_a);
                    let separation_b = normal * (mass_normalized_penetration * inv_mass_b);

                    *pos_a -= separation_a;
                    *pos_b += separation_b;

                    // --- 2. VELOCITY RESOLUTION (Verlet Bounce) ---
                    // Calculate current implicit velocities
                    let vel_a = *pos_a - *pos_old_a;
                    let vel_b = *pos_b - *pos_old_b;
                    
                    // Find relative velocity along the collision normal line
                    let rel_vel = vel_b - vel_a;
                    let vel_along_normal = rel_vel.dot(normal);

                    // Only bounce if they are moving TOWARD each other
                    if vel_along_normal < zero {
                        let restitution = tuning.restitution;
                        let impulse_scalar = -(<V::Scalar as FloatScalar>::ONE + restitution) * vel_along_normal;
                        let impulse_per_mass = impulse_scalar / total_inv_mass;
                        
                        let impulse_vec = normal * impulse_per_mass;

                        // Update pos_old to set up the new implicit bounce velocity
                        // (Subtracting impulse from pos_old makes future frames move faster away)
                        *pos_old_a += impulse_vec * inv_mass_a;
                        *pos_old_b -= impulse_vec * inv_mass_b;
                    }
                }
            }
        }
    }
}
//  use crate::{math::DVec2, sim::solver::{partition::collision::CollisionRegistry, tuning::ParticlePhysicsTuning}};

// pub struct VerletSolver;
// impl VerletSolver {

//     pub fn detect_collisions(
//         &self,
//         len: usize,
//         scratch_radii: &[f64],
//         scratch_pos: &[DVec2], // Assuming a Vector type like Vec2{x, y}
//         registry: &mut CollisionRegistry<DVec2>
//     ) {
//         registry.clear();

       
        
//         for i in 0..len {
//             let radius_a = scratch_radii[i];
//             let pos_a = scratch_pos[i];

//             for j in (i + 1)..len {
//                 let radius_b = scratch_radii[j];
//                 let pos_b = scratch_pos[j];

//                 let delta = pos_b - pos_a; // Direction pointing from A to B
//                 let distance_sq = delta.dot(delta);
//                 let min_dist = radius_a + radius_b;

//                 if distance_sq < min_dist * min_dist && distance_sq > 0.0 {
//                     let distance = distance_sq.sqrt();
//                     let penetration = min_dist - distance;
//                     let normal = delta / distance; // Normalized vector A -> B

//                     // Push to your registry. Note the custom constructor logic!
//                     registry.push(i, j, normal, penetration);
//                 }
//             }
//         }
//     }


//     #[inline(always)]
//     pub fn resolve_particle_collisions(
//         tuning: &ParticlePhysicsTuning,
//         pos_a: &mut DVec2,
//         pos_b: &mut DVec2,
//         radius_a: f64,
//         radius_b: f64,
//     ) {
//         let delta = *pos_b - *pos_a;
//         let distance_sq = delta.dot(delta);
//         let min_dist = radius_a + radius_b;
//         let min_dist_sq = min_dist * min_dist;

//         if distance_sq < min_dist_sq && distance_sq > 0.0 {
//             let distance = distance_sq.sqrt();
//             let penetration = min_dist - distance;

//             // 1. Apply Penetration Slop: Ignore tiny overlaps to reduce microscopic jitter
//             if penetration > tuning.penetration_slop {
//                 let normal = delta / distance;

//                 // 2. Apply Baumgarte Bias: Only resolve a fraction of the overlap per iteration
//                 // This prevents the positional corrections from introducing wild phantom kinetic energy
//                 let corrected_penetration = (penetration - tuning.penetration_slop) * tuning.penetration_correction_bias;
//                 let separation = normal * (corrected_penetration * 0.5);

//                 *pos_a -= separation;
//                 *pos_b += separation;
//             }
//         }
//     }


//     #[inline(always)]
//     pub fn update_kinetics(
//         tuning: &ParticlePhysicsTuning,
//         pos: &mut f64,
//         pos_old: &mut f64,
//         dt: f64,
//         acc: &mut f64,
//     ) {
//         let temp_pos = *pos;
        
//         // 1. Calculate the implicit velocity vector (displacement)
//         let displacement = temp_pos - *pos_old;

//         // 2. Calculate frame-rate independent damping factor
//         // tuning.global_damping = 0.0 means no drag. Higher numbers mean more drag.
//         let damping_factor = (-tuning.global_damping * dt).exp();

//         // 3. Apply Verlet integration: Next = Current + (Displacement * Damping) + (Acc * dt^2)
//         let next_pos = temp_pos + (displacement * damping_factor) + (*acc * dt * dt);

//         *pos = next_pos;
//         *pos_old = temp_pos;
//         *acc = 0.0;
//     }

   
//     #[inline(always)]
//     fn apply_position_constraints(
//         tuning: &ParticlePhysicsTuning,
//         min: f64,
//         max: f64,
//         radius: f64,
//         pos: &mut f64,
//         pos_old: &mut f64,
//     ) {
//         // --- MINIMUM BOUNDARY ---
//         let min_allowed = min + radius;
//         if *pos < min_allowed {
//             let velocity = *pos - *pos_old; 
//             if velocity <= 0.0 {
//                 *pos = min_allowed;

//                 if velocity.abs() < tuning.velocity_bounce_threshold {
//                     *pos_old = min_allowed;
//                 } else {
//                     // FIX: Add the flipped velocity. 
//                     // Since velocity is negative, subtracting a negative would be wrong.
//                     // We want pos_old to be LESS than min_allowed so the next frame moves right.
//                     *pos_old = min_allowed - (velocity.abs() * tuning.restitution);
//                 }
//             }
//         }

//         // --- MAXIMUM BOUNDARY ---
//         let max_allowed = max - radius;
//         if *pos > max_allowed {
//             let velocity = *pos - *pos_old; 
//             if velocity >= 0.0 {
//                 *pos = max_allowed;

//                 if velocity.abs() < tuning.velocity_bounce_threshold {
//                     *pos_old = max_allowed;
//                 } else {
//                     // FIX: We want pos_old to be GREATER than max_allowed so the next frame moves left.
//                     *pos_old = max_allowed + (velocity.abs() * tuning.restitution);
//                 }
//             }
//         }
//     }


//     #[inline(always)]
//     pub fn apply_position_constraints_2d(
//         tuning: &ParticlePhysicsTuning,
//         min_bound: DVec2, // (min_x, min_y)
//         max_bound: DVec2, // (max_x, max_y)
//         radius: f64,
//         pos: &mut DVec2,
//         pos_old: &mut DVec2,
//     ) {
//         // 1. Capture initial positions to detect if a boundary correction happens
//         let initial_pos = *pos;
//         let velocity = *pos - *pos_old;

//         // 2. Run the X-axis constraints using your original 1D function
//         Self::apply_position_constraints(
//             tuning,
//             min_bound.x,
//             max_bound.x,
//             radius,
//             &mut pos.x,
//             &mut pos_old.x,
//         );

//         // If X position changed, a wall was hit -> Apply friction to the Y velocity
//         if pos.x != initial_pos.x {
//             let tangent_friction = velocity.y * tuning.friction;
//             pos_old.y += tangent_friction;
//         }

//         // 3. Run the Y-axis constraints using your original 1D function
//         Self::apply_position_constraints(
//             tuning,
//             min_bound.y,
//             max_bound.y,
//             radius,
//             &mut pos.y,
//             &mut pos_old.y,
//         );

//         // If Y position changed, a floor/ceiling was hit -> Apply friction to the X velocity
//         if pos.y != initial_pos.y {
//             let tangent_friction = velocity.x * tuning.friction;
//             pos_old.x += tangent_friction;
//         }
//     }
    

  
     
//     /// Applies velocity-based restitution and surface friction to a colliding pair exactly once.
//     /// Run this loop once across your active collision registry BEFORE the relaxation loop.
//     #[inline(always)]
//     pub fn apply_collision_restitution(
//         tuning: &ParticlePhysicsTuning,
//         pos_a: &DVec2,
//         pos_b: &DVec2,
//         pos_old_a: &mut DVec2,
//         pos_old_b: &mut DVec2,
//         radius_a: f64,
//         radius_b: f64,
//     ) {
//         let delta = *pos_b - *pos_a;
//         let distance_sq = delta.dot(delta);
//         let min_dist = radius_a + radius_b;

//         if distance_sq < min_dist * min_dist && distance_sq > 0.0 {
//             let distance = distance_sq.sqrt();
//             let normal = delta / distance;

//             // 1. Calculate implicit velocities
//             let vel_a = *pos_a - *pos_old_a;
//             let vel_b = *pos_b - *pos_old_b;

//             // 2. Find relative velocity
//             let relative_vel = vel_b - vel_a;
//             let vel_along_normal = relative_vel.dot(normal);

//             // 3. Only process if they are moving TOWARD each other or sliding against each other
//             if vel_along_normal < 0.0 {
//                 // --- RESTITUTION (Normal Axis) ---
//                 let speed = vel_along_normal.abs();
//                 let bounce_factor = if speed < tuning.velocity_bounce_threshold {
//                     0.0 
//                 } else {
//                     tuning.restitution
//                 };

//                 let restitution_impulse = normal * (vel_along_normal * (1.0 + bounce_factor) * 0.5);

//                 // --- FRICTION (Tangent Axis) ---
//                 // Extract the component of relative velocity moving along the surface
//                 let tangent_vel = relative_vel - (normal * vel_along_normal);
                
//                 // Assume tuning.friction is a standard coefficient (e.g., 0.1 for ice, 0.8 for rubber)
//                 // Divide by 2.0 to distribute the friction impulse evenly between both particles
//                 let friction_impulse = tangent_vel * (tuning.friction * 0.5);

//                 // --- APPLY IMPULSES TO VERLET STATE ---
//                 // Restitution pushes them apart; Friction opposes the sliding direction
//                 *pos_old_a += restitution_impulse + friction_impulse;
//                 *pos_old_b -= restitution_impulse + friction_impulse;
//             }
//         }
//     }

//       /// Forces a particle to stay within the bounds instantly. 
//     /// Call this loop across all particles when the window resizes if you want to snap them to the edge.
//     #[inline(always)]
//     pub fn clamp_to_bounds_1d(pos: &mut f64, min: f64, max: f64, radius: f64) {
//         let min_allowed = min + radius;
//         let max_allowed = max - radius;

//         if *pos < min_allowed {
//             *pos = min_allowed;
//         } else if *pos > max_allowed {
//             *pos = max_allowed;
//         }
//     }

//     /// Proportions a particle's position from an old window size to a new window size.
//     /// Call this across all particles during a resize to prevent layout squishing issues.
//      #[inline(always)]
//     pub fn scale_to_bounds(
//         pos: &mut f64, 
//         pos_old: &mut f64,
//         old_min: f64, 
//         old_max: f64, 
//         new_min: f64, 
//         new_max: f64
//     ) {
//         let old_range = old_max - old_min;
//         if old_range <= 0.0 { return; } // Prevent division by zero

//         // Calculate relative position factor (0.0 to 1.0)
//         let pct = (*pos - old_min) / old_range;
//         let pct_old = (*pos_old - old_min) / old_range;

//         let new_range = new_max - new_min;
//         *pos = new_min + (pct * new_range);
//         *pos_old = new_min + (pct_old * new_range);
//     }

//     /// 2D wrapper that routes both dimensions through the 1D scale logic.
//     #[inline(always)]
//     pub fn scale_to_bounds_2d(
//         pos: &mut DVec2, 
//         pos_old: &mut DVec2,
//         old_min: DVec2, 
//         old_max: DVec2, 
//         new_min: DVec2, 
//         new_max: DVec2
//     ) {
//         // Process X Axis
//         Self::scale_to_bounds(
//             &mut pos.x, 
//             &mut pos_old.x, 
//             old_min.x, 
//             old_max.x, 
//             new_min.x, 
//             new_max.x
//         );

//         // Process Y Axis
//         Self::scale_to_bounds(
//             &mut pos.y, 
//             &mut pos_old.y, 
//             old_min.y, 
//             old_max.y, 
//             new_min.y, 
//             new_max.y
//         );
//     }

 
// }
