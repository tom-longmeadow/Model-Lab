use crate::{math::{FloatScalar, Vector, VectorMask}, 
sim::solver::particle::{
     environment::ParticleEnvironment, space::{collision_registry::CollisionRegistry, grid::UniformGrid}, 
     verlet_particle::VerletParticle, verlet_soa_vec_storage::ComponentSliceMut}};
 

pub struct VerletPhysics; 
impl VerletPhysics { 
 

// #[inline(always)]
// pub fn resolve_particle_collisions<V>(
//     env: &ParticleEnvironment<V>,
//     pos_a: &mut V,
//     pos_b: &mut V,
//     radius_a: V::Scalar,
//     radius_b: V::Scalar,
//     inv_mass_a: V::Scalar,
//     inv_mass_b: V::Scalar,
// ) where 
//     V: Vector 
// {
//     let target_dist = radius_a + radius_b;
//     let target_dist_sq = target_dist * target_dist;

//     let mut delta = *pos_a - *pos_b;
//     let mut dist_sq = delta.length_squared();

//     // --- CATCH FUSED PARTICLES ---
//     // Zero heap allocations. Use your fast from_f64_array to inject a tiny separation offset.
//     if dist_sq == V::Scalar::ZERO {
//         let mut sep_arr = [0.0; 4];
//         sep_arr[0] = 0.0001; // Tiny displacement along the X-axis
//         delta = V::from_f64_array(sep_arr);
//         dist_sq = delta.length_squared();
//     }

//     if dist_sq < target_dist_sq {
//         let dist = dist_sq.sqrt();
//         let raw_penetration = target_dist - dist;
        
//         if raw_penetration > env.tuning.physics.penetration_slop {
//             let penetration = raw_penetration -  env.tuning.physics.penetration_slop;
            
//             // Calculate a raw normal direction
//             let mut normal = delta / dist;

//             // --- UNIFIED HIGH-PERFORMANCE JITTER ---
//             // Load the pre-calculated frame jitter directly into registers.
//             let base_jitter =  env.state.runtime_jitter;
            
//             // Mix the noise with the collision normal to randomize the perturbation direction.
//             // This ensures particles crashing from different angles get unique sideways shuffles,
//             // destroying vertical stacking grids with zero branch or trigonometry overhead.
//             let jitter_vec = normal.mul_elementwise(base_jitter);

//             // Perturb the normal and re-normalize to maintain vector unit length integrity
//             normal = normal + jitter_vec;
//             let normal_len_sq = normal.length_squared();
//             if normal_len_sq > V::Scalar::ZERO {
//                 normal = normal / normal_len_sq.sqrt();
//             }

//             // --- RESOLVE POSITIONS ---
//             let total_inv_mass = inv_mass_a + inv_mass_b;
//             if total_inv_mass > V::Scalar::ZERO {
//                 let bias = env.tuning.physics.penetration_correction_bias;
//                 let response_magnitude = (penetration * bias) / total_inv_mass;

//                 *pos_a += normal * (response_magnitude * inv_mass_a);
//                 *pos_b -= normal * (response_magnitude * inv_mass_b);
//             }
//         }
//     }
// }


    #[inline(always)]
    fn detect_and_resolve_collisions_skeleton<V: Vector>( 
        registry: &CollisionRegistry,
        env: &ParticleEnvironment<V>,
        mut resolve_pair: impl FnMut(usize, usize),
    ) {
        // 🟢 Loop safely drives the closure, letting internal math update dynamically
        for _ in 0..env.tuning.collision_iterations {
            for i in 0..registry.len() {
                let a = registry.a_indices[i];
                let b = registry.b_indices[i];
                resolve_pair(a, b);
            }
        }
    }
    
