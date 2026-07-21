use crate::{math::{FloatScalar, Vector}, sim::solver::particle::{environment::ParticleEnvironment, flags::CollisionFlags, 
space::{collision_registry::CollisionRegistry, grid_key::GridKey}}};
use std::hash::Hash;

pub struct VerletSoaCollision;
impl VerletSoaCollision {
    
    
    #[inline(always)]
    pub fn populate_grid<V,F>( 
        positions: &[V],  
        environment: &mut ParticleEnvironment<V, F>, 
    ) where 
        V: Vector,
         V::Quantized: Hash + Eq + Copy + GridKey,
        F: CollisionFlags + 'static,
    {
        environment.space.grid.populate_sort(positions);
    }

    #[inline(always)]
    pub fn resolve_collisions<V, F>( 
        positions: &mut [V],
        inv_masses: &[V::Scalar],
        radii: &[V::Scalar], 
        registry: &mut CollisionRegistry,
        environment: &ParticleEnvironment<V, F>, 
     ) where 
        V: Vector,
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

        let iterations_count = environment.tuning.collision_iterations;
        if iterations_count == 0 { return; }

        if F::RESTITUTION {
            registry.clear();  
        }

        // FIXED: Upfront raw unchecked window bounds mapping
        let pos_slice = unsafe { positions.get_unchecked_mut(0..max_len) };
        let inv_mass_slice = unsafe { inv_masses.get_unchecked(0..max_len) };
        let radii_slice = unsafe { radii.get_unchecked(0..max_len) };

        let slop = environment.tuning.physics.penetration_slop;
        let bias = environment.tuning.physics.penetration_correction_bias;
        let base_jitter = environment.state.runtime_jitter;
        let grid = &environment.space.grid;

        type QKey<VecT> = <VecT as Vector>::Quantized;

        // Clear previous frame tracking history cleanly
        registry.clear();

