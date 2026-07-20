use crate::{math::{Vector, FloatScalar}, 
sim::{lifecycle::ElementLifecycle, 
    solver::particle::{environment::ParticleEnvironment, flags::CollisionFlags, 
        lifecycle::Stream, verlet_particle::VerletParticle}, storage::ElementStorage}};


pub struct FountainLifecycle<V: Vector> {
    pub stream: Stream<V>,
}


impl<V: Vector> FountainLifecycle<V> { 
    #[inline]
    pub fn new(stream: Stream<V>) -> Self {
        Self { stream }
    }
}

impl<St, V, F> ElementLifecycle<St, ParticleEnvironment<V, F>> for FountainLifecycle<V>
where
    St: ElementStorage<Element = VerletParticle<V>> + 'static,
    V: Vector + 'static,
    V::Scalar: FloatScalar + 'static,
    F: CollisionFlags + 'static,
{
    #[inline(always)]
    fn process_lifecycle(
        &mut self, 
        storage: &mut St, 
        tick: u64, 
        dt: f64,
        _scratch_indices: &mut Vec<usize>, 
        environment: &ParticleEnvironment<V, F>  
    ) {
 
        let bounds = &environment.space.bounds;  
        let v_dt = V::Scalar::from_f64(dt);

     
        if self.stream.should_emit(tick) {
            let burst_count = 12;
            
            // Core structural spacing parameters
            let spacing_factor = V::Scalar::from_f64(2.2);
            let fat_diameter = self.stream.radius.clone() * spacing_factor;
            
            // Extract base trajectory profiles
            let base_pos = self.stream.get_position(bounds);
            let spawned_vel = self.stream.velocity.clone();
            let radius = self.stream.radius.clone();
            let density = self.stream.density.clone();

            // 🚀 1. DYNAMIC PERPENDICULAR NOZZLE AXIS DERIVATION
            // Normalize the velocity vector to extract the pure travel heading
            let stream_heading = spawned_vel.clone().normalize();
            
            // Compute the side vector perpendicular to the velocity field
            // (Utilizes your native Vector trait's perpendicular/cross-product utilities)
            let width_axis = stream_heading.perpendicular_vector(); 

            for i in 0..burst_count {
                // Calculate row and column indices (3 columns wide)
                let col = (i % 3) as f64; // 0.0 (Left), 1.0 (Center), 2.0 (Right)
                let row = (i / 3) as f64; // 0.0, 1.0, 2.0... (Height layers)

                // Maps col to [-1.0, 0.0, 1.0] to center the nozzle alignment nicely
                let width_offset_scalar = V::Scalar::from_f64(col - 1.0);
                
                // Height offset stacks each row backward in time along the flow vector
                let height_offset_scalar = V::Scalar::from_f64(row);

                // Map colors linearly based on stream height layers
                let total_rows = f64::max(1.0, (burst_count as f64 / 3.0).ceil() - 1.0);
                let burst_percent = if total_rows > 0.0 { row / total_rows } else { 0.0 };
                let color = environment.state.get_color(burst_percent);

                // 🚀 2. CALCULATE ORIENTED SPATIAL SHIFTS
                // Offset horizontally along the dynamically derived perpendicular nozzle width vector
                let horizontal_shift = width_axis.clone() * (fat_diameter.clone() * width_offset_scalar);
                
                // Offset vertically backwards along the stream travel heading
                let vertical_shift = stream_heading.clone() * (fat_diameter.clone() * height_offset_scalar);

                // Combine base position with oriented injection grid adjustments
                let spawned_pos = base_pos.clone() + horizontal_shift - vertical_shift;

                // Compute historical Verlet position: pos_old = pos - vel * dt
                let spawned_pos_old = spawned_pos.clone() - (spawned_vel.clone() * v_dt.clone());

                // Construct full structural particle context
                let particle = VerletParticle {
                    pos:      spawned_pos,
                    pos_old:  spawned_pos_old,
                    acc:      V::ZERO,
                    radius:   radius.clone(),
                    color,
                    inv_mass: V::Scalar::ONE / density.clone(),  
                };
                
                // Direct zero-friction insertion into backing layout arrays
                storage.push(particle);
            }
        }
        
        // -----------------------------------------------------------------
        // PHASE 2: CULLING (Optional placeholder hook)
        // -----------------------------------------------------------------
        // If your Verlet particles have lifespans, loop over the active view 
        // to query lookups and populate scratch_indices for tail-swap cleanup here!
    }
}



// pub enum FireworkStage {
//     RocketShell, // Rising projectile
//     SparkFragment, // Exploded debris
// }

// // Marker columns for our firework layout payload
// pub enum FireworkColumns {
//     Position = 0,
//     Velocity = 1,
//     TimeLeft = 2,
//     Stage = 3,
// }

// pub struct FireworksController {
//     pub max_sparks_per_explosion: usize,
// }

// impl<St, Env> InteractiveLifecycle<St, Env> for FireworksController
// where
//     St: ElementStorage + 'static,
//     // Ensure the storage engine exposes our runtime bitmask views
//     for<'a> St::View<'a>: Into<SoaView<'a, St::Element>>, 
// {
//     fn process_lifecycle(
//         &mut self, 
//         storage: &mut St, 
//         scratch_indices: &mut Vec<usize>, 
//         _environment: &Env
//     ) {
//         let initial_len = storage.len();
//         if initial_len == 0 { return; }

//         // 1. Safely acquire a snapshot view of our stable descriptors
//         let view = storage.view();
//         let view_ref: SoaView<'_, St::Element> = view.into();
        
//         // Pull clean read-only slices out of the storage mask
//         let positions = view_ref.slice(FireworkColumns::Position);
//         let velocities = view_ref.slice(FireworkColumns::Velocity);
//         let times_left = view_ref.slice(FireworkColumns::TimeLeft);
//         let stages     = view_ref.slice(FireworkColumns::Stage);

//         // 2. Scan active elements up to the initial frame length 
//         // (Any new sparks pushed inside this loop are processed on the next frame)
//         for idx in 0..initial_len {
//             // Check if this specific particle has run out of time
//             if times_left[idx] <= 0.0 {
//                 scratch_indices.push(idx); // Mark parent shell for deletion

//                 if let FireworkStage::RocketShell = stages[idx] {
//                     // --- THE EXPLOSION TRIGGER ---
//                     // Capture the exact spatial location where the shell died
//                     let blast_center = positions[idx];
//                     let parent_vel = velocities[idx];

//                     for i in 0..self.max_sparks_per_explosion {
//                         let random_dir = compute_sphere_distribution(i);
                        
//                         // Fabricate a new child element fragment spraying outward
//                         let spark = St::Element::new_spark(
//                             blast_center, 
//                             parent_vel + random_dir, 
//                             1.5 // lifespan in seconds
//                         );

//                         // Direct engine push! Append new children to the tail of the array.
//                         // Safe because the layout engine handles reallocations cleanly.
//                         storage.push(spark);
//                     }
//                 }
//             }
//         }
//     }
// }

// fn compute_sphere_distribution(index: usize) -> [f32; 3] {
//     // Simple mock math for radial projection vectors
//     let angle = (index as f32) * 0.5;
//     [angle.cos() * 5.0, angle.sin() * 5.0, (index as f32).sin() * 2.0]
// }
