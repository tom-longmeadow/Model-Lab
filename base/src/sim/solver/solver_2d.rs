use crate::{math::DVec2, sim::solver::{partition::collision::CollisionRegistry, tuning::ParticlePhysicsTuning}};

 pub struct Solver2D;
impl Solver2D {
    
    pub fn detect_collisions(
        &self,
        len: usize,
        scratch_radii: &[f64],
        scratch_pos: &[DVec2], // Assuming a Vector type like Vec2{x, y}
        registry: &mut CollisionRegistry<DVec2>
    ) {
        registry.clear();

       
        
        for i in 0..len {
            let radius_a = scratch_radii[i];
            let pos_a = scratch_pos[i];

            for j in (i + 1)..len {
                let radius_b = scratch_radii[j];
                let pos_b = scratch_pos[j];

                let delta = pos_b - pos_a; // Direction pointing from A to B
                let distance_sq = delta.dot(delta);
                let min_dist = radius_a + radius_b;

                if distance_sq < min_dist * min_dist && distance_sq > 0.0 {
                    let distance = distance_sq.sqrt();
                    let penetration = min_dist - distance;
                    let normal = delta / distance; // Normalized vector A -> B

                    // Push to your registry. Note the custom constructor logic!
                    registry.push(i, j, normal, penetration);
                }
            }
        }
    }


    #[inline(always)]
    pub fn resolve_particle_collisions(
        tuning: &ParticlePhysicsTuning,
        pos_a: &mut DVec2,
        pos_b: &mut DVec2,
        radius_a: f64,
        radius_b: f64,
    ) {
        let delta = *pos_b - *pos_a;
        let distance_sq = delta.dot(delta);
        let min_dist = radius_a + radius_b;
        let min_dist_sq = min_dist * min_dist;

        if distance_sq < min_dist_sq && distance_sq > 0.0 {
            let distance = distance_sq.sqrt();
            let penetration = min_dist - distance;

            // 1. Apply Penetration Slop: Ignore tiny overlaps to reduce microscopic jitter
            if penetration > tuning.penetration_slop {
                let normal = delta / distance;

                // 2. Apply Baumgarte Bias: Only resolve a fraction of the overlap per iteration
                // This prevents the positional corrections from introducing wild phantom kinetic energy
                let corrected_penetration = (penetration - tuning.penetration_slop) * tuning.penetration_correction_bias;
                let separation = normal * (corrected_penetration * 0.5);

                *pos_a -= separation;
                *pos_b += separation;
            }
        }
    }


    // #[inline(always)]
    // pub fn resolve_particle_collisions(
    //     tuning: &ParticlePhysicsTuning,
    //     pos_a: &mut DVec2,
    //     pos_b: &mut DVec2,
    //     radius_a: f64,
    //     radius_b: f64,
    // ) {
    //     let delta = *pos_b - *pos_a;
    //     let distance_sq = delta.dot(delta);
    //     let min_dist = radius_a + radius_b;
    //     let min_dist_sq = min_dist * min_dist;

    //     if distance_sq < min_dist_sq && distance_sq > 0.0 {
    //         let distance = distance_sq.sqrt();
    //         let penetration = min_dist - distance;
    //         let normal = delta / distance;

    //         // Separate positions equally (0.5 weight each)
    //         let separation = normal * (penetration * 0.5);

    //         *pos_a -= separation;
    //         *pos_b += separation;
    //     }
    // }


//     /// Displaces particles by the movement of the window edges,
//     /// transferring the window's velocity into the system.
//    /// Displaces particles by the movement of the window edges across both dimensions.
//     /// By updating `pos` and ignoring `pos_old`, Verlet automatically
//     /// captures the window's kinetic velocity on the next physics step.
//     #[inline(always)]
//     pub fn push_by_bounds_2d(
//         pos: DVec2,
//         old_min: DVec2,
//         old_max: DVec2,
//         new_min: DVec2,
//         new_max: DVec2,
//         radius: f64,
//     ) -> DVec2 {
//         // Calculate axis displacements using our pure helper
//         let out_x = Self::push_by_bounds_axis(pos.x, old_min.x, old_max.x, new_min.x, new_max.x, radius);
//         let out_y = Self::push_by_bounds_axis(pos.y, old_min.y, old_max.y, new_min.y, new_max.y, radius);
        
//         DVec2::new(out_x, out_y)
//     }

//     /// Pure 1D helper to isolate calculation work per-axis
//     #[inline(always)]
//     fn push_by_bounds_axis(
//         pos: f64,
//         old_min: f64,
//         old_max: f64,
//         new_min: f64,
//         new_max: f64,
//         radius: f64,
//     ) -> f64 {
//         let old_min_allowed = old_min + radius;
//         let old_max_allowed = old_max - radius;
//         let new_min_allowed = new_min + radius;
//         let new_max_allowed = new_max - radius;

//         let mut current_pos = pos;

//         let old_range = old_max_allowed - old_min_allowed;
//         if old_range <= 0.0 { return current_pos; }
        
//         // Determine edge proximity relative to the old viewport layout
//         let pct = (current_pos - old_min_allowed) / old_range;

//         let min_delta = new_min_allowed - old_min_allowed;
//         let max_delta = new_max_allowed - old_max_allowed;

//         if pct < 0.5 {
//             // Closer to Left/Top wall
//             let weight = (1.0 - (pct * 2.0)).max(0.0);
//             current_pos += min_delta * weight;
//         } else {
//             // Closer to Right/Bottom wall
//             let weight = ((pct - 0.5) * 2.0).max(0.0);
//             current_pos += max_delta * weight;
//         }

//         // Exact safety clamp bounds check
//         if current_pos < new_min_allowed {
//             current_pos = new_min_allowed;
//         } else if current_pos > new_max_allowed {
//             current_pos = new_max_allowed;
//         }

//         current_pos
//     }
}