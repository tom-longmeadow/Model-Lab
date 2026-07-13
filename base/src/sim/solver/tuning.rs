
const SLOP_COEFFICIENT: f64 = 0.02;         // 2% of the reference radius
const BOUNCE_THRESHOLD_FACTOR: f64 = 0.05;   // 5% of reference radius per second
const MAX_VELOCITY_MULTIPLIER: f64 = 50.0;  // Can travel 50 diameters per second

const RESTITUTION: f64 = 0.5;  
const FRICTION: f64 = 0.3; 
const POSITION_CORRECTION_BIAS: f64 = 0.2; 
const GLOBAL_DAMPING: f64 = 0.12; // Example physical drag coefficient



#[derive(Clone, Copy, Debug)]
pub struct ParticlePhysicsTuning {

    pub restitution: f64,               // Bounciness factor: 0.0 means a dead stop (like clay), 1.0 means an ideal elastic bounce losing no energy.
    pub velocity_bounce_threshold: f64, // Minimum speed required to trigger a bounce
    pub penetration_slop: f64,          // Allowed overlap before correction
    pub penetration_correction_bias: f64, // Baumgarte stabilization factor: Resolves 20% of positional penetration per frame to prevent aggressive explosions.
    pub global_damping: f64,           // Frame-rate independent linear drag: Multiplied against velocity each frame to prevent floating-point energy gain.
    pub friction: f64,                 // Surface friction: Determines how quickly a particle slows down when sliding against a wall or another particle.
    
    // 

    
    
    // pub max_velocity: f64,              // Speed cap to prevent particles escaping the world
}

impl ParticlePhysicsTuning { 

     pub fn new(radius_min: f64, radius_max: f64, collision_iterations: u64) -> Self {
        Self::with_all(
            radius_min,
            radius_max,
            RESTITUTION,
            collision_iterations,
        )
    }


    pub fn with_all(
        radius_min: f64,
        radius_max: f64,
        restitution: f64,
        collision_iterations: u64, 
    ) -> Self {  
        let iterations_f = collision_iterations as f64;
        
        // Target resolving 20% of penetration per complete frame step.
        // Adjust this base value (0.2) to make the simulation softer (0.1) or stiffer (0.4).
        let target_frame_bias = 0.2; 

        Self {
            restitution,
            velocity_bounce_threshold: radius_min * BOUNCE_THRESHOLD_FACTOR,
            penetration_slop: radius_min * SLOP_COEFFICIENT, 
            // Scaled properly so the entire pass approaches the target_frame_bias smoothly
            penetration_correction_bias: target_frame_bias / iterations_f, 
            global_damping: GLOBAL_DAMPING,
            friction: FRICTION
        }
    }

 
 
}
 
 impl Default for ParticlePhysicsTuning{
    fn default() -> Self {
        Self::new(0.0, 0.0, 1)
    }
 }


    