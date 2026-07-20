// use crate::{math::{FloatScalar, Vector, VectorMask}, 
// sim::solver::particle::{
//      environment::ParticleEnvironment, space::{collision_registry::CollisionRegistry}, 
//      verlet_particle::VerletParticle}};
 

// pub struct VerletPhysics; 
// impl VerletPhysics { 
 
 
//     #[inline(always)]
//     fn resolve_collisions_skeleton<V: Vector>( 
//         registry: &CollisionRegistry,
//         env: &ParticleEnvironment<V>,
//         mut resolve_pair: impl FnMut(usize, usize),
//     ) {
//         // 🟢 Loop safely drives the closure, letting internal math update dynamically
//         for _ in 0..env.tuning.collision_iterations {
//             for i in 0..registry.len() {
//                 let a = registry.a_indices[i];
//                 let b = registry.b_indices[i];
//                 resolve_pair(a, b);
//             }
//         }
//     }

//      #[inline]
//     pub unsafe fn soa_resolve_collisions<V: Vector>(
//         positions: &mut [V],        // 🟢 FIXED: Contiguous vector slice straight from storage layout
//         inv_masses: &[V::Scalar],   // 🟢 FIXED: Safe standard slice layouts
//         radii: &[V::Scalar],
//         registry: &CollisionRegistry, 
//         env: &ParticleEnvironment<V>,
//     ) {
//         let p_ptr = positions.as_mut_ptr();
//         let m_ptr = inv_masses.as_ptr();
//         let r_ptr = radii.as_ptr();

//         let slop = env.tuning.physics.penetration_slop;
//         let bias = env.tuning.physics.penetration_correction_bias;
//         let base_jitter = env.state.runtime_jitter;

//         Self::resolve_collisions_skeleton(registry, env, |a, b| {
//             unsafe {
//                 // Safely fetch pointers to the contiguous Vector values
//                 let pos_a_ptr = p_ptr.add(a);
//                 let pos_b_ptr = p_ptr.add(b);

//                 // Use the Vector trait operators directly (Consistent A to B normal direction)
//                 let mut delta = *pos_a_ptr - *pos_b_ptr;
//                 let mut dist_sq = delta.length_squared();
//                 let target_dist = *r_ptr.add(a) + *r_ptr.add(b);

//                 // Zero-allocation fallback path using native vector slice construction
//                 if dist_sq == V::Scalar::ZERO {
//                     let sep_arr = [<V::Scalar as FloatScalar>::from_f64(0.0001), V::Scalar::ZERO];
//                     delta = V::from_slice(&sep_arr);
//                     dist_sq = delta.length_squared();
//                 }

//                 if dist_sq < target_dist * target_dist {
//                     let dist = dist_sq.sqrt();
//                     let raw_penetration = target_dist - dist;

//                     if raw_penetration > slop {
//                         let penetration = raw_penetration - slop;
//                         let mut normal = delta / dist;

//                         // Unified element-wise jitter math
//                         let jitter_vec = normal.mul_elementwise(base_jitter);
//                         normal = normal + jitter_vec;
//                         let normal_len_sq = normal.length_squared();
//                         if normal_len_sq > V::Scalar::ZERO {
//                             normal = normal / normal_len_sq.sqrt();
//                         }

//                         let inv_mass_a = *m_ptr.add(a);
//                         let inv_mass_b = *m_ptr.add(b);
//                         let total_inv_mass = inv_mass_a + inv_mass_b;

//                         if total_inv_mass > V::Scalar::ZERO {
//                             let response_magnitude = (penetration * bias) / total_inv_mass;
                            
//                             let displacement_a = normal * (response_magnitude * inv_mass_a);
//                             let displacement_b = normal * (response_magnitude * inv_mass_b);

//                             // Write back unified vector properties with zero stride math overhead
//                             *pos_a_ptr = *pos_a_ptr + displacement_a;
//                             *pos_b_ptr = *pos_b_ptr - displacement_b;
//                         }
//                     }
//                 }
//             }
//         });
//     }
    
