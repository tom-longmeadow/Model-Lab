
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
    pub fn new(substep_count: u64, collision_iterations: u64, tuning_size: V::Scalar, restitution: V::Scalar) -> Self {
        Self {
            substep_count,  
            collision_iterations, 
            physics: PhysicsTuning::<V::Scalar>::new(tuning_size, tuning_size, restitution)
        }
    }

    pub fn update_physics(&mut self, min_size: V::Scalar, max_size: V::Scalar,){
        self.physics.update(min_size, max_size);
    }
}
 

 #[derive(Clone, Copy, Debug, PartialEq)] 
pub struct PhysicsTuning<S: FloatScalar> {
    pub restitution: S,               
    pub velocity_bounce_threshold: S, 
    pub penetration_slop: S,          
    pub penetration_correction_bias: S, 
    pub global_damping: S,           
    pub friction: S,  
    pub max_velocity: S,        
}
 
impl<S: FloatScalar> PhysicsTuning<S> {
    
    pub fn new(min_size: S, max_size: S, restitution: S) -> Self {
        Self { 
            restitution,
            velocity_bounce_threshold: min_size * S::from_f64(2.0),
            penetration_slop: min_size * S::from_f64(0.02), 
            penetration_correction_bias: S::from_f64(0.4),
            global_damping: S::from_f64(0.1),
            friction: S::from_f64(0.2),
             max_velocity: max_size * S::from_f64(10000.0), 
        }
    }
 
    pub fn update(&mut self, min_size: S, max_size: S){
        self.velocity_bounce_threshold = min_size * S::from_f64(2.0);
        self.penetration_slop = min_size * S::from_f64(0.02);
        self.max_velocity = max_size * S::from_f64(10000.0);
    }
     
}
  
impl<S: FloatScalar> Default for PhysicsTuning<S> {
    fn default() -> Self {
        let one = S::from_f64(1.0);
        Self::new(one, one, one)
    }
}


// #[derive(Clone, Copy, Debug, PartialEq)] 
// pub struct PhysicsTuning<S: FloatScalar> {
//     /// Bounciness factor: 0.0 means a dead stop, 1.0 means an ideal elastic bounce.
//     pub restitution: S,               
//     /// Minimum speed required to trigger a bounce
//     pub velocity_bounce_threshold: S, 
//     /// Allowed overlap before correction
//     pub penetration_slop: S,          
//     /// Baumgarte stabilization factor
//     pub penetration_correction_bias: S, 
//     /// Frame-rate independent linear drag
//     pub global_damping: S,           
//     /// Surface friction
//     pub friction: S,  
//     /// Velocity cap to prevent tunneling
//     pub max_velocity: S,        
// }

// impl<S: FloatScalar> PhysicsTuning<S> { 
//     pub fn new(size_min: S, size_max: S) -> Self { 
//         let default_restitution = S::from_f64(0.4); 
        
//         Self::with_all(
//             size_min, 
//             size_max, 
//             default_restitution, 
//         )
//     }

//     pub fn with_all(
//         size_min: S, 
//         size_max: S,
//         restitution: S, 
//     ) -> Self {  
//         let target_frame_bias = S::from_f64(0.4); // Raw continuous target tracking rate

//         Self {
//             restitution,
//             velocity_bounce_threshold: size_min * S::from_f64(2.0),
//             penetration_slop: size_min * S::from_f64(0.02), 
//             // 🟢 FIX: Do not pre-divide by iterations here. Keep it unscaled.
//             penetration_correction_bias: target_frame_bias, 
//             global_damping: S::from_f64(0.1),
//             friction: S::from_f64(0.2), 
//             max_velocity: size_max * S::from_f64(10000.0), 
//         }
//     }
// }

// impl<S: FloatScalar> Default for PhysicsTuning<S> {
//     fn default() -> Self {
//         let one = S::from_f64(1.0);
//         Self::new(one, one)
//     }
// }

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


    