 
pub trait CollisionFlags {
    const JITTER: bool = false;
    const RESTITUTION: bool = false;
    const USE_BIAS: bool = false;
    const USE_SLOP: bool = false;
}

pub struct AllCollisionFlags;
impl CollisionFlags for AllCollisionFlags {
    const JITTER: bool = true;
    const RESTITUTION: bool = true;  
    const USE_BIAS: bool = true;
    const USE_SLOP: bool = true;
}

pub struct NoneCollisionFLags;
impl CollisionFlags for NoneCollisionFLags {
    const JITTER: bool = false;
    const RESTITUTION: bool = false;  
    const USE_BIAS: bool = false;
    const USE_SLOP: bool = false;
}

// Configuration for your SPH/Water solver
pub struct FluidCollisionFlags;
impl CollisionFlags for FluidCollisionFlags {
    const JITTER: bool = true; // Helps resolve particle packing overlap
    const RESTITUTION: bool = false; // Fluids absorb energy, don't bounce like rigid bodies
    const USE_BIAS: bool = false;
    const USE_SLOP: bool = true;
}

pub struct RigidCollisionFlags;  
impl CollisionFlags for RigidCollisionFlags {
    const JITTER: bool = false;
    const RESTITUTION: bool = true;  
    const USE_BIAS: bool = true;
    const USE_SLOP: bool = false;
}
 
 
 


