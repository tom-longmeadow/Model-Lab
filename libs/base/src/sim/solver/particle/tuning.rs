
use crate::math::{FloatScalar, Vector};
 
pub struct SimulationTuning<V> 
where 
    V: Vector 
{
    pub substep_count: u64,
    pub collision_iterations: u64, 
    pub physics: PhysicsTuning<V::Scalar>, 
}

impl<V: Vector> SimulationTuning<V> {
    pub fn new(substep_count: u64, collision_iterations: u64, tuning_size: V::Scalar) -> Self {
        Self {
            substep_count,  
            collision_iterations, 
            physics: PhysicsTuning::<V::Scalar>::new(tuning_size, tuning_size, collision_iterations)
        }
    }

    pub fn update_physics(&mut self, min_size: V::Scalar, max_size: V::Scalar,){
        self.physics = PhysicsTuning::new(min_size, max_size, self.collision_iterations);
    }
}
 
#[derive(Clone, Copy, Debug, PartialEq)] 
pub struct PhysicsTuning<S: FloatScalar> {
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
}

impl<S: FloatScalar> PhysicsTuning<S> { 
    pub fn new(size_min: S, size_max: S, collision_iterations: u64) -> Self {
        // Fallback or baseline restitution (0.5)
        let default_restitution = S::from_f64(0.75); 
        
        Self::with_all(
            size_min, 
            size_max, 
            default_restitution,
            collision_iterations,
        )
    }

    pub fn with_all(
        size_min: S, 
        size_max: S,
        restitution: S,
        collision_iterations: u64, 
    ) -> Self {  
        // Convert configurations and constants using your trait conversion
        let iterations_f = S::from_f64(collision_iterations as f64); 
        let slop_coefficient = S::from_f64(0.02);   
        let bounce_threshold_factor = S::from_f64(2.0);  
        let global_damping_constant = S::from_f64(0.1);
        let friction_constant = S::from_f64(0.1);
        let target_frame_bias = S::from_f64(0.4);  
        let max_velocity = size_max * S::from_f64(600.0);   

        Self {
            restitution,
            velocity_bounce_threshold: size_min * bounce_threshold_factor,
            penetration_slop: size_min * slop_coefficient, 
            penetration_correction_bias: target_frame_bias / iterations_f, 
            global_damping: global_damping_constant,
            friction: friction_constant, 
            max_velocity, 
        }
    } 
}

impl<S: FloatScalar> Default for PhysicsTuning<S> {
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


    