        // FIXED: Inlined the entire loop pass directly to eliminate closure reference indirection overhead.
        for _ in 0..iterations_count {
            for &cell_key in &grid.active_keys {
                let cell = match grid.cells.get(&cell_key) {
                    Some(c) => c,
                    None => continue,
                };
                
                let indices = &cell.indices;
                let len = indices.len();
                if len == 0 { continue; }
                let cell_indices = unsafe { indices.get_unchecked(0..len) };

                // =========================================================
                // 1. INTRA-CELL PAIR RESOLUTION 
                // =========================================================
                if len >= 2 {
                    for i in 0..len - 1 {
                        let a = unsafe { *cell_indices.get_unchecked(i) };
                        if a >= max_len { continue; }

                        for j in (i + 1)..len {
                            let b = unsafe { *cell_indices.get_unchecked(j) };
                            if b >= max_len { continue; }

                            // Inline execution block with zero-cost pointer mapping
                            unsafe {
                                Self::resolve_single_pair::<V,F>(
                                    a, b, pos_slice, inv_mass_slice, radii_slice,
                                    slop, bias, base_jitter, registry
                                );
                            }
                        }
                    }
                }

                // =========================================================
                // 2. NEIGHBOR-CELL PAIR RESOLUTION
                // =========================================================
                for &offset in QKey::<V>::OFFSETS { 
                    let neighbor_key = cell_key + offset;
                    if let Some(neighbor_cell) = grid.cells.get(&neighbor_key) {
                        let neighbor_indices = &neighbor_cell.indices;
                        let neighbor_len = neighbor_indices.len();
                        
                        if neighbor_len > 0 {
                            let n_indices = unsafe { neighbor_indices.get_unchecked(0..neighbor_len) };

                            for i in 0..len {
                                let a = unsafe { *cell_indices.get_unchecked(i) };
                                if a >= max_len { continue; }

                                for j in 0..neighbor_len {
                                    let b = unsafe { *n_indices.get_unchecked(j) };
                                    if b >= max_len { continue; }

                                    unsafe {
                                        Self::resolve_single_pair::<V,F>(
                                            a, b, pos_slice, inv_mass_slice, radii_slice,
                                            slop, bias, base_jitter, registry
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Internal fast-path calculator helper for resolving an individual interaction pair.
    #[inline(always)]
    unsafe fn resolve_single_pair<V,F>(
        a: usize,
        b: usize,
        pos_slice: &mut [V],
        inv_mass_slice: &[V::Scalar],
        radii_slice: &[V::Scalar],
        slop: V::Scalar,
        bias: V::Scalar,
        base_jitter: V,
        registry: &mut CollisionRegistry,
    ) where 
        V: Vector,
         V::Quantized: Hash + Eq + Copy + GridKey,
        F: CollisionFlags + 'static,
    {
        // FIXED: Explicit localized unsafe blocks for strict modern Rust compiler enforcement
        let (pos_a, pos_b, target_dist) = unsafe {
            (
                *pos_slice.get_unchecked(a),
                *pos_slice.get_unchecked(b),
                *radii_slice.get_unchecked(a) + *radii_slice.get_unchecked(b),
            )
        };
        
        let mut delta = pos_a - pos_b;
        let mut dist_sq = delta.length_squared();
        let target_dist_sq = target_dist * target_dist;

        // Protection against singular zero alignment distances
        if dist_sq == V::Scalar::ZERO {
            let sep_arr = [V::Scalar::from_f64(0.0001), V::Scalar::ZERO];
            delta = V::from_slice(&sep_arr);
            dist_sq = delta.length_squared();
        }

        if dist_sq < target_dist_sq {
            let dist = dist_sq.sqrt();
            let raw_penetration = target_dist - dist;

            let is_valid_penetration = if F::USE_SLOP {
                raw_penetration > slop
            } else {
                raw_penetration > V::Scalar::ZERO
            };

            if is_valid_penetration {
                let penetration = if F::USE_SLOP {
                    raw_penetration - slop
                } else {
                    raw_penetration
                };

                let mut normal = delta / dist;

                if F::JITTER {
                    let jitter_vec = normal.mul_elementwise(base_jitter);
                    normal = normal + jitter_vec;
                    let normal_len_sq = normal.length_squared();
                    if normal_len_sq > V::Scalar::ZERO {
                        normal = normal / normal_len_sq.sqrt();
                    }
                }

                // FIXED: Explicit localized unsafe block for mass extraction
                let (inv_mass_a, inv_mass_b) = unsafe {
                    (
                        *inv_mass_slice.get_unchecked(a),
                        *inv_mass_slice.get_unchecked(b),
                    )
                };
                let total_inv_mass = inv_mass_a + inv_mass_b;

                if total_inv_mass > V::Scalar::ZERO {
                    let response_magnitude = if F::USE_BIAS {
                        (penetration * bias) / total_inv_mass
                    } else {
                        penetration / total_inv_mass
                    };

                    let displacement_a = normal * (response_magnitude * inv_mass_a);
                    let displacement_b = normal * (response_magnitude * inv_mass_b);

                    // FIXED: Explicit localized unsafe block for final mutation writes
                    unsafe {
                        *pos_slice.get_unchecked_mut(a) = pos_a + displacement_a;
                        *pos_slice.get_unchecked_mut(b) = pos_b - displacement_b;
                    }

                    if F::RESTITUTION {
                        registry.push(a, b);
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn apply_particle_restitution<V, F>( 
        registry: &CollisionRegistry,  
        positions: &[V],            
        positions_old: &mut [V],    
        inv_masses: &[V::Scalar],     
        radii: &[V::Scalar],
        environment: &ParticleEnvironment<V, F>,
    ) where
        V: Vector,
        F: CollisionFlags + 'static,
    {
        // 🟢 COMPILE-TIME BLANKET REMOVAL: Clean, zero-cost branch elimination
        if !F::RESTITUTION || registry.is_empty() { 
            return; 
        }

        let max_len = positions.len();
        if max_len == 0 
            || positions_old.len() < max_len 
            || inv_masses.len() < max_len 
            || radii.len() < max_len 
        {
            return;
        }

        let restitution = environment.tuning.physics.restitution;
        if restitution <= V::Scalar::ZERO { 
            return; 
        }

        let slop = environment.tuning.physics.penetration_slop;
        let reg_len = registry.len();

        // FIXED: Convert slices to raw pointers upfront. 
        // This solves the multiple mutable borrow error and gives LLVM clean aliasing routes.
        let pos_ptr = positions.as_ptr();
        let pos_old_ptr = positions_old.as_mut_ptr();
        let inv_mass_ptr = inv_masses.as_ptr();
        let radii_ptr = radii.as_ptr();

        // Ensure safe window bounds for the registry histories
        let reg_a = unsafe { registry.a_indices.get_unchecked(0..reg_len) };
        let reg_b = unsafe { registry.b_indices.get_unchecked(0..reg_len) };

        for i in 0..reg_len {
            unsafe {
                let a = *reg_a.get_unchecked(i);
                let b = *reg_b.get_unchecked(i);

                // Hardware verification gate
                if a >= max_len || b >= max_len { continue; }

                // FIXED: Read values natively using raw pointer offsets to completely bypass bounds tracking
                let pos_a = *pos_ptr.add(a);
                let pos_b = *pos_ptr.add(b);

                let delta = pos_a - pos_b;
                let dist_sq = delta.length_squared();

                let target_dist = *radii_ptr.add(a) + *radii_ptr.add(b) + slop;
                let target_dist_sq = target_dist * target_dist;

                // Branchless combination gate: Prevents deep nested indentation parsing
                if dist_sq < target_dist_sq && dist_sq > V::Scalar::ZERO {
                    let dist = dist_sq.sqrt();
                    let normal = delta / dist; 

                    let inv_mass_a = *inv_mass_ptr.add(a);
                    let inv_mass_b = *inv_mass_ptr.add(b);
                    let total_inv_mass = inv_mass_a + inv_mass_b;

                    if total_inv_mass > V::Scalar::ZERO {
                        // Extract historical velocity vectors natively: Vel = Pos - PosOld
                        let vel_a = pos_a - *pos_old_ptr.add(a);
                        //let vel_b = pos_b - *pos_ptr.add(b); // Note: keeping your logic mapping matching pos_old read intents
                        let vel_b_actual = pos_b - *pos_old_ptr.add(b); // Corrected to use pos_old for b
                        let rel_vel = vel_a - vel_b_actual;

                        let normal_vel_mag = rel_vel.dot(normal);

                        // Only apply an impulse if particles are moving toward each other
                        if normal_vel_mag < V::Scalar::ZERO {
                            let target_normal_vel = -normal_vel_mag * restitution;
                            let delta_vel_mag = target_normal_vel - normal_vel_mag;

                            let vel_impulse_mag = delta_vel_mag / total_inv_mass;
                            let impulse = normal * vel_impulse_mag;

                            // Direct hardware writing back into the memory locations via raw pointers
                            *pos_old_ptr.add(a) = *pos_old_ptr.add(a) - (impulse * inv_mass_a);
                            *pos_old_ptr.add(b) = *pos_old_ptr.add(b) + (impulse * inv_mass_b);
                        }
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