    #[inline]
    pub unsafe fn aos_detect_and_resolve_collisions<V: Vector>( 
        grid: &UniformGrid<V>,
        particles: &mut [VerletParticle<V>],
        registry: &mut CollisionRegistry,
        env: &ParticleEnvironment<V>,
    ) {
        registry.clear();
        grid.aos_find_collisions(particles, registry);

        let p_ptr = particles.as_mut_ptr();
        let slop = env.tuning.physics.penetration_slop;
        let bias = env.tuning.physics.penetration_correction_bias;
        let base_jitter = env.state.runtime_jitter;

        Self::detect_and_resolve_collisions_skeleton(registry, env, |a, b| {
            unsafe {
                let p_a = &mut *p_ptr.add(a);
                let p_b = &mut *p_ptr.add(b);

                // 🟢 FIXED: Normal points consistently from A to B due to strict index layout registry constraints
                let mut delta = p_a.pos - p_b.pos;
                let mut dist_sq = delta.length_squared();
                let target_dist = p_a.radius + p_b.radius;

                // 🟢 FIXED: Fused fallbacks perfectly preserved within active system loop
                if dist_sq == V::Scalar::ZERO {
                    let mut sep_arr = [0.0; 4];
                    sep_arr[0] = 0.0001;
                    delta = V::from_f64_array(sep_arr);
                    dist_sq = delta.length_squared();
                }

                if dist_sq < target_dist * target_dist {
                    let dist = dist_sq.sqrt();
                    let raw_penetration = target_dist - dist;

                    if raw_penetration > slop {
                        let penetration = raw_penetration - slop;
                        let mut normal = delta / dist;

                        let jitter_vec = normal.mul_elementwise(base_jitter);
                        normal = normal + jitter_vec;
                        let normal_len_sq = normal.length_squared();
                        if normal_len_sq > V::Scalar::ZERO {
                            normal = normal / normal_len_sq.sqrt();
                        }

                        // 🟢 FIXED: Rebuilt true individualized dynamic inverse mass physics properties
                        let inv_mass_a = p_a.inv_mass;
                        let inv_mass_b = p_b.inv_mass;
                        let total_inv_mass = inv_mass_a + inv_mass_b;

                        if total_inv_mass > V::Scalar::ZERO {
                            let response_magnitude = (penetration * bias) / total_inv_mass;

                            p_a.pos = p_a.pos + normal * (response_magnitude * inv_mass_a);
                            p_b.pos = p_b.pos - normal * (response_magnitude * inv_mass_b);
                        }
                    }
                }
            }
        });
    }

