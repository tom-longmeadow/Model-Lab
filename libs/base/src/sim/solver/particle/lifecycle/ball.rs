use crate::{math::{FloatScalar, Vector}, 
sim::{lifecycle::ElementLifecycle, 
    solver::particle::{environment::{GravityModel, ParticleEnvironment}, 
    flags::{CollisionFlags, FluidCollisionFlags}, lifecycle::Stream, 
    space::{GridSpace, grid_key::GridKey}, state::State, tuning::SimulationTuning, 
    verlet_particle::VerletParticle}, storage::ElementStorage}, ui::layout::color::Color};
use std::{env, hash::Hash};

pub struct BallLifecycle<V: Vector> {
    pub stream: Stream<V>,
}
 

impl<V: Vector + 'static> BallLifecycle<V> 
where
    V::Scalar: FloatScalar + 'static, // Synced to use FloatScalar tools cleanly
     V::Quantized: Hash + Eq + Copy + GridKey,
{ 

    #[inline]
    pub fn new() -> Self {
         let stream = Stream::new(
            20,                                      // start_tick
            1,                                       // ticks_per_spawn 
            V::from_f64_array([0.2, 0.95]),           // relative_position
            V::from_f64_array([3500.0, -900.0]),        // velocity
            V::Scalar::from_f64(10.0),                // radius
            V::Scalar::from_f64(1.0),                // density
        );

        Self { stream }
    }

    pub fn environment() -> ParticleEnvironment<V, FluidCollisionFlags> {
        // --- 1. CLOCK & ITERATION SETTINGS ---
        let substep_count: u64 = 8;
        let collision_iterations: u64 = 2;
        let max_particles: usize = 600;  
         
        // --- 2. PHYSICS SIZING & MATERIAL CONSTRAINTS ---
        let cell_size = V::Scalar::from_f64(0.0);
        let gravity_force = V::from_f64_array([0.0, -2000.0]);

        let space = GridSpace::new(cell_size);
        
        // --- 3. HARDWARE SPEED CONST TUNING CONFIGURATIONS ---
        let tuning = SimulationTuning::new(
            60.0,
            substep_count, 
            collision_iterations, 
            max_particles,
            cell_size, 
            V::Scalar::from_f64(0.6), 
            V::Scalar::from_f64(0.4)
        );
         
        // --- 4. ENGINE RUNTIME VISUALIZATION ASSETS ---
        let state = State::new(&Color::RAINBOW);
        let gravity = GravityModel::Constant(gravity_force); 
        
        ParticleEnvironment::new(space, tuning, state, gravity)
    }
}

impl<St, V, F> ElementLifecycle<St, ParticleEnvironment<V, F>> for BallLifecycle<V>
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
        let len = storage.len(); 

        let burst_count = 1;
        let max = environment.tuning.max_particles - burst_count;
     
        if self.stream.should_emit(tick) && len <= max  {
  
            // Extract base trajectory profiles
            let base_pos = self.stream.get_position(bounds);
            let spawned_vel = self.stream.velocity.clone();
            let radius = self.stream.radius.clone();
            let density = self.stream.density.clone();

            let spawned_pos = base_pos.clone();// + horizontal_shift - vertical_shift;
            let spawned_pos_old = spawned_pos.clone() - (spawned_vel.clone() * v_dt.clone());
          
            let percent: f64 = storage.len() as f64 / environment.tuning.max_particles as f64;
            let color = environment.state.get_color(percent);

            // Construct full structural particle context
            let particle = VerletParticle {
                pos:      base_pos,
                pos_old:  spawned_pos_old,
                acc:      V::ZERO,
                radius:   radius.clone(),
                color,
                inv_mass: V::Scalar::ONE / density.clone(),  
            };
            
            // Direct zero-friction insertion into backing layout arrays
            storage.push(particle);


//         // let mut position2 = position;  
//         // position2.as_slice_mut()[1] -= radius * V::Scalar::from_f64(3.0);

//         // let mut position3 = position; 
//         // position3.as_slice_mut()[1] -= radius * V::Scalar::from_f64(6.0);

//         // let mut position4 = position; 
//         // position4.as_slice_mut()[1] -= radius * V::Scalar::from_f64(9.0);
        
//         // 🟢 FIXED: Clone velocity for the first particle so it remains available for the second
//         let p1 = VerletParticle::new(position)
//             .with_velocity(velocity.clone(), step_dt)
//             .with_radius(radius, density)
//             .with_color(color);

//         // let p2 = VerletParticle::new(position2)
//         //     .with_velocity(velocity, step_dt)
//         //     .with_radius(radius, density)
//         //     .with_color(color);

//         //  let p3 = VerletParticle::new(position3)
//         //     .with_velocity(velocity, step_dt)
//         //     .with_radius(radius, density)
//         //     .with_color(color);

//         // let p4 = VerletParticle::new(position4)
//         //     .with_velocity(velocity, step_dt)
//         //     .with_radius(radius, density)
//         //     .with_color(color);

//         storage.push(p1);
//         // storage.push(p2);
//         // storage.push(p3);
//         // storage.push(p4);
//         config.particle_count = storage.len();


            // let burst_count = self.stream.droplets_per_burst;
            
            // // Core structural spacing parameters
            // let spacing_factor = V::Scalar::from_f64(2.2);
            // let fat_diameter = self.stream.radius.clone() * spacing_factor;
            
            // // Extract base trajectory profiles
            // let base_pos = self.stream.get_position(bounds);
            // let spawned_vel = self.stream.velocity.clone();
            // let radius = self.stream.radius.clone();
            // let density = self.stream.density.clone();

            // // 🚀 1. DYNAMIC PERPENDICULAR NOZZLE AXIS DERIVATION
            // // Normalize the velocity vector to extract the pure travel heading
            // let stream_heading = spawned_vel.clone().normalize();
            
            // // Compute the side vector perpendicular to the velocity field
            // // (Utilizes your native Vector trait's perpendicular/cross-product utilities)
            // let width_axis = stream_heading.perpendicular_vector(); 

            // for i in 0..burst_count {
            //     // Calculate row and column indices (3 columns wide)
            //     let col = (i % 3) as f64; // 0.0 (Left), 1.0 (Center), 2.0 (Right)
            //     let row = (i / 3) as f64; // 0.0, 1.0, 2.0... (Height layers)

            //     // Maps col to [-1.0, 0.0, 1.0] to center the nozzle alignment nicely
            //     let width_offset_scalar = V::Scalar::from_f64(col - 1.0);
                
            //     // Height offset stacks each row backward in time along the flow vector
            //     let height_offset_scalar = V::Scalar::from_f64(row);

            //     // Map colors linearly based on stream height layers
            //     let total_rows = f64::max(1.0, (burst_count as f64 / 3.0).ceil() - 1.0);
            //     let burst_percent = if total_rows > 0.0 { row / total_rows } else { 0.0 };
            //     let color = environment.state.get_color(burst_percent);

            //     // 🚀 2. CALCULATE ORIENTED SPATIAL SHIFTS
            //     // Offset horizontally along the dynamically derived perpendicular nozzle width vector
            //     let horizontal_shift = width_axis.clone() * (fat_diameter.clone() * width_offset_scalar);
                
            //     // Offset vertically backwards along the stream travel heading
            //     let vertical_shift = stream_heading.clone() * (fat_diameter.clone() * height_offset_scalar);

            //     // Combine base position with oriented injection grid adjustments
            //     let spawned_pos = base_pos.clone() + horizontal_shift - vertical_shift;

            //     // Compute historical Verlet position: pos_old = pos - vel * dt
            //     let spawned_pos_old = spawned_pos.clone() - (spawned_vel.clone() * v_dt.clone());

            //     // Construct full structural particle context
            //     let particle = VerletParticle {
            //         pos:      spawned_pos,
            //         pos_old:  spawned_pos_old,
            //         acc:      V::ZERO,
            //         radius:   radius.clone(),
            //         color,
            //         inv_mass: V::Scalar::ONE / density.clone(),  
            //     };
                
            //     // Direct zero-friction insertion into backing layout arrays
            //     storage.push(particle);
            // }
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