//      #[inline]
//     pub fn aos_resolve_collisions<V: Vector>( // 🟢 Renamed to reflect its exact job
//         particles: &mut [VerletParticle<V>],
//         registry: &CollisionRegistry, // 🟢 READ-ONLY: No longer clears or overwrites broadphase data
//         env: &ParticleEnvironment<V>,
//     ) {
//         // 🟢 FIXED: registry.clear() and grid.aos_find_collisions() completely deleted!

//         let p_ptr = particles.as_mut_ptr();
//         let slop = env.tuning.physics.penetration_slop;
//         let bias = env.tuning.physics.penetration_correction_bias;
//         let base_jitter = env.state.runtime_jitter;

//         Self::resolve_collisions_skeleton(registry, env, |a, b| {
//             unsafe {
//                 let p_a = &mut *p_ptr.add(a);
//                 let p_b = &mut *p_ptr.add(b);

//                 // 🟢 FIXED: Normal points consistently from A to B due to strict index layout registry constraints
//                 let mut delta = p_a.pos - p_b.pos;
//                 let mut dist_sq = delta.length_squared();
//                 let target_dist = p_a.radius + p_b.radius;

//                 // 🟢 FIXED: Fused fallbacks use native slices instead of f64 stack array conversions
//                 if dist_sq == V::Scalar::ZERO {
//                     let sep_arr = [<V::Scalar as FloatScalar>::from_f64(0.0001), V::Scalar::ZERO];
//                     delta = V::from_slice(&sep_arr);
//                     dist_sq = delta.length_squared();
//                 }

//                 if dist_sq < target_dist * target_dist {
//                     let dist = dist_sq.sqrt();
//                     let raw_penetration = target_dist - dist;

//                     if raw_penetration > slop {
//                         let penetration = raw_penetration - slop;
//                         let mut normal = delta / dist;

//                         let jitter_vec = normal.mul_elementwise(base_jitter);
//                         normal = normal + jitter_vec;
//                         let normal_len_sq = normal.length_squared();
//                         if normal_len_sq > V::Scalar::ZERO {
//                             normal = normal / normal_len_sq.sqrt();
//                         }

//                         // 🟢 FIXED: Rebuilt true individualized dynamic inverse mass physics properties
//                         let inv_mass_a = p_a.inv_mass;
//                         let inv_mass_b = p_b.inv_mass;
//                         let total_inv_mass = inv_mass_a + inv_mass_b;

//                         if total_inv_mass > V::Scalar::ZERO {
//                             let response_magnitude = (penetration * bias) / total_inv_mass;

//                             p_a.pos = p_a.pos + normal * (response_magnitude * inv_mass_a);
//                             p_b.pos = p_b.pos - normal * (response_magnitude * inv_mass_b);
//                         }
//                     }
//                 }
//             }
//         });
//     }

  

    

//     #[inline] 
// pub fn soa_update_kinetics<V: Vector>(
//     positions: &mut [V],
//     positions_old: &mut [V],
//     accelerations: &mut [V],
//     dt: V::Scalar,       // Current sub-step dt 
//     env: &ParticleEnvironment<V>,
// ) {
//     let len = positions.len();
//     if len == 0 { return; }

//     // Calculate your continuous per-second exponential damping factor
//     let damping_factor = (-env.tuning.physics.global_damping * dt).exp(); 
//     let dt_sq = dt * dt;

//     for i in 0..len {
//         unsafe {
//             let current_pos = *positions.get_unchecked(i);
//             let old_pos = *positions_old.get_unchecked(i);
//             let acc = *accelerations.get_unchecked(i);

//             // 🟢 FIXED: Mathematically simplified since dt == prev_dt at fixed Hz
//             let displacement = current_pos - old_pos;
//             let next_pos = current_pos + (displacement * damping_factor) + (acc * dt_sq);