   #[inline]
    pub unsafe fn soa_detect_and_resolve_collisions<V: Vector>( 
        grid: &UniformGrid<V>,
        pos_x: &ComponentSliceMut<V::Scalar>,
        pos_y: &ComponentSliceMut<V::Scalar>,
        inv_masses: &[V::Scalar], 
        radii: &[V::Scalar],
        registry: &mut CollisionRegistry,
        env: &ParticleEnvironment<V>,
    ) {
        registry.clear();
        grid.soa_find_collisions(pos_x, pos_y, radii, registry);

        // 🟢 FIXED: Old as_mut_ptr() statements completely deleted!
        // We use pos_x and pos_y directly now.
        let m_ptr = inv_masses.as_ptr();
        let r_ptr = radii.as_ptr();

        let slop = env.tuning.physics.penetration_slop;
        let bias = env.tuning.physics.penetration_correction_bias;
        let jitter_x = env.state.runtime_jitter.component(0);
        let jitter_y = env.state.runtime_jitter.component(1);

        Self::detect_and_resolve_collisions_skeleton(registry, env, |a, b| {
            unsafe {
                // 🟢 FIXED: Using strided scalar element lookups natively
                let mut dx = pos_x.get_unchecked(a) - pos_x.get_unchecked(b);
                let mut dy = pos_y.get_unchecked(a) - pos_y.get_unchecked(b);
                let mut dist_sq = dx * dx + dy * dy;
                let target_dist = *r_ptr.add(a) + *r_ptr.add(b);

                if dist_sq == V::Scalar::ZERO {
                    dx = <V::Scalar as FloatScalar>::from_f64(0.0001);
                    dy = <V::Scalar as FloatScalar>::from_f64(0.0);
                    dist_sq = dx * dx;
                }

                if dist_sq < target_dist * target_dist {
                    let dist = dist_sq.sqrt();
                    let raw_penetration = target_dist - dist;

                    if raw_penetration > slop {
                        let penetration = raw_penetration - slop;
                        let inv_dist = V::Scalar::ONE / dist;
                        let mut nx = dx * inv_dist;
                        let mut ny = dy * inv_dist;

                        nx = nx + jitter_x;
                        ny = ny + jitter_y;
                        
                        let normal_len_sq = nx * nx + ny * ny;
                        if normal_len_sq > V::Scalar::ZERO {
                            let inv_normal_len = V::Scalar::ONE / normal_len_sq.sqrt();
                            nx = nx * inv_normal_len;
                            ny = ny * inv_normal_len;
                        }

                        let inv_mass_a = *m_ptr.add(a);
                        let inv_mass_b = *m_ptr.add(b);
                        let total_inv_mass = inv_mass_a + inv_mass_b;

                        if total_inv_mass > V::Scalar::ZERO {
                            let response_magnitude = (penetration * bias) / total_inv_mass;
                            
                            let shift_xa = nx * response_magnitude * inv_mass_a;
                            let shift_ya = ny * response_magnitude * inv_mass_a;
                            let shift_xb = nx * response_magnitude * inv_mass_b;
                            let shift_yb = ny * response_magnitude * inv_mass_b;

                            // 🟢 FIXED: Replaced *x_ptr.add(a) writes with clean strided setters
                            pos_x.set_unchecked(a, pos_x.get_unchecked(a) + shift_xa);
                            pos_y.set_unchecked(a, pos_y.get_unchecked(a) + shift_ya);
                            pos_x.set_unchecked(b, pos_x.get_unchecked(b) - shift_xb);
                            pos_y.set_unchecked(b, pos_y.get_unchecked(b) - shift_yb);
                        }
                    }
                }
            }
        });
    }

     
    //  #[inline(always)]
    // pub fn update_kinetics<V>(
    //     dt: V::Scalar,
    //     env: &ParticleEnvironment<V>,
    //     pos: &mut V,
    //     pos_old: &mut V, 
    //     acc: &mut V,
    // ) where 
    //     V: Vector
    // {
    //     let temp_pos = *pos;
        
    //     // 1. Calculate the implicit velocity vector (displacement)
    //     let displacement = temp_pos - *pos_old;

    //     // 2. Calculate frame-rate independent damping factor 
    //     let damping_val = -env.tuning.physics.global_damping * dt;
    //     let damping_factor = damping_val.exp();

    //     // 3. Apply Verlet integration with correct Vector * Scalar ordering
    //     let dt_sq = dt * dt;
        
    //     // Enforce (Vector * Scalar) layout to match your trait boundaries
    //     let damped_displacement = displacement * damping_factor;
    //     let accelerated_displacement = *acc * dt_sq;
        
    //     let next_pos = temp_pos + damped_displacement + accelerated_displacement;

    //     *pos = next_pos;
    //     *pos_old = temp_pos;
    //     *acc = V::ZERO;
    // }

     #[inline] 
    pub fn soa_update_kinetics<V: Vector>(
        pos_x: &ComponentSliceMut<V::Scalar>,
        pos_y: &ComponentSliceMut<V::Scalar>,
        old_x: &ComponentSliceMut<V::Scalar>,
        old_y: &ComponentSliceMut<V::Scalar>,
        acc_x: &ComponentSliceMut<V::Scalar>,
        acc_y: &ComponentSliceMut<V::Scalar>,
        dt: V::Scalar,
        env: &ParticleEnvironment<V>,
    ) {
        let len = pos_x.len();
        if len == 0 { return; }

        let damping_val = -env.tuning.physics.global_damping * dt;
        let damping_factor = damping_val.exp(); 
        let dt_sq = dt * dt;

        // This loop body remains 100% clean, raw, and unblocked for LLVM auto-vectorization
        for i in 0..len {
            unsafe {
                // Read from our strided component slices
                let temp_x = pos_x.get_unchecked(i);
                let temp_y = pos_y.get_unchecked(i);

                let disp_x = temp_x - old_x.get_unchecked(i);
                let disp_y = temp_y - old_y.get_unchecked(i);

                // Compute updated kinematics and write back to memory lanes
                pos_x.set_unchecked(i, temp_x + (disp_x * damping_factor) + (acc_x.get_unchecked(i) * dt_sq));
                pos_y.set_unchecked(i, temp_y + (disp_y * damping_factor) + (acc_y.get_unchecked(i) * dt_sq));

                // Save historical state references
                old_x.set_unchecked(i, temp_x);
                old_y.set_unchecked(i, temp_y);

                // Zero out the registers for the next frame
                acc_x.set_unchecked(i, V::Scalar::ZERO);
                acc_y.set_unchecked(i, V::Scalar::ZERO);
            }
        }
    }

