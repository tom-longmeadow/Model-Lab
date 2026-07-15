use crate::{math::{FloatScalar, Vector, VectorMask}, sim::solver::particle::{partition::collision::CollisionRegistry, tuning::ParticlePhysicsTuning}};
 
pub struct VerletPhysics; 
impl VerletPhysics { 

     #[inline(always)]
    pub fn update_kinetics<V>(
        dt: V::Scalar,
        tuning: &ParticlePhysicsTuning<V::Scalar>,
        pos: &mut V,
        pos_old: &mut V, 
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

    // #[inline(always)]
    // pub fn clamp_position_bounds<V>(
    //     min_bound: V,
    //     max_bound: V,
    //     radius: V::Scalar,
    //     pos: &mut V,
    // ) where 
    //     V: Vector
    // {
    //     let r_vec = V::splat(radius);
    //     let min_allowed = min_bound + r_vec;
    //     let max_allowed = max_bound - r_vec;

    //     // Hard clamp current position strictly inside the boundaries
    //     *pos = pos.max(min_allowed).min(max_allowed);
    // }

    // #[inline(always)]
    // pub fn apply_position_constraints<V>(
    //     dt: V::Scalar,
    //     tuning: &ParticlePhysicsTuning<V::Scalar>,
    //     min_bound: V,
    //     max_bound: V,
    //     radius: V::Scalar,
    //     pos: &mut V,
    //     pos_old: &mut V,
    // ) where 
    //     V: Vector
    // {
    //     // 1. Calculate implicit frame displacement (already scaled by dt implicitly)
    //     let vel = *pos - *pos_old;

    //     // 2. Expand radius and set limits
    //     let r = V::splat(radius);
    //     let min_collision_limit = min_bound + r;
    //     let max_collision_limit = max_bound - r;

    //     // 3. Generate component-wise collision masks
    //     let under_min_mask = pos.cmplt(min_collision_limit);
    //     let over_max_mask = pos.cmpgt(max_collision_limit);
    //     let collision_mask = V::mask_or(under_min_mask, over_max_mask);

    //     // 4. Correct positions immediately to prevent gluing to walls
    //     let mut new_pos = *pos;
    //     new_pos = V::select(under_min_mask, min_collision_limit, new_pos);
    //     new_pos = V::select(over_max_mask, max_collision_limit, new_pos);

    //     // 5. Clean bounce: Restitution is a ratio (0.0 to 1.0). Do NOT multiply by dt here!
    //     let bounced_vel_normal = (-vel) * tuning.restitution;

    //     // 6. Clean friction: Dampen non-colliding components securely
    //     let friction_diminish = V::Scalar::ONE - (dt * tuning.friction);
    //     let slowed_vel_tangential = vel * friction_diminish;

    //     // 7. Use SIMD selection to route velocity values axis-by-axis
    //     let new_vel = V::select(collision_mask, bounced_vel_normal, slowed_vel_tangential);

    //     // 8. Reconstruct Verlet history so next frame inherits the outward momentum
    //     *pos = new_pos;
    //     *pos_old = new_pos - new_vel;
    // }

     
    
    
   
// #[inline(always)] 
// pub fn apply_position_constraints<V>(
//     dt: V::Scalar,
//     tuning: &ParticlePhysicsTuning<V::Scalar>,
//     min_bound: V,
//     max_bound: V,
//     radius: V::Scalar,
//     pos: &mut V,
//     pos_old: &mut V,
// ) where 
//     V: Vector
// {
//     let vel = *pos - *pos_old;

//     let r = V::splat(radius);
//     let min_collision_limit = min_bound + r;
//     let max_collision_limit = max_bound - r;

//     let under_min_mask = pos.cmplt(min_collision_limit);
//     let over_max_mask = pos.cmpgt(max_collision_limit);
//     let collision_mask = V::mask_or(under_min_mask, over_max_mask);

//     if collision_mask.any() {
//         // 1. Correct positions immediately to prevent wall-penetration
//         let mut new_pos = *pos;
//         new_pos = V::select(under_min_mask, min_collision_limit, new_pos);
//         new_pos = V::select(over_max_mask, max_collision_limit, new_pos);

//         // 2. Load the central frame-based jitter vector
//         let base_noise = V::from_f64_array(tuning.runtime_jitter);
        
//         // 3. Separate raw bounce (normal) and standard sliding dampening (tangential)
//         let bounced_vel_normal = (-vel) * tuning.restitution;
//         let friction_diminish = V::Scalar::ONE - (dt * tuning.friction);
//         let slowed_vel_tangential = vel * friction_diminish;

//         // 4. ACTIVE SLIDE DETECTION & JITTER INJECTION
//         // We create an axis-inverted jitter multiplier. 
//         // If Axis X is hitting a wall, we want to inject jitter into Axis Y/Z to slide it sideways.
//         // We do this by swapping or combining the noise elements element-wise.
//         let one_with_noise = V::splat(V::Scalar::ONE) + base_noise;
        
//         // Apply jitter directly to both velocity pathways
//         let jittered_bounced_vel = bounced_vel_normal.mul_elementwise(one_with_noise);
        
//         // This line introduces a constant shuffling force along the wall even if 'vel' approaches zero,
//         // which prevents columns of particles from stabilizing against boundaries.
//         let jittered_tangential_vel = slowed_vel_tangential + base_noise * dt;

//         // 5. Select axis routes based on SIMD collision state
//         // Colliding axes get the inverted bounce reaction.
//         // Non-colliding axes get the jittered tangential/sliding velocity.
//         let new_vel = V::select(collision_mask, jittered_bounced_vel, jittered_tangential_vel);

//         *pos = new_pos;
//         *pos_old = new_pos - new_vel;
//     } else {
//         // OPEN AIR PATH: Zero noise calculations, completely pure execution branch
//         let friction_diminish = V::Scalar::ONE - (dt * tuning.friction);
//         let clean_slowed_vel_tangential = vel * friction_diminish;
        
//         *pos_old = *pos - clean_slowed_vel_tangential;
//     }
// }
     

     #[inline(always)] 
pub fn apply_position_constraints<V>(
    dt: V::Scalar,
    tuning: &ParticlePhysicsTuning<V::Scalar>,
    min_bound: V,
    max_bound: V,
    radius: V::Scalar,
    pos: &mut V,
    pos_old: &mut V,
) where 
    V: Vector
{
    let vel = *pos - *pos_old;

    let r = V::splat(radius);
    let min_collision_limit = min_bound + r;
    let max_collision_limit = max_bound - r;

    let under_min_mask = pos.cmplt(min_collision_limit);
    let over_max_mask = pos.cmpgt(max_collision_limit);
    let collision_mask = V::mask_or(under_min_mask, over_max_mask);

    if collision_mask.any() {
        // 1. Correct positions immediately to prevent wall-penetration
        let mut new_pos = *pos;
        new_pos = V::select(under_min_mask, min_collision_limit, new_pos);
        new_pos = V::select(over_max_mask, max_collision_limit, new_pos);

        // 2. Load the central frame-based jitter vector
        let base_noise = V::from_f64_array(tuning.runtime_jitter);
        
        // 3. Separate clean raw bounce (normal) and standard sliding dampening (tangential)
        let clean_bounced_vel_normal = (-vel) * tuning.restitution;
        let friction_diminish = V::Scalar::ONE - (dt * tuning.friction);
        let slowed_vel_tangential = vel * friction_diminish;

        // 4. ACTIVE SLIDE JITTER INJECTION
        // Introduce a constant shuffling force along the wall components.
        // This keeps particles fluidly moving sideways out of rigid stacks, even at rest.
        let jittered_tangential_vel = slowed_vel_tangential + base_noise * dt;

        // 5. Select axis routes based on SIMD collision state
        // - Colliding axes get the clean, un-jittered perpendicular reflection normal.
        // - Non-colliding axes get the jittered tangential/sliding velocity.
        let new_vel = V::select(collision_mask, clean_bounced_vel_normal, jittered_tangential_vel);

        *pos = new_pos;
        *pos_old = new_pos - new_vel;
    } else {
        // OPEN AIR PATH: Zero noise calculations, completely pure execution branch
        let friction_diminish = V::Scalar::ONE - (dt * tuning.friction);
        let clean_slowed_vel_tangential = vel * friction_diminish;
        
        *pos_old = *pos - clean_slowed_vel_tangential;
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


//    #[inline(always)]
//     pub fn resolve_particle_collisions<V>(
//         tuning: &ParticlePhysicsTuning<V::Scalar>,
//         pos_a: &mut V,
//         pos_b: &mut V,
//         radius_a: V::Scalar,
//         radius_b: V::Scalar,
//         inv_mass_a: V::Scalar,
//         inv_mass_b: V::Scalar,
//     ) where 
//         V: Vector 
//     {
//         let delta = *pos_a - *pos_b;
//         let dist_sq = delta.length_squared();
        
//         let target_dist = radius_a + radius_b;
//         let target_dist_sq = target_dist * target_dist;

//         if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
//             let dist = dist_sq.sqrt();
//             let raw_penetration = target_dist - dist;
            
//             if raw_penetration > tuning.penetration_slop {
//                 let penetration = raw_penetration - tuning.penetration_slop;
//                 let normal = delta / dist;

//                 let total_inv_mass = inv_mass_a + inv_mass_b;
//                 if total_inv_mass > V::Scalar::ZERO {
//                     let bias = tuning.penetration_correction_bias;
//                     let response_magnitude = (penetration * bias) / total_inv_mass;

//                     *pos_a += normal * (response_magnitude * inv_mass_a);
//                     *pos_b -= normal * (response_magnitude * inv_mass_b);
//                 }
//             }
//         }
//     }

#[inline(always)]
pub fn resolve_particle_collisions<V>(
    tuning: &ParticlePhysicsTuning<V::Scalar>,
    pos_a: &mut V,
    pos_b: &mut V,
    radius_a: V::Scalar,
    radius_b: V::Scalar,
    inv_mass_a: V::Scalar,
    inv_mass_b: V::Scalar,
) where 
    V: Vector 
{
    let target_dist = radius_a + radius_b;
    let target_dist_sq = target_dist * target_dist;

    let mut delta = *pos_a - *pos_b;
    let mut dist_sq = delta.length_squared();

    // --- CATCH FUSED PARTICLES ---
    // Zero heap allocations. Use your fast from_f64_array to inject a tiny separation offset.
    if dist_sq == V::Scalar::ZERO {
        let mut sep_arr = [0.0; 4];
        sep_arr[0] = 0.0001; // Tiny displacement along the X-axis
        delta = V::from_f64_array(sep_arr);
        dist_sq = delta.length_squared();
    }

    if dist_sq < target_dist_sq {
        let dist = dist_sq.sqrt();
        let raw_penetration = target_dist - dist;
        
        if raw_penetration > tuning.penetration_slop {
            let penetration = raw_penetration - tuning.penetration_slop;
            
            // Calculate a raw normal direction
            let mut normal = delta / dist;

            // --- UNIFIED HIGH-PERFORMANCE JITTER ---
            // Load the pre-calculated frame jitter directly into registers.
            let base_jitter = V::from_f64_array(tuning.runtime_jitter);
            
            // Mix the noise with the collision normal to randomize the perturbation direction.
            // This ensures particles crashing from different angles get unique sideways shuffles,
            // destroying vertical stacking grids with zero branch or trigonometry overhead.
            let jitter_vec = normal.mul_elementwise(base_jitter);

            // Perturb the normal and re-normalize to maintain vector unit length integrity
            normal = normal + jitter_vec;
            let normal_len_sq = normal.length_squared();
            if normal_len_sq > V::Scalar::ZERO {
                normal = normal / normal_len_sq.sqrt();
            }

            // --- RESOLVE POSITIONS ---
            let total_inv_mass = inv_mass_a + inv_mass_b;
            if total_inv_mass > V::Scalar::ZERO {
                let bias = tuning.penetration_correction_bias;
                let response_magnitude = (penetration * bias) / total_inv_mass;

                *pos_a += normal * (response_magnitude * inv_mass_a);
                *pos_b -= normal * (response_magnitude * inv_mass_b);
            }
        }
    }
}

    /// 2. Restitution-only history modification (Runs ONCE after the loops close)
    #[inline(always)]
    pub fn apply_particle_restitution<V>(
        tuning: &ParticlePhysicsTuning<V::Scalar>,
        pos_a: &V,
        pos_b: &V,
        pos_old_a: &mut V,
        pos_old_b: &mut V,
        radius_a: V::Scalar,
        radius_b: V::Scalar,
        inv_mass_a: V::Scalar,
        inv_mass_b: V::Scalar,
    ) where 
        V: Vector 
    {
        let delta = *pos_a - *pos_b;
        let dist_sq = delta.length_squared();
        
        // Add a slight extra margin to catch particles that were just resolved and are touching
        let target_dist = radius_a + radius_b + tuning.penetration_slop;
        let target_dist_sq = target_dist * target_dist;

        if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
            let dist = dist_sq.sqrt();
            let normal = delta / dist;

            let total_inv_mass = inv_mass_a + inv_mass_b;
            if total_inv_mass > V::Scalar::ZERO {
                // Read velocities after all position corrections have completed
                let vel_a = *pos_a - *pos_old_a;
                let vel_b = *pos_b - *pos_old_b;
                let relative_vel = vel_a - vel_b;

                let normal_vel_mag = relative_vel.dot(normal);

                // Only bounce if they are traveling towards each other
                if normal_vel_mag < V::Scalar::ZERO {
                    let target_normal_vel = -normal_vel_mag * tuning.restitution;
                    let delta_vel_mag = target_normal_vel - normal_vel_mag;

                    let vel_impulse_mag = delta_vel_mag / total_inv_mass;
                    let vel_change_vector = normal * vel_impulse_mag;

                    // Modify history registers cleanly exactly once
                    *pos_old_a -= vel_change_vector * inv_mass_a;
                    *pos_old_b += vel_change_vector * inv_mass_b;
                }
            }
        }
    }
    

     // #[inline(always)]
    // pub fn apply_collision_restitution<V>(
    //     dt: V::Scalar,
    //     tuning: &ParticlePhysicsTuning<V::Scalar>,
    //     relaxed_pos_a: &V,     // The final relaxed position (for normal vector)
    //     relaxed_pos_b: &V,     // The final relaxed position (for normal vector)
    //     unrelaxed_pos_a: &V,   // Snapshot taken BEFORE relaxation
    //     unrelaxed_pos_b: &V,   // Snapshot taken BEFORE relaxation
    //     pos_old_a: &mut V,     // Modifies state
    //     pos_old_b: &mut V,     // Modifies state
    //     radius_a: V::Scalar,
    //     radius_b: V::Scalar,
    // ) where 
    //     V: Vector 
    // {
    //     type S<V> = <V as Vector>::Scalar;

    //     // --- 1. GEOMETRY CHECK (Using relaxed positions for current contact normal) ---
    //     let delta = *relaxed_pos_b - *relaxed_pos_a;
    //     let distance_sq = delta.dot(delta);
    //     let min_dist = radius_a + radius_b;

    //     if distance_sq < min_dist * min_dist && distance_sq > S::<V>::ZERO {
    //         let distance = distance_sq.sqrt();
    //         let normal = delta / distance;

    //         // --- 2. CRITICAL FIX: TRUE KINETIC VELOCITY (Using unrelaxed snapshot) ---
    //         // This stops positional shoves from pretending to be real velocity energy.
    //         let vel_a = *unrelaxed_pos_a - *pos_old_a;
    //         let vel_b = *unrelaxed_pos_b - *pos_old_b;

    //         // --- 3. RELATIVE VELOCITY PROCESSING ---
    //         let relative_vel = vel_b - vel_a;
    //         let vel_along_normal = relative_vel.dot(normal);

    //         // Only process if they are physically moving TOWARD each other
    //         if vel_along_normal < S::<V>::ZERO {
    //             // --- 4. RESTITUTION (Normal Axis) ---
    //             let speed = vel_along_normal.abs();
    //             let sub_step_bounce_threshold = tuning.velocity_bounce_threshold * dt;
    //             let restitution_coeff = tuning.restitution;

    //             let bounce_factor = if speed < sub_step_bounce_threshold {
    //                 S::<V>::ZERO 
    //             } else {
    //                 restitution_coeff
    //             };

    //             let half = S::<V>::from_f64(0.5);
    //             let one = S::<V>::ONE;

    //             // Keep this scalar negative (representing incoming relative velocity)
    //             let restitution_scalar = vel_along_normal * (one + bounce_factor) * half;
    //             let restitution_impulse = normal * restitution_scalar;

    //             // --- 5. FRICTION (Tangent Axis) ---
    //             let tangent_vel = relative_vel - (normal * vel_along_normal);
    //             let friction_impulse = tangent_vel * (tuning.friction * half);

    //             // --- 6. APPLY IMPULSES TO VERLET STATE ---
    //             // Subtracting negative restitution impulse pushes pos_old backward, creating a bounce.
    //             // Adding/Subtracting friction impulse pulls pos_old closer to pos, dampening the slide.
    //             *pos_old_a -= restitution_impulse; 
    //             *pos_old_b += restitution_impulse;

    //             *pos_old_a += friction_impulse; 
    //             *pos_old_b -= friction_impulse;
    //         }
    //     }
    // }
    // #[inline(always)]
    // pub fn resolve_particle_collisions<V>(
    //     tuning: &ParticlePhysicsTuning<V::Scalar>,
    //     pos_a: &mut V,
    //     pos_b: &mut V,
    //     radius_a: V::Scalar,
    //     radius_b: V::Scalar,
    //     inv_mass_a: V::Scalar,
    //     inv_mass_b: V::Scalar,
    // ) where 
    //     V: Vector 
    // {
    //     let delta = *pos_b - *pos_a;
    //     let distance_sq = delta.dot(delta);
    //     let min_dist = radius_a + radius_b;
    //     let min_dist_sq = min_dist * min_dist;
    //     let zero = <V::Scalar as FloatScalar>::ZERO;

    //     if distance_sq < min_dist_sq && distance_sq > zero {
    //         let distance = distance_sq.sqrt();
    //         let penetration = min_dist - distance;
    //         let slop = tuning.penetration_slop;
            
    //         if penetration > slop {
    //             let total_inv_mass = inv_mass_a + inv_mass_b;
    //             if total_inv_mass > zero {
    //                 let normal = delta / distance;
                    
    //                 // --- POSITIONAL RESOLUTION ONLY ---
    //                 // We only separate current positions here.
    //                 // We do NOT modify pos_old or calculate bounce impulses.
    //                 let bias = tuning.penetration_correction_bias;
    //                 let corrected_penetration = (penetration - slop) * bias;
    //                 let mass_normalized_penetration = corrected_penetration / total_inv_mass;

    //                 let separation_a = normal * (mass_normalized_penetration * inv_mass_a);
    //                 let separation_b = normal * (mass_normalized_penetration * inv_mass_b);

    //                 *pos_a -= separation_a;
    //                 *pos_b += separation_b;
    //             }
    //         }
    //     }
    // }


     // /// Proportions a particle's position from an old window size to a new window size. 
    // #[inline(always)]
    // pub fn scale_to_bounds<V>(
    //     tuning: &ParticlePhysicsTuning<V::Scalar>,
    //     pos: &mut V, 
    //     pos_old: &mut V,
    //     old_min: V, 
    //     old_max: V, 
    //     new_min: V, 
    //     new_max: V
    // ) where 
    //     V: Vector
    // {
    //     let old_range = old_max - old_min;
    //     let zero = V::ZERO; 
    //     let valid_mask = old_range.cmpgt(zero);

    //     // 2. Prevent division-by-zero crashes on hardware
    //     // Where valid_mask is false (range <= 0.0), replace with 1.0. 
    //     // The calculated pct for these dead lanes will be wrong, but it is discarded later.
    //     let one_vector = V::splat(<V::Scalar as FloatScalar>::ONE);
    //     let safe_div = V::select(valid_mask, old_range, one_vector);

    //     // 3. Compute proportions elementwise
    //     let new_range = new_max - new_min;
    //     let pct = (*pos - old_min).div_elementwise(safe_div);
    //     let pct_old = (*pos_old - old_min).div_elementwise(safe_div);

    //     let candidate_pos = new_min + pct.mul_elementwise(new_range);
    //     let candidate_pos_old = new_min + pct_old.mul_elementwise(new_range);

    //     // 4. Conditional write-back (Only apply changes if old_range > 0.0)
    //     *pos = V::select(valid_mask, candidate_pos, *pos);
    //     *pos_old = V::select(valid_mask, candidate_pos_old, *pos_old);
    // }

}
 