//             *positions.get_unchecked_mut(i) = next_pos;
//             *positions_old.get_unchecked_mut(i) = current_pos;
//             *accelerations.get_unchecked_mut(i) = V::ZERO;
//         }
//     }
// }

// // ============================================================================
// // ARRAY OF STRUCTURES (AoS) KINETICS
// // ============================================================================
// #[inline]
// pub fn aos_update_kinetics<V: Vector>( 
//     particles: &mut [VerletParticle<V>],
//     dt: V::Scalar,       // Current sub-step dt 
//     env: &ParticleEnvironment<V>,
// ) {
//     if particles.is_empty() { return; }

//     // Calculate your continuous per-second exponential damping factor
//     let damping_factor = (-env.tuning.physics.global_damping * dt).exp();
//     let dt_sq = dt * dt;

//     for p in particles.iter_mut() {
//         let temp_pos = p.pos;

//         // 🟢 FIXED: Mathematically simplified since dt == prev_dt at fixed Hz
//         let displacement = temp_pos - p.pos_old;
        
//         p.pos = temp_pos + (displacement * damping_factor) + (p.acc * dt_sq);
//         p.pos_old = temp_pos;
//         p.acc = V::ZERO;
//     }
// }
  

//     /// Applies boundary constraints, wall friction, and sliding jitter to an array of AoS structures.
//     #[inline]
//     pub fn aos_apply_position_constraints<V: Vector>( 
//         particles: &mut [VerletParticle<V>],
//         dt: V::Scalar,
//         env: &ParticleEnvironment<V>,
//     ) {
//         if particles.is_empty() { return; }

//         let bounds_min = env.space.bounds.min;
//         let bounds_max = env.space.bounds.max;
//         let restitution = env.tuning.physics.restitution;
//         let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);
//         let base_noise = env.state.runtime_jitter;

//         for p in particles.iter_mut() {
//             let vel = p.pos - p.pos_old;
//             let r = V::splat(p.radius);
            
//             let min_limit = bounds_min + r;
//             let max_limit = bounds_max - r;

//             let under_min_mask = p.pos.cmplt(min_limit);
//             let over_max_mask = p.pos.cmpgt(max_limit);
//             let collision_mask = V::mask_or(under_min_mask, over_max_mask);

//             if collision_mask.any() {
//                 let mut new_pos = p.pos;
//                 new_pos = V::select(under_min_mask, min_limit, new_pos);
//                 new_pos = V::select(over_max_mask, max_limit, new_pos);

//                 let clean_bounced_vel_normal = (-vel) * restitution;
//                 let jittered_tangential_vel = (vel * friction_diminish) + (base_noise * dt);

//                 let new_vel = V::select(collision_mask, clean_bounced_vel_normal, jittered_tangential_vel);

//                 p.pos = new_pos;
//                 p.pos_old = new_pos - new_vel;
//             } else {
//                 // Open air path: Apply standard air resistance/friction
//                 let clean_slowed_vel_tangential = vel * friction_diminish;
//                 p.pos_old = p.pos - clean_slowed_vel_tangential;
//             }
//         }
//     }

//     #[inline]
//     pub fn soa_apply_position_constraints<V: Vector>( 
//         positions: &mut [V],        // 🟢 FIXED: Converted to unified vector slice layout
//         positions_old: &mut [V],    // 🟢 FIXED: Converted to unified vector slice layout
//         radii: &[V::Scalar],
//         dt: V::Scalar,
//         env: &ParticleEnvironment<V>,
//     ) {
//         let len = positions.len();
//         if len == 0 { return; }

//         let min_bound = env.space.bounds.min;
//         let max_bound = env.space.bounds.max;

//         let restitution = env.tuning.physics.restitution;
//         let friction_diminish = V::Scalar::ONE - (dt * env.tuning.physics.friction);
//         let noise_dt = env.state.runtime_jitter * dt;