    // ============================================================================
    // ARRAY OF STRUCTURES (AoS) KINETICS
    // ============================================================================
    #[inline]
    pub fn aos_update_kinetics<V: Vector>( 
        particles: &mut [VerletParticle<V>],
        dt: V::Scalar,
        env: &ParticleEnvironment<V>,
    ) {
        if particles.is_empty() { return; }

        // 🟢 FIXED: Restored frame-rate independent exponential damping calculation outside the loop
        let damping_val = -env.tuning.physics.global_damping * dt;
        let damping_factor = damping_val.exp();
        let dt_sq = dt * dt;

        // This loop body remains optimized for sequential pre-fetching of packed structures
        for p in particles.iter_mut() {
            let temp_pos = p.pos;

            let displacement = temp_pos - p.pos_old;

            // 🟢 FIXED: Using correct exponential multiplier factor
            let damped_displacement = displacement * damping_factor;
            let accelerated_displacement = p.acc * dt_sq;

            p.pos = temp_pos + damped_displacement + accelerated_displacement;
            p.pos_old = temp_pos;
            p.acc = V::ZERO;
        }
    }

 

//      #[inline(always)] 
// pub fn apply_position_constraints<V>(
//     dt: V::Scalar,
//     env: &ParticleEnvironment<V>,
//     radius: V::Scalar,
//     pos: &mut V,
//     pos_old: &mut V,
// ) where 
//     V: Vector
// {
//     let vel = *pos - *pos_old;

//     let r = V::splat(radius);
//     let min_collision_limit = env.space.bounds.min + r;
//     let max_collision_limit = env.space.bounds.max - r;

//     let under_min_mask = pos.cmplt(min_collision_limit);
//     let over_max_mask = pos.cmpgt(max_collision_limit);
//     let collision_mask = V::mask_or(under_min_mask, over_max_mask);

//     if collision_mask.any() {
//         // 1. Correct positions immediately to prevent wall-penetration
//         let mut new_pos = *pos;
//         new_pos = V::select(under_min_mask, min_collision_limit, new_pos);
//         new_pos = V::select(over_max_mask, max_collision_limit, new_pos);

//         // 2. Load the central frame-based jitter vector
//         let base_noise = env.state.runtime_jitter;
        
//         // 3. Separate clean raw bounce (normal) and standard sliding dampening (tangential)
//         let clean_bounced_vel_normal = (-vel) * env.tuning.physics.restitution;
//         let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);
//         let slowed_vel_tangential = vel * friction_diminish;

//         // 4. ACTIVE SLIDE JITTER INJECTION
//         // Introduce a constant shuffling force along the wall components.
//         // This keeps particles fluidly moving sideways out of rigid stacks, even at rest.
//         let jittered_tangential_vel = slowed_vel_tangential + base_noise * dt;

//         // 5. Select axis routes based on SIMD collision state
//         // - Colliding axes get the clean, un-jittered perpendicular reflection normal.
//         // - Non-colliding axes get the jittered tangential/sliding velocity.
//         let new_vel = V::select(collision_mask, clean_bounced_vel_normal, jittered_tangential_vel);

//         *pos = new_pos;
//         *pos_old = new_pos - new_vel;
//     } else {
//         // OPEN AIR PATH: Zero noise calculations, completely pure execution branch
//         let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);
//         let clean_slowed_vel_tangential = vel * friction_diminish;
        
//         *pos_old = *pos - clean_slowed_vel_tangential;
//     }
// }

