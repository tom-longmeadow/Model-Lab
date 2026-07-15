use crate::{math::{FloatScalar, Vector, VectorMask}, sim::solver::particle::{environment::ParticleEnvironment, runtime::RuntimeState, space::{GridSpace, collision::CollisionRegistry}, tuning::PhysicsTuning}};
 
pub struct VerletPhysics; 
impl VerletPhysics { 

     #[inline(always)]
    pub fn update_kinetics<V>(
        dt: V::Scalar,
        env: &ParticleEnvironment<V>,
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
        let damping_val = -env.tuning.physics.global_damping * dt;
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

     

     #[inline(always)] 
pub fn apply_position_constraints<V>(
    dt: V::Scalar,
    env: &ParticleEnvironment<V>,
    radius: V::Scalar,
    pos: &mut V,
    pos_old: &mut V,
) where 
    V: Vector
{
    let vel = *pos - *pos_old;

    let r = V::splat(radius);
    let min_collision_limit = env.space.bounds.min + r;
    let max_collision_limit = env.space.bounds.max - r;

    let under_min_mask = pos.cmplt(min_collision_limit);
    let over_max_mask = pos.cmpgt(max_collision_limit);
    let collision_mask = V::mask_or(under_min_mask, over_max_mask);

    if collision_mask.any() {
        // 1. Correct positions immediately to prevent wall-penetration
        let mut new_pos = *pos;
        new_pos = V::select(under_min_mask, min_collision_limit, new_pos);
        new_pos = V::select(over_max_mask, max_collision_limit, new_pos);

        // 2. Load the central frame-based jitter vector
        let base_noise = env.state.runtime_jitter;
        
        // 3. Separate clean raw bounce (normal) and standard sliding dampening (tangential)
        let clean_bounced_vel_normal = (-vel) * env.tuning.physics.restitution;
        let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);
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
        let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);
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

 

#[inline(always)]
pub fn resolve_particle_collisions<V>(
    env: &ParticleEnvironment<V>,
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
        
        if raw_penetration > env.tuning.physics.penetration_slop {
            let penetration = raw_penetration -  env.tuning.physics.penetration_slop;
            
            // Calculate a raw normal direction
            let mut normal = delta / dist;

            // --- UNIFIED HIGH-PERFORMANCE JITTER ---
            // Load the pre-calculated frame jitter directly into registers.
            let base_jitter =  env.state.runtime_jitter;
            
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
                let bias = env.tuning.physics.penetration_correction_bias;
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
        tuning: &PhysicsTuning<V::Scalar>,
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
    

   
}
 