//         let p_ptr = positions.as_mut_ptr();
//         let o_ptr = positions_old.as_mut_ptr();
//         let r_ptr = radii.as_ptr();

//         for i in 0..len {
//             unsafe {
//                 let r = *r_ptr.add(i);
//                 let pos_ptr = p_ptr.add(i);
//                 let old_ptr = o_ptr.add(i);

//                 let p = *pos_ptr;
//                 let o = *old_ptr;

//                 // Build radius bounds vectors natively using splat
//                 let r_v = V::splat(r);
//                 let min_limit = min_bound + r_v;
//                 let max_limit = max_bound - r_v;

//                 let vel = p - o;

//                 // 1. Establish strict collision states across all axes via vector masks
//                 let hit_min_mask = p.cmplt(min_limit);
//                 let hit_max_mask = p.cmpgt(max_limit);
                
//                 // Track if any wall boundary constraint was triggered
//                 if hit_min_mask.any() || hit_max_mask.any() {
//                     // Cache the layout configuration properties as local arrays for fast component lookup
//                     let mut p_arr = [V::Scalar::ZERO; 4];
//                     let mut o_arr = [V::Scalar::ZERO; 4];
                    
//                     p_arr[0] = p.component(0);
//                     p_arr[1] = p.component(1);
//                     o_arr[0] = o.component(0);
//                     o_arr[1] = o.component(1);

//                     for axis in 0..2 {
//                         let pa = p_arr[axis];
//                         let oa = o_arr[axis];
//                         let va = pa - oa;
//                         let min_l = min_limit.component(axis);
//                         let max_l = max_limit.component(axis);

//                         if pa < min_l {
//                             p_arr[axis] = min_l;
//                             o_arr[axis] = min_l - (-va * restitution);
//                         } else if pa > max_l {
//                             p_arr[axis] = max_l;
//                             o_arr[axis] = max_l - (-va * restitution);
//                         } else {
//                             // Touching the opposite axis boundary -> Apply friction reduction and sliding noise
//                             let jitter_vel = (va * friction_diminish) + noise_dt.component(axis);
//                             o_arr[axis] = pa - jitter_vel;
//                         }
//                     }

//                     // Reconstruct from native scalar slices back to active vector structures
//                     *pos_ptr = V::from_slice(&p_arr);
//                     let composite_mask = V::mask_or(hit_min_mask, hit_max_mask);
//                     let fallback_val = p - (vel * friction_diminish);

//                     *old_ptr = V::select(
//                         composite_mask, 
//                         V::from_slice(&o_arr), 
//                         fallback_val
// );
//                 } else {
//                     // PURE OPEN AIR PATH - Zero noise injection, perfectly smooth ballistic movement
//                     *old_ptr = p - (vel * friction_diminish);
//                 }
//             }
//         }
//     }
   

//     #[inline]
//     pub fn aos_apply_particle_restitution<V: Vector>( 
//         registry: &CollisionRegistry,  
//         particles: &mut [VerletParticle<V>],
//         env: &ParticleEnvironment<V>,
//     ) {
//         if registry.is_empty() { return; }

//         let p_ptr = particles.as_mut_ptr();
//         let slop = env.tuning.physics.penetration_slop;
//         let restitution = env.tuning.physics.restitution;

//         for i in 0..registry.len() {
//             let a = registry.a_indices[i];
//             let b = registry.b_indices[i];
            
//             unsafe {
//                 let p_a = &mut *p_ptr.add(a);
//                 let p_b = &mut *p_ptr.add(b);

//                 let delta = p_a.pos - p_b.pos;
//                 let dist_sq = delta.length_squared();
                
//                 let target_dist = p_a.radius + p_b.radius + slop;
//                 let target_dist_sq = target_dist * target_dist;

//                 if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
//                     let dist = dist_sq.sqrt();
//                     let normal = delta / dist;