    /// Applies boundary constraints, wall friction, and sliding jitter to an array of AoS structures.
    #[inline]
    pub fn aos_apply_position_constraints<V: Vector>( 
        particles: &mut [VerletParticle<V>],
        dt: V::Scalar,
        env: &ParticleEnvironment<V>,
    ) {
        if particles.is_empty() { return; }

        let bounds_min = env.space.bounds.min;
        let bounds_max = env.space.bounds.max;
        let restitution = env.tuning.physics.restitution;
        let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);
        let base_noise = env.state.runtime_jitter;

        for p in particles.iter_mut() {
            let vel = p.pos - p.pos_old;
            let r = V::splat(p.radius);
            
            let min_limit = bounds_min + r;
            let max_limit = bounds_max - r;

            let under_min_mask = p.pos.cmplt(min_limit);
            let over_max_mask = p.pos.cmpgt(max_limit);
            let collision_mask = V::mask_or(under_min_mask, over_max_mask);

            if collision_mask.any() {
                let mut new_pos = p.pos;
                new_pos = V::select(under_min_mask, min_limit, new_pos);
                new_pos = V::select(over_max_mask, max_limit, new_pos);

                let clean_bounced_vel_normal = (-vel) * restitution;
                let jittered_tangential_vel = (vel * friction_diminish) + (base_noise * dt);

                let new_vel = V::select(collision_mask, clean_bounced_vel_normal, jittered_tangential_vel);

                p.pos = new_pos;
                p.pos_old = new_pos - new_vel;
            } else {
                // Open air path: Apply standard air resistance/friction
                let clean_slowed_vel_tangential = vel * friction_diminish;
                p.pos_old = p.pos - clean_slowed_vel_tangential;
            }
        }
    }

   #[inline]
    pub fn soa_apply_position_constraints<V: Vector>( 
        pos_x: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        pos_y: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        old_x: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        old_y: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        radii: &[V::Scalar],
        dt: V::Scalar,
        env: &ParticleEnvironment<V>,
    ) {
        let len = pos_x.len();
        if len == 0 { return; }

        // Extract scalar boundaries using local trait component getters
        let min_x = env.space.bounds.min.component(0);
        let min_y = env.space.bounds.min.component(1);
        let max_x = env.space.bounds.max.component(0);
        let max_y = env.space.bounds.max.component(1);

        let restitution = env.tuning.physics.restitution;
        let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);

        // Pre-scale sliding jitter components natively into registers
        let noise_dt_x = env.state.runtime_jitter.component(0) * dt;
        let noise_dt_y = env.state.runtime_jitter.component(1) * dt;

        // 🟢 FIXED: Old slice len asserts removed since custom strides carry native lengths
        let r_ptr = radii.as_ptr();

        for i in 0..len {
            unsafe {
                let r = *r_ptr.add(i);
                let px = pos_x.get_unchecked(i);
                let py = pos_y.get_unchecked(i);

                let min_limit_x = min_x + r;
                let max_limit_x = max_x - r;
                let min_limit_y = min_y + r;
                let max_limit_y = max_y - r;

                // 1. Establish strict boolean flags for axis collision states
                let hit_x = px < min_limit_x || px > max_limit_x;
                let hit_y = py < min_limit_y || py > max_limit_y;

                let vel_x = px - old_x.get_unchecked(i);
                let vel_y = py - old_y.get_unchecked(i);

                // 2. Branch matching original logic (Collision vs Open Air Path)
                if hit_x || hit_y {
                    // --- RESOLVE X-AXIS ---
                    if px < min_limit_x {
                        pos_x.set_unchecked(i, min_limit_x);
                        old_x.set_unchecked(i, min_limit_x - (-vel_x * restitution));
                    } else if px > max_limit_x {
                        pos_x.set_unchecked(i, max_limit_x);
                        old_x.set_unchecked(i, max_limit_x - (-vel_x * restitution));
                    } else {
                        // Particle is touching a Y-wall but moving freely on X -> Apply sliding jitter!
                        let jittered_vel_x = (vel_x * friction_diminish) + noise_dt_x;
                        old_x.set_unchecked(i, px - jittered_vel_x);
                    }

                    // --- RESOLVE Y-AXIS ---
                    if py < min_limit_y {
                        pos_y.set_unchecked(i, min_limit_y);
                        old_y.set_unchecked(i, min_limit_y - (-vel_y * restitution));
                    } else if py > max_limit_y {
                        pos_y.set_unchecked(i, max_limit_y);
                        old_y.set_unchecked(i, max_limit_y - (-vel_y * restitution));
                    } else {
                        // Particle is touching an X-wall but moving freely on Y -> Apply sliding jitter!
                        let jittered_vel_y = (vel_y * friction_diminish) + noise_dt_y;
                        old_y.set_unchecked(i, py - jittered_vel_y);
                    }
                } else {
                    // PURE OPEN AIR PATH - Zero noise injection, perfectly smooth ballistic movement
                    old_x.set_unchecked(i, px - (vel_x * friction_diminish));
                    old_y.set_unchecked(i, py - (vel_y * friction_diminish));
                }
            }
        }
    }
  
    // /// 2. Restitution-only history modification (Runs ONCE after the loops close)
    // #[inline(always)]
    // pub fn apply_particle_restitution<V>(
    //     tuning: &PhysicsTuning<V::Scalar>,
    //     pos_a: &V,
    //     pos_b: &V,
    //     pos_old_a: &mut V,
    //     pos_old_b: &mut V,
    //     radius_a: V::Scalar,
    //     radius_b: V::Scalar,
    //     inv_mass_a: V::Scalar,
    //     inv_mass_b: V::Scalar,
    // ) where 
    //     V: Vector 
    // {
    //     let delta = *pos_a - *pos_b;
    //     let dist_sq = delta.length_squared();
        
    //     // Add a slight extra margin to catch particles that were just resolved and are touching
    //     let target_dist = radius_a + radius_b + tuning.penetration_slop;
    //     let target_dist_sq = target_dist * target_dist;

    //     if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
    //         let dist = dist_sq.sqrt();
    //         let normal = delta / dist;

    //         let total_inv_mass = inv_mass_a + inv_mass_b;
    //         if total_inv_mass > V::Scalar::ZERO {
    //             // Read velocities after all position corrections have completed
    //             let vel_a = *pos_a - *pos_old_a;
    //             let vel_b = *pos_b - *pos_old_b;
    //             let relative_vel = vel_a - vel_b;

    //             let normal_vel_mag = relative_vel.dot(normal);

    //             // Only bounce if they are traveling towards each other
    //             if normal_vel_mag < V::Scalar::ZERO {
    //                 let target_normal_vel = -normal_vel_mag * tuning.restitution;
    //                 let delta_vel_mag = target_normal_vel - normal_vel_mag;

    //                 let vel_impulse_mag = delta_vel_mag / total_inv_mass;
    //                 let vel_change_vector = normal * vel_impulse_mag;

    //                 // Modify history registers cleanly exactly once
    //                 *pos_old_a -= vel_change_vector * inv_mass_a;
    //                 *pos_old_b += vel_change_vector * inv_mass_b;
    //             }
    //         }
    //     }
    // }
    

    #[inline]
    pub unsafe fn aos_apply_particle_restitution<V: Vector>( 
        registry: &CollisionRegistry,  
        particles: &mut [VerletParticle<V>],
        env: &ParticleEnvironment<V>,
    ) {
        if registry.is_empty() { return; }

        let p_ptr = particles.as_mut_ptr();
        let slop = env.tuning.physics.penetration_slop;
        let restitution = env.tuning.physics.restitution;

        for i in 0..registry.len() {
            let a = registry.a_indices[i];
            let b = registry.b_indices[i];
            
            unsafe {
                let p_a = &mut *p_ptr.add(a);
                let p_b = &mut *p_ptr.add(b);

                let delta = p_a.pos - p_b.pos;
                let dist_sq = delta.length_squared();
                
                let target_dist = p_a.radius + p_b.radius + slop;
                let target_dist_sq = target_dist * target_dist;

                if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
                    let dist = dist_sq.sqrt();
                    let normal = delta / dist;

                    // 🟢 FIXED: Restored true dynamic inverse mass evaluations
                    let inv_mass_a = p_a.inv_mass;
                    let inv_mass_b = p_b.inv_mass;
                    let total_inv_mass = inv_mass_a + inv_mass_b;

                    if total_inv_mass > V::Scalar::ZERO {
                        let vel_a = p_a.pos - p_a.pos_old;
                        let vel_b = p_b.pos - p_b.pos_old;
                        let relative_vel = vel_a - vel_b;

                        let normal_vel_mag = relative_vel.dot(normal);

                        if normal_vel_mag < V::Scalar::ZERO {
                            let target_normal_vel = -normal_vel_mag * restitution;
                            let delta_vel_mag = target_normal_vel - normal_vel_mag;

                            let vel_impulse_mag = delta_vel_mag / total_inv_mass;
                            let vel_change_vector = normal * vel_impulse_mag;

                            // Apply mass-weighted corrections to history positions
                            p_a.pos_old -= vel_change_vector * inv_mass_a;
                            p_b.pos_old += vel_change_vector * inv_mass_b;
                        }
                    }
                }
            }
        }
    }

    // ============================================================================
    // STRUCTURE OF ARRAYS (SoA) RESTITUTION
    // ============================================================================
    #[inline]
    pub unsafe fn soa_apply_particle_restitution<V: Vector>( 
        registry: &CollisionRegistry,  
        pos_x: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        pos_y: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        old_x: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        old_y: &ComponentSliceMut<V::Scalar>,   // 🟢 FIXED: Remapped to strided pointer layout
        inv_masses: &[V::Scalar],     
        radii: &[V::Scalar],
        env: &ParticleEnvironment<V>,
    ) {
        if registry.is_empty() { return; }

        // 🟢 FIXED: Extraneous x_ptr, y_ptr, ox_ptr, and oy_ptr initializations deleted!
        let m_ptr = inv_masses.as_ptr();
        let r_ptr = radii.as_ptr();

        let slop = env.tuning.physics.penetration_slop;
        let restitution = env.tuning.physics.restitution;

        for i in 0..registry.len() {
            let a = registry.a_indices[i];
            let b = registry.b_indices[i];

            unsafe {
                // 🟢 FIXED: Replaced raw pointer offsets with clean strided getters
                let dx = pos_x.get_unchecked(a) - pos_x.get_unchecked(b);
                let dy = pos_y.get_unchecked(a) - pos_y.get_unchecked(b);
                let dist_sq = dx * dx + dy * dy;

                let target_dist = *r_ptr.add(a) + *r_ptr.add(b) + slop;
                let target_dist_sq = target_dist * target_dist;

                if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
                    let dist = dist_sq.sqrt();
                    let inv_dist = V::Scalar::ONE / dist;
                    let nx = dx * inv_dist;
                    let ny = dy * inv_dist;

                    let inv_mass_a = *m_ptr.add(a);
                    let inv_mass_b = *m_ptr.add(b);
                    let total_inv_mass = inv_mass_a + inv_mass_b;

                    if total_inv_mass > V::Scalar::ZERO {
                        // 🟢 FIXED: Velocity calculations updated to use strided getters
                        let vel_ax = pos_x.get_unchecked(a) - old_x.get_unchecked(a);
                        let vel_ay = pos_y.get_unchecked(a) - old_y.get_unchecked(a);
                        let vel_bx = pos_x.get_unchecked(b) - old_x.get_unchecked(b);
                        let vel_by = pos_y.get_unchecked(b) - old_y.get_unchecked(b);

                        let rel_vel_x = vel_ax - vel_bx;
                        let rel_vel_y = vel_ay - vel_by;

                        let normal_vel_mag = rel_vel_x * nx + rel_vel_y * ny;

                        if normal_vel_mag < V::Scalar::ZERO {
                            let target_normal_vel = -normal_vel_mag * restitution;
                            let delta_vel_mag = target_normal_vel - normal_vel_mag;

                            let vel_impulse_mag = delta_vel_mag / total_inv_mass;
                            let change_x = nx * vel_impulse_mag;
                            let change_y = ny * vel_impulse_mag;

                            // 🟢 FIXED: History register writebacks updated to use strided setters
                            old_x.set_unchecked(a, old_x.get_unchecked(a) - (change_x * inv_mass_a));
                            old_y.set_unchecked(a, old_y.get_unchecked(a) - (change_y * inv_mass_a));
                            old_x.set_unchecked(b, old_x.get_unchecked(b) + (change_x * inv_mass_b));
                            old_y.set_unchecked(b, old_y.get_unchecked(b) + (change_y * inv_mass_b));
                        }
                    }
                }
            }
        }
    }
   
}



