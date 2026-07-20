use crate::{math::{FloatScalar, Vector}, sim::solver::particle::{environment::ParticleEnvironment, flags::CollisionFlags, 
space::{collision_registry::CollisionRegistry, grid::UniformGrid, grid_key::GridKey}}};
use std::hash::Hash;

pub struct VerletSoaCollision;
impl VerletSoaCollision {
    #[inline(always)]
    pub fn populate_grid<V, F>( 
        positions: &[V],  
        environment: &mut ParticleEnvironment<V, F>, 
    ) where 
        V: Vector + 'static,
       V::Quantized: Hash + Eq + Copy + GridKey,
        F: CollisionFlags + 'static, // Links cleanly to your environment strategy
    {
        let grid = &mut environment.space.grid;
        grid.populate_sort(positions);
    }
 

    #[inline(always)]
    fn run_collision_pass<V, F>(
        grid: &UniformGrid<V>,
        iterations_count: u64,
        mut resolve_pair: F,
    ) where
        V: Vector,
        V::Quantized: Hash + Eq + Copy + GridKey,
        F: FnMut(usize, usize),
    {
        type QKey<VecT> = <VecT as Vector>::Quantized;

        if iterations_count == 0 { return; }

        for _ in 0..iterations_count {
            for &cell_key in &grid.active_keys {
                // Safe fallback or keeping unwrap for logic correctness
                let cell = match grid.cells.get(&cell_key) {
                    Some(c) => c,
                    None => continue,
                };
                
                let indices = &cell.indices;
                let len = indices.len();

                // 1. INTRA-CELL (Safe windowing via upfront length check)
                if len >= 2 {
                    let indices = &indices[..len]; // Window bounds declaration
                    for i in 0..len - 1 {
                        let idx_a = indices[i];
                        for j in (i + 1)..len {
                            let idx_b = indices[j];
                            resolve_pair(idx_a, idx_b);
                        }
                    }
                }

                // 2. NEIGHBOR-CELL (Safe windowing for both slices)
                for &offset in QKey::<V>::OFFSETS { 
                    let neighbor_key = cell_key + offset;
                    if let Some(neighbor_cell) = grid.cells.get(&neighbor_key) {
                        let neighbor_indices = &neighbor_cell.indices;
                        let neighbor_len = neighbor_indices.len();
                        
                        if len > 0 && neighbor_len > 0 {
                            // Window both arrays to their actual lengths
                            let indices = &indices[..len];
                            let neighbor_indices = &neighbor_indices[..neighbor_len];

                            for i in 0..len {
                                let idx_a = indices[i];
                                for j in 0..neighbor_len {
                                    let idx_b = neighbor_indices[j];
                                    resolve_pair(idx_a, idx_b);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn resolve_collisions<V, A, F>( 
        positions: &mut [V],
        //positions_old: &mut [V],  
        inv_masses: &[V::Scalar],
        radii: &[V::Scalar], 
        registry: &mut CollisionRegistry, // 🟢 ADDED: Pass the reuseable tracking cache
        environment: &ParticleEnvironment<V, F>, 
    ) where 
        V: Vector + 'static,
        V::Scalar: 'static,
        V::Quantized: Hash + Eq + Copy + GridKey,
        F: CollisionFlags + 'static, 
    {
        let max_len = positions.len();
        
        if max_len == 0 
            || inv_masses.len() < max_len 
            || radii.len() < max_len 
        {
            return; 
        }

        if F::RESTITUTION {
            registry.clear();  
        }

        let positions = &mut positions[..max_len];
        //let positions_old = if F::RESTITUTION { &mut positions_old[..max_len] } else { &mut [] };
        let inv_masses = &inv_masses[..max_len];
        let radii = &radii[..max_len];

        let slop = environment.tuning.physics.penetration_slop;
        let bias = environment.tuning.physics.penetration_correction_bias;
        let base_jitter = environment.state.runtime_jitter;
        let iterations_count = environment.tuning.collision_iterations;

        struct PhysCtx<S, VecT> {
            slop: S, bias: S, base_jitter: VecT, 
        }
        let ctx = PhysCtx { slop, bias, base_jitter };

        // Clear previous frame history to keep indices fresh
        registry.clear();

        Self::run_collision_pass(&environment.space.grid, iterations_count, |a, b| {
            if a >= max_len || b >= max_len { return; }

            let (pos_a, pos_b, target_dist) = (positions[a], positions[b], radii[a] + radii[b]);
            let mut delta = pos_a - pos_b;
            let mut dist_sq = delta.length_squared();
            let target_dist_sq = target_dist * target_dist;

            if dist_sq == V::Scalar::ZERO {
                let sep_arr = [<V::Scalar as FloatScalar>::from_f64(0.0001), V::Scalar::ZERO];
                delta = V::from_slice(&sep_arr);
                dist_sq = delta.length_squared();
            }

            if dist_sq < target_dist_sq {
                let dist = dist_sq.sqrt();
                let raw_penetration = target_dist - dist;

                let is_valid_penetration = if F::USE_SLOP {
                    raw_penetration > ctx.slop
                } else {
                    raw_penetration > V::Scalar::ZERO
                };

                if is_valid_penetration {
                    let penetration = if F::USE_SLOP {
                        raw_penetration - ctx.slop
                    } else {
                        raw_penetration
                    };

                    let mut normal = delta / dist;

                    if F::JITTER {
                        let jitter_vec = normal.mul_elementwise(ctx.base_jitter);
                        normal = normal + jitter_vec;
                        let normal_len_sq = normal.length_squared();
                        if normal_len_sq > V::Scalar::ZERO {
                            normal = normal / normal_len_sq.sqrt();
                        }
                    }

                    let inv_mass_a = inv_masses[a];
                    let inv_mass_b = inv_masses[b];
                    let total_inv_mass = inv_mass_a + inv_mass_b;

                    if total_inv_mass > V::Scalar::ZERO {
                        let response_magnitude = if F::USE_BIAS {
                            (penetration * ctx.bias) / total_inv_mass
                        } else {
                            penetration / total_inv_mass
                        };

                        let displacement_a = normal * (response_magnitude * inv_mass_a);
                        let displacement_b = normal * (response_magnitude * inv_mass_b);

                        positions[a] = pos_a + displacement_a;
                        positions[b] = pos_b - displacement_b;

                    
                        if F::RESTITUTION {
                            registry.push(a, b);
                        }
                    }
                }
            }
        });
    }

    /// Applies a clean velocity bounce pass across all pairs that collided during the frame,
    /// executing strictly AFTER all iterative position relaxation passes have finished.
    #[inline(always)]
    pub fn apply_particle_restitution<V, F>( 
        registry: &CollisionRegistry,  
        positions: &[V],            
        positions_old: &mut [V],    
        inv_masses: &[V::Scalar],     
        radii: &[V::Scalar],
        environment: &ParticleEnvironment<V, F>,
    ) where
        V: Vector + 'static,
        V::Scalar: FloatScalar + 'static,
        F: CollisionFlags + 'static, // Harness the zero-cost compiler optimization toggles
    {
        // 🟢 COMPILE-TIME OPTIMIZATION OVERRIDE:
        // If this is a Fluid or Fountain simulation, the compiler erases this entire loop body 
        // from the executable block. Absolute zero runtime execution overhead.
        if !F::RESTITUTION || registry.is_empty() { 
            return; 
        }

        let max_len = positions.len();
        if max_len == 0 || positions_old.len() < max_len || inv_masses.len() < max_len || radii.len() < max_len {
            return;
        }

        // Sub-slice everything upfront to guarantee the compiler completely eliminates all bounds checks
        let positions = &positions[..max_len];
        let positions_old = &mut positions_old[..max_len];
        let inv_masses = &inv_masses[..max_len];
        let radii = &radii[..max_len];

        let slop = environment.tuning.physics.penetration_slop;
        let restitution = environment.tuning.physics.restitution;

        if restitution <= V::Scalar::ZERO { 
            return; 
        }

        // 🚀 SAFE & AUTO-VECTORIZED PAIRWISE PASS:
        // Processes real inter-particle impact vectors across recorded collision history
        for i in 0..registry.len() {
            let a = registry.a_indices[i];
            let b = registry.b_indices[i];

            // Safety check against malicious registry corruptions or old tracking data
            if a >= max_len || b >= max_len { continue; }

            // Extract values directly through safe, zero-overhead slice indexers
            let delta = positions[a] - positions[b];
            let dist_sq = delta.length_squared();

            let target_dist = radii[a] + radii[b] + slop;
            let target_dist_sq = target_dist * target_dist;

            if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
                let dist = dist_sq.sqrt();
                let normal = delta / dist; 

                let inv_mass_a = inv_masses[a];
                let inv_mass_b = inv_masses[b];
                let total_inv_mass = inv_mass_a + inv_mass_b;

                if total_inv_mass > V::Scalar::ZERO {
                    // Extract exact historical velocity vectors natively: Vel = Pos - PosOld
                    let vel_a = positions[a] - positions_old[a];
                    let vel_b = positions[b] - positions_old[b];
                    let rel_vel = vel_a - vel_b;

                    // Calculate normal velocity magnitude via Vector dot product
                    let normal_vel_mag = rel_vel.dot(normal);

                    if normal_vel_mag < V::Scalar::ZERO {
                        let target_normal_vel = -normal_vel_mag * restitution;
                        let delta_vel_mag = target_normal_vel - normal_vel_mag;

                        let vel_impulse_mag = delta_vel_mag / total_inv_mass;
                        let impulse = normal * vel_impulse_mag;

                        // Safely adjust historical buffers to instantly apply the bounce velocity
                        positions_old[a] = positions_old[a] - (impulse * inv_mass_a);
                        positions_old[b] = positions_old[b] + (impulse * inv_mass_b);
                    }
                }
            }
        }
    }

// #[inline(always)]
// pub fn resolve_collisions<
//     V, 
//     A, 
//     const JITTER: bool, 
//     const RESTITUTION: bool, 
//     const USE_BIAS: bool, 
//     const USE_SLOP: bool
// >( 
//     positions: &mut [V],
//     positions_old: &mut [V],  
//     inv_masses: &[V::Scalar],
//     radii: &[V::Scalar],
//     v_dt: V::Scalar, 
//     environment: &ParticleEnvironment<V>, 
// ) where 
//     V: Vector,
//     V::Quantized: Hash + Eq + Copy,
//     A: ParticleAttributes<V>, 
// {
//     let max_len = positions.len();
    
//     if max_len == 0 
//         || (RESTITUTION && positions_old.len() < max_len)
//         || inv_masses.len() < max_len 
//         || radii.len() < max_len 
//     {
//         return; 
//     }

//     let positions = &mut positions[..max_len];
//     let positions_old = if RESTITUTION { &mut positions_old[..max_len] } else { &mut [] };
//     let inv_masses = &inv_masses[..max_len];
//     let radii = &radii[..max_len];

//     let slop = environment.tuning.physics.penetration_slop;
//     let bias = environment.tuning.physics.penetration_correction_bias;
//     let base_jitter = environment.state.runtime_jitter;
//     let restitution = environment.tuning.physics.restitution;
//     let iterations_count = environment.tuning.collision_iterations;

//     struct PhysCtx<S, VecT> {
//         slop: S, bias: S, base_jitter: VecT, restitution: S, v_dt: S,
//     }
//     let ctx = PhysCtx { slop, bias, base_jitter, restitution, v_dt };

//     Self::run_collision_pass(&environment.space.grid, iterations_count, |a, b| {
//         if a >= max_len || b >= max_len { return; }

//         let (pos_a, pos_b, target_dist) = (positions[a], positions[b], radii[a] + radii[b]);
//         let mut delta = pos_a - pos_b;
//         let mut dist_sq = delta.length_squared();
//         let target_dist_sq = target_dist * target_dist;

//         if dist_sq == V::Scalar::ZERO {
//             let sep_arr = [<V::Scalar as FloatScalar>::from_f64(0.0001), V::Scalar::ZERO];
//             delta = V::from_slice(&sep_arr);
//             dist_sq = delta.length_squared();
//         }

//         if dist_sq < target_dist_sq {
//             let dist = dist_sq.sqrt();
//             let raw_penetration = target_dist - dist;

            
//             let is_valid_penetration = if USE_SLOP {
//                 raw_penetration > ctx.slop
//             } else {
//                 raw_penetration > V::Scalar::ZERO
//             };

//             if is_valid_penetration {
//                 let penetration = if USE_SLOP {
//                     raw_penetration - ctx.slop
//                 } else {
//                     raw_penetration
//                 };

//                 let mut normal = delta / dist;

//                 if JITTER {
//                     let jitter_vec = normal.mul_elementwise(ctx.base_jitter);
//                     normal = normal + jitter_vec;
//                     let normal_len_sq = normal.length_squared();
//                     if normal_len_sq > V::Scalar::ZERO {
//                         normal = normal / normal_len_sq.sqrt();
//                     }
//                 }

//                 let inv_mass_a = inv_masses[a];
//                 let inv_mass_b = inv_masses[b];
//                 let total_inv_mass = inv_mass_a + inv_mass_b;

//                 if total_inv_mass > V::Scalar::ZERO {
//                     // 🟢 COMPILE-TIME STRIPPING (BIAS OPTIMIZATION):
//                     // If USE_BIAS is false, the engine skips ctx.bias scaling entirely. 
//                     // This creates a pure positional relaxation impulse.
//                     let response_magnitude = if USE_BIAS {
//                         (penetration * ctx.bias) / total_inv_mass
//                     } else {
//                         penetration / total_inv_mass
//                     };

//                     let displacement_a = normal * (response_magnitude * inv_mass_a);
//                     let displacement_b = normal * (response_magnitude * inv_mass_b);

//                     let new_pos_a = pos_a + displacement_a;
//                     let new_pos_b = pos_b - displacement_b;

//                     if RESTITUTION && ctx.restitution > V::Scalar::ZERO {
//                         let p_old_a = positions_old[a];
//                         let p_old_b = positions_old[b];

//                         let vel_a = (new_pos_a - p_old_a) / ctx.v_dt;
//                         let vel_b = (new_pos_b - p_old_b) / ctx.v_dt;
//                         let relative_velocity = vel_a - vel_b;
//                         let vel_along_normal = relative_velocity.dot(normal);

//                         if vel_along_normal < V::Scalar::ZERO {
//                             let impulse_scalar = -(V::Scalar::ONE + ctx.restitution) * vel_along_normal;
//                             let impulse_magnitude = impulse_scalar / total_inv_mass;
//                             let impulse_vec = normal * impulse_magnitude;
//                             let history_shift = impulse_vec * ctx.v_dt;

//                             positions_old[a] = p_old_a - (history_shift * inv_mass_a);
//                             positions_old[b] = p_old_b + (history_shift * inv_mass_b);
//                         }
//                     }

//                     positions[a] = new_pos_a;
//                     positions[b] = new_pos_b;
//                 }
//             }
//         }
//     });
// }
 

// pub unsafe fn jitter_restitution_bias_penetration<V>(
//     grid: &UniformGrid<V>,
//     positions: &mut [V],
//     positions_old: &mut [V],  
//     inv_masses: &[V::Scalar],
//     radii: &[V::Scalar],
//     v_dt: V::Scalar, 
//     environment: &ParticleEnvironment<V>,
// ) where 
//     V: Vector,
//     V::Quantized: Hash + Eq + Copy,
// {
//     type QKey<VecT> = <VecT as Vector>::Quantized;

//     let slop = environment.tuning.physics.penetration_slop;
//     let bias = environment.tuning.physics.penetration_correction_bias;
//     let base_jitter = environment.state.runtime_jitter;
//     let restitution = environment.tuning.physics.restitution;

//     let iterations_count = environment.tuning.collision_iterations;
//     if iterations_count == 0 { return; }

//     struct PhysCtx<S, VecT> {
//         slop: S,
//         bias: S,
//         base_jitter: VecT,  
//         restitution: S,
//         v_dt: S,
//     }

//     let ctx = PhysCtx { 
//         slop, 
//         bias, 
//         base_jitter, 
//         restitution, 
//         v_dt 
//     };
 
//     // Helper function signature is clean; inner unsafe blocks satisfy the compiler
//     #[inline(always)]
//     fn resolve_pair_optimized<V>(
//         a: usize, 
//         b: usize, 
//         positions: &mut [V],
//         positions_old: &mut [V],  
//         inv_masses: &[V::Scalar],
//         radii: &[V::Scalar],
//         ctx: &PhysCtx<V::Scalar, V>,
//     ) where 
//         V: Vector 
//     {
//         // Isolate unsafe memory reads
//         let (pos_a, pos_b, target_dist) = unsafe {
//             (
//                 *positions.get_unchecked(a),
//                 *positions.get_unchecked(b),
//                 *radii.get_unchecked(a) + *radii.get_unchecked(b)
//             )
//         };

//         let mut delta = pos_a - pos_b;
//         let mut dist_sq = delta.length_squared();
//         let target_dist_sq = target_dist * target_dist;

//         if dist_sq == V::Scalar::ZERO {
//             let sep_arr = [<V::Scalar as FloatScalar>::from_f64(0.0001), V::Scalar::ZERO];
//             delta = V::from_slice(&sep_arr);
//             dist_sq = delta.length_squared();
//         }

//         if dist_sq < target_dist_sq {
//             let dist = dist_sq.sqrt();
//             let raw_penetration = target_dist - dist;

//             if raw_penetration > ctx.slop {
//                 let penetration = raw_penetration - ctx.slop;
//                 let mut normal = delta / dist;

//                 let jitter_vec = normal.mul_elementwise(ctx.base_jitter);
//                 normal = normal + jitter_vec;
//                 let normal_len_sq = normal.length_squared();
//                 if normal_len_sq > V::Scalar::ZERO {
//                     normal = normal / normal_len_sq.sqrt();
//                 }

//                 let (inv_mass_a, inv_mass_b) = unsafe {
//                     (*inv_masses.get_unchecked(a), *inv_masses.get_unchecked(b))
//                 };
//                 let total_inv_mass = inv_mass_a + inv_mass_b;

//                 if total_inv_mass > V::Scalar::ZERO {
//                     let response_magnitude = (penetration * ctx.bias) / total_inv_mass;
//                     let displacement_a = normal * (response_magnitude * inv_mass_a);
//                     let displacement_b = normal * (response_magnitude * inv_mass_b);

//                     let new_pos_a = pos_a + displacement_a;
//                     let new_pos_b = pos_b - displacement_b;

//                     if ctx.restitution > V::Scalar::ZERO {
//                         let (p_old_a, p_old_b) = unsafe {
//                             (*positions_old.get_unchecked(a), *positions_old.get_unchecked(b))
//                         };

//                         let vel_a = (new_pos_a - p_old_a) / ctx.v_dt;
//                         let vel_b = (new_pos_b - p_old_b) / ctx.v_dt;
//                         let relative_velocity = vel_a - vel_b;
//                         let vel_along_normal = relative_velocity.dot(normal);

//                         if vel_along_normal < V::Scalar::ZERO {
//                             let impulse_scalar = -(V::Scalar::ONE + ctx.restitution) * vel_along_normal;
//                             let impulse_magnitude = impulse_scalar / total_inv_mass;
//                             let impulse_vec = normal * impulse_magnitude;
//                             let history_shift = impulse_vec * ctx.v_dt;

//                             unsafe {
//                                 *positions_old.get_unchecked_mut(a) = p_old_a - (history_shift * inv_mass_a);
//                                 *positions_old.get_unchecked_mut(b) = p_old_b + (history_shift * inv_mass_b);
//                             }
//                         }
//                     }

//                     // Isolate unsafe memory writes
//                     unsafe {
//                         *positions.get_unchecked_mut(a) = new_pos_a;
//                         *positions.get_unchecked_mut(b) = new_pos_b;
//                     }
//                 }
//             }
//         }
//     }

//     for _ in 0..iterations_count {
//         for &cell_key in &grid.active_keys {
//             // unwrap_unchecked requires unsafe
//             let cell = unsafe { grid.cells.get(&cell_key).unwrap_unchecked() };
//             let indices = &cell.indices;
//             let len = indices.len();

//             // 1. INTRA-CELL
//             if len >= 2 {
//                 for i in 0..len.saturating_sub(1) {
//                     let idx_a = unsafe { *indices.get_unchecked(i) };
//                     for j in (i + 1)..len {
//                         let idx_b = unsafe { *indices.get_unchecked(j) };
//                         resolve_pair_optimized(
//                             idx_a, 
//                             idx_b, 
//                             positions, positions_old, inv_masses, radii, &ctx
//                         );
//                     }
//                 }
//             }

//             // 2. NEIGHBOR-CELL
//             for &offset in QKey::<V>::OFFSETS { 
//                 let neighbor_key = cell_key + offset;
//                 if let Some(neighbor_cell) = grid.cells.get(&neighbor_key) {
//                     let neighbor_indices = &neighbor_cell.indices;
//                     let neighbor_len = neighbor_indices.len();
                    
//                     for i in 0..len {
//                         let idx_a = unsafe { *indices.get_unchecked(i) };
//                         for j in 0..neighbor_len {
//                             let idx_b = unsafe { *neighbor_indices.get_unchecked(j) };
//                             resolve_pair_optimized(
//                                 idx_a, 
//                                 idx_b, 
//                                 positions, positions_old, inv_masses, radii, &ctx
//                             );
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

}