//                     // 🟢 FIXED: Restored true dynamic inverse mass evaluations
//                     let inv_mass_a = p_a.inv_mass;
//                     let inv_mass_b = p_b.inv_mass;
//                     let total_inv_mass = inv_mass_a + inv_mass_b;

//                     if total_inv_mass > V::Scalar::ZERO {
//                         let vel_a = p_a.pos - p_a.pos_old;
//                         let vel_b = p_b.pos - p_b.pos_old;
//                         let relative_vel = vel_a - vel_b;

//                         let normal_vel_mag = relative_vel.dot(normal);

//                         if normal_vel_mag < V::Scalar::ZERO {
//                             let target_normal_vel = -normal_vel_mag * restitution;
//                             let delta_vel_mag = target_normal_vel - normal_vel_mag;

//                             let vel_impulse_mag = delta_vel_mag / total_inv_mass;
//                             let vel_change_vector = normal * vel_impulse_mag;

//                             // Apply mass-weighted corrections to history positions
//                             p_a.pos_old -= vel_change_vector * inv_mass_a;
//                             p_b.pos_old += vel_change_vector * inv_mass_b;
//                         }
//                     }
//                 }
//             }
//         }
//     }

    
//     #[inline]
//     pub unsafe fn soa_apply_particle_restitution<V: Vector>( 
//         registry: &CollisionRegistry,  
//         positions: &mut [V],        // 🟢 FIXED: Continuous vector slice straight from storage layout
//         positions_old: &mut [V],    // 🟢 FIXED: Continuous vector slice straight from storage layout
//         inv_masses: &[V::Scalar],     
//         radii: &[V::Scalar],
//         env: &ParticleEnvironment<V>,
//     ) {
//         if registry.is_empty() { return; }

//         let p_ptr = positions.as_mut_ptr();
//         let o_ptr = positions_old.as_mut_ptr();
//         let m_ptr = inv_masses.as_ptr();
//         let r_ptr = radii.as_ptr();

//         let slop = env.tuning.physics.penetration_slop;
//         let restitution = env.tuning.physics.restitution;

//         for i in 0..registry.len() {
//             let a = registry.a_indices[i];
//             let b = registry.b_indices[i];

//             unsafe {
//                 let pos_a_ptr = p_ptr.add(a);
//                 let pos_b_ptr = p_ptr.add(b);
//                 let old_a_ptr = o_ptr.add(a);
//                 let old_b_ptr = o_ptr.add(b);

//                 // Use the Vector trait directly to find the displacement delta vector
//                 let delta = *pos_a_ptr - *pos_b_ptr;
//                 let dist_sq = delta.length_squared();

//                 let target_dist = *r_ptr.add(a) + *r_ptr.add(b) + slop;
//                 let target_dist_sq = target_dist * target_dist;

//                 if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
//                     let dist = dist_sq.sqrt();
//                     let normal = delta / dist; // Normalized direction vector

//                     let inv_mass_a = *m_ptr.add(a);
//                     let inv_mass_b = *m_ptr.add(b);
//                     let total_inv_mass = inv_mass_a + inv_mass_b;

//                     if total_inv_mass > V::Scalar::ZERO {
//                         // Calculate velocity vectors natively: Vel = Pos - PosOld
//                         let vel_a = *pos_a_ptr - *old_a_ptr;
//                         let vel_b = *pos_b_ptr - *old_b_ptr;
//                         let rel_vel = vel_a - vel_b;

//                         // Calculate normal velocity magnitude via Vector dot product
//                         let normal_vel_mag = rel_vel.dot(normal);

//                         if normal_vel_mag < V::Scalar::ZERO {
//                             let target_normal_vel = -normal_vel_mag * restitution;
//                             let delta_vel_mag = target_normal_vel - normal_vel_mag;

//                             let vel_impulse_mag = delta_vel_mag / total_inv_mass;
//                             let impulse = normal * vel_impulse_mag;