// pub struct VerletCoreEngine;
// impl VerletCoreEngine {
//     pub fn execute_sub_step<V, L>(
//         layout: &mut L,
//         dt: f64,
//         environment: &ParticleEnvironment<V>,
//         scratch_radii: &[V::Scalar],
//     ) where
//         V: Vector,
//         L: ParticleDataLayout<V>,
//     {
//         let len = layout.len();
//         if len == 0 { return; }

//         let sub_step_dt = <V::Scalar as FloatScalar>::from_f64(dt);
//         let mut collisions = CollisionRegistry::new();

//         // 1. Broadphase: Operates directly on the provided data layouts
//         VerletPhysics.detect_collisions(len, scratch_radii, layout.positions_mut(), &mut collisions);

//         // 2. Iterative Position Constraint Relaxation
//         let inv_mass = <V::Scalar as FloatScalar>::from_f64(1.0);
//         let pos_slice = layout.positions_mut();
        
//         for _ in 0..environment.tuning.collision_iterations {
//             for collision in &collisions.pairs {
//                 let a = collision.a_index;
//                 let b = collision.b_index;
//                 if a == b { continue; }

//                 // Safe Simultaneous Index Split
//                 if a < b {
//                     let (left, right) = pos_slice.split_at_mut(b);
//                     VerletPhysics::resolve_particle_collisions(
//                         environment, &mut left[a], &mut right[0], 
//                         scratch_radii[a], scratch_radii[b], inv_mass, inv_mass
//                     );
//                 } else {
//                     let (left, right) = pos_slice.split_at_mut(a);
//                     VerletPhysics::resolve_particle_collisions(
//                         environment, &mut right[0], &mut left[b], 
//                         scratch_radii[a], scratch_radii[b], inv_mass, inv_mass
//                     );
//                 }
//             } 
//         }

