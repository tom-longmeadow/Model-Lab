
use crate::math::FloatScalar;

#[derive(Clone, Copy, Debug)]
pub struct ParticlePhysicsTuning<S: FloatScalar> {
    /// Bounciness factor: 0.0 means a dead stop, 1.0 means an ideal elastic bounce.
    pub restitution: S,               
    /// Minimum speed required to trigger a bounce
    pub velocity_bounce_threshold: S, 
    /// Allowed overlap before correction
    pub penetration_slop: S,          
    /// Baumgarte stabilization factor
    pub penetration_correction_bias: S, 
    /// Frame-rate independent linear drag
    pub global_damping: S,           
    /// Surface friction
    pub friction: S,  
    /// Velocity cap to prevent tunneling
    pub max_velocity: S,     
    // Add a pre-mixed vector state that your engine updates once per frame
    pub runtime_jitter: [f64; 4], // Large enough array to cover any vector dimension          
}

impl<S: FloatScalar> ParticlePhysicsTuning<S> { 
    pub fn new(radius_min: S, radius_max: S, collision_iterations: u64) -> Self {
        // Fallback or baseline restitution (0.5)
        let default_restitution = S::from_f64(0.5); 
        
        Self::with_all(
            radius_min, 
            radius_max, 
            default_restitution,
            collision_iterations,
        )
    }

    pub fn with_all(
        radius_min: S, 
        radius_max: S,
        restitution: S,
        collision_iterations: u64, 
    ) -> Self {  
        // Convert configurations and constants using your trait conversion
        let iterations_f = S::from_f64(collision_iterations as f64); 
        let slop_coefficient = S::from_f64(0.02);   
        let bounce_threshold_factor = S::from_f64(2.0);  
        let global_damping_constant = S::from_f64(0.2);
        let friction_constant = S::from_f64(0.3);
        let target_frame_bias = S::from_f64(0.4);  
        let max_velocity = radius_max * S::from_f64(600.0);  
        let runtime_jitter: [f64; 4] = [0.0; 4];

        Self {
            restitution,
            velocity_bounce_threshold: radius_min * bounce_threshold_factor,
            penetration_slop: radius_min * slop_coefficient, 
            penetration_correction_bias: target_frame_bias / iterations_f, 
            global_damping: global_damping_constant,
            friction: friction_constant, 
            max_velocity,
            runtime_jitter
        }
    }

    /// Call this once per frame before processing particle constraints.
    /// It uses a golden ratio multiplier to cycle the seed chaotically.
    pub fn update_jitter(&mut self, frame_count: u64) {
        // High-frequency constants to break up numerical alignment
        let seed = frame_count.wrapping_add(0x9E3779B97F4A7C15);
        
        // Generate 4 highly unaligned pseudo-random values
        for i in 0..4 {
            let mut x = seed.wrapping_add(i as u64).wrapping_mul(0xBF58476D1CE4E5B9);
            x = (x ^ (x >> 30)).wrapping_mul(0x94D049BB133111EB);
            x = (x ^ (x >> 27)).wrapping_mul(0x7305754198654329);
            let raw_float = (x ^ (x >> 31)) as f64 / u64::MAX as f64; // Maps to [0.0, 1.0]
             
            self.runtime_jitter[i] = (raw_float * 0.02) - 0.01;
        }
    }

}

impl<S: FloatScalar> Default for ParticlePhysicsTuning<S> {
    fn default() -> Self {
        let one = S::from_f64(1.0);
        Self::new(one, one,1)
    }
}

// #[derive(Clone, Copy, Debug)]
// pub struct ParticlePhysicsTuning {

//     pub restitution: f64,               // Bounciness factor: 0.0 means a dead stop (like clay), 1.0 means an ideal elastic bounce losing no energy.
//     pub velocity_bounce_threshold: f64, // Minimum speed required to trigger a bounce
//     pub penetration_slop: f64,          // Allowed overlap before correction
//     pub penetration_correction_bias: f64, // Baumgarte stabilization factor: Resolves 20% of positional penetration per frame to prevent aggressive explosions.
//     pub global_damping: f64,           // Frame-rate independent linear drag: Multiplied against velocity each frame to prevent floating-point energy gain.
//     pub friction: f64,                 // Surface friction: Determines how quickly a particle slows down when sliding against a wall or another particle.
// }

// impl ParticlePhysicsTuning { 

//      pub fn new(radius_min: f64, collision_iterations: u64) -> Self {
//         Self::with_all(
//             radius_min, 
//             RESTITUTION,
//             collision_iterations,
//         )
//     }


//     pub fn with_all(
//         radius_min: f64, 
//         restitution: f64,
//         collision_iterations: u64, 
//     ) -> Self {  

//         let iterations_f = collision_iterations as f64;
        
//         // Target resolving 20% of penetration per complete frame step.
//         // Adjust this base value (0.2) to make the simulation softer (0.1) or stiffer (0.4).
//         let target_frame_bias = 0.2; 

//         Self {
//             restitution,
//             velocity_bounce_threshold: radius_min * BOUNCE_THRESHOLD_FACTOR,
//             penetration_slop: radius_min * SLOP_COEFFICIENT, 
//             // Scaled properly so the entire pass approaches the target_frame_bias smoothly
//             penetration_correction_bias: target_frame_bias / iterations_f, 
//             global_damping: GLOBAL_DAMPING,
//             friction: FRICTION, 
//         }
//     }

 
 
// }
 
//  impl Default for ParticlePhysicsTuning{
//     fn default() -> Self {
//         Self::new(1.0, 1)
//     }
//  }


    