//                             // Adjust previous historical positions to alter the implicit velocity vector
//                             *old_a_ptr = *old_a_ptr - (impulse * inv_mass_a);
//                             *old_b_ptr = *old_b_ptr + (impulse * inv_mass_b);
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }



// // pub struct VerletCoreEngine;
// // impl VerletCoreEngine {
// //     pub fn execute_sub_step<V, L>(
// //         layout: &mut L,
// //         dt: f64,
// //         environment: &ParticleEnvironment<V>,
// //         scratch_radii: &[V::Scalar],
// //     ) where
// //         V: Vector,
// //         L: ParticleDataLayout<V>,
// //     {
// //         let len = layout.len();
// //         if len == 0 { return; }

// //         let sub_step_dt = <V::Scalar as FloatScalar>::from_f64(dt);
// //         let mut collisions = CollisionRegistry::new();

// //         // 1. Broadphase: Operates directly on the provided data layouts
// //         VerletPhysics.detect_collisions(len, scratch_radii, layout.positions_mut(), &mut collisions);

// //         // 2. Iterative Position Constraint Relaxation
// //         let inv_mass = <V::Scalar as FloatScalar>::from_f64(1.0);
// //         let pos_slice = layout.positions_mut();
        
// //         for _ in 0..environment.tuning.collision_iterations {
// //             for collision in &collisions.pairs {
// //                 let a = collision.a_index;
// //                 let b = collision.b_index;
// //                 if a == b { continue; }

// //                 // Safe Simultaneous Index Split
// //                 if a < b {
// //                     let (left, right) = pos_slice.split_at_mut(b);
// //                     VerletPhysics::resolve_particle_collisions(
// //                         environment, &mut left[a], &mut right[0], 
// //                         scratch_radii[a], scratch_radii[b], inv_mass, inv_mass
// //                     );
// //                 } else {
// //                     let (left, right) = pos_slice.split_at_mut(a);
// //                     VerletPhysics::resolve_particle_collisions(
// //                         environment, &mut right[0], &mut left[b], 
// //                         scratch_radii[a], scratch_radii[b], inv_mass, inv_mass
// //                     );
// //                 }
// //             } 
// //         }

// //         let (pos_slice, old_slice) = layout.positions_and_old_mut();
// //         for collision in &collisions.pairs {
// //             let a = collision.a_index;
// //             let b = collision.b_index;
// //             if a == b { continue; }

// //             let (pos_a, pos_b) = if a < b {
// //                 let (left, right) = pos_slice.split_at_mut(b); (&left[a], &right[0])
// //             } else {
// //                 let (left, right) = pos_slice.split_at_mut(a); (&right[0], &left[b])
// //             };

// //             let (old_a, old_b) = if a < b {
// //                 let (left, right) = old_slice.split_at_mut(b); (&mut left[a], &mut right[0])
// //             } else {
// //                 let (left, right) = old_slice.split_at_mut(a); (&mut right[0], &mut left[b])
// //             };

// //             VerletPhysics::apply_particle_restitution(
// //                 &environment.tuning.physics, pos_a, pos_b, old_a, old_b,
// //                 scratch_radii[a], scratch_radii[b], inv_mass, inv_mass,
// //             );
// //         }

// //         // 4. Global Environmental Constraints
// //         let (pos_slice, old_slice) = layout.positions_and_old_mut();
// //         for i in 0..len {
// //             VerletPhysics::apply_position_constraints(
// //                 sub_step_dt, environment, scratch_radii[i], &mut pos_slice[i], &mut old_slice[i]
// //             );
// //         }

// //         // 5. Final Clamping & Writeback
// //         let sub_step_max_vel = environment.tuning.physics.max_velocity * sub_step_dt;
// //         let max_vel_squared = sub_step_max_vel * sub_step_max_vel;
// //         layout.commit_kinetics(max_vel_squared, sub_step_max_vel);
// //     }
// // }



 