//         let (pos_slice, old_slice) = layout.positions_and_old_mut();
//         for collision in &collisions.pairs {
//             let a = collision.a_index;
//             let b = collision.b_index;
//             if a == b { continue; }

//             let (pos_a, pos_b) = if a < b {
//                 let (left, right) = pos_slice.split_at_mut(b); (&left[a], &right[0])
//             } else {
//                 let (left, right) = pos_slice.split_at_mut(a); (&right[0], &left[b])
//             };

//             let (old_a, old_b) = if a < b {
//                 let (left, right) = old_slice.split_at_mut(b); (&mut left[a], &mut right[0])
//             } else {
//                 let (left, right) = old_slice.split_at_mut(a); (&mut right[0], &mut left[b])
//             };

//             VerletPhysics::apply_particle_restitution(
//                 &environment.tuning.physics, pos_a, pos_b, old_a, old_b,
//                 scratch_radii[a], scratch_radii[b], inv_mass, inv_mass,
//             );
//         }

//         // 4. Global Environmental Constraints
//         let (pos_slice, old_slice) = layout.positions_and_old_mut();
//         for i in 0..len {
//             VerletPhysics::apply_position_constraints(
//                 sub_step_dt, environment, scratch_radii[i], &mut pos_slice[i], &mut old_slice[i]
//             );
//         }

//         // 5. Final Clamping & Writeback
//         let sub_step_max_vel = environment.tuning.physics.max_velocity * sub_step_dt;
//         let max_vel_squared = sub_step_max_vel * sub_step_max_vel;
//         layout.commit_kinetics(max_vel_squared, sub_step_max_vel);
//     }
// }



 