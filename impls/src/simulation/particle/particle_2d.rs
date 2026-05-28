/// A 2D particle for use with VecStorage<Particle2D>.
/// Stores position, previous position (Verlet), physical properties, and appearance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle2D {
    /// Current position in world space.
    pub x:      f32,
    pub y:      f32,

    /// Previous position — used by Verlet integration.
    pub px:     f32,
    pub py:     f32,

    /// Radius in world units.
    pub radius: f32,

    /// Mass — use 0.0 for a pinned/static particle.
    pub mass:   f32,

    /// RGBA colour for rendering.
    pub color:  [u8; 4],
}

impl Particle2D {
    /// Creates a stationary particle at (x, y).
    pub fn new(x: f32, y: f32, radius: f32, mass: f32, color: [u8; 4]) -> Self {
        Self { x, y, px: x, py: y, radius, mass, color }
    }

    /// Creates a particle with an initial velocity by offsetting previous position.
    pub fn with_velocity(mut self, vx: f32, vy: f32, dt: f32) -> Self {
        self.px = self.x - vx * dt;
        self.py = self.y - vy * dt;
        self
    }

    /// Returns true if this particle is pinned (zero mass).
    pub fn is_static(&self) -> bool { self.mass == 0.0 }

    /// Velocity estimate from Verlet positions and dt.
    pub fn velocity(&self, dt: f32) -> [f32; 2] {
        [(self.x - self.px) / dt, (self.y - self.py) / dt]
    }
}

impl Default for Particle2D {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0, [255, 255, 255, 255])
    }
}

// use crate::simulation::particle::particle_2d::Particle2D;
// use crate::simulation::storage::aos_vec::AosVecStorage;
// use crate::simulation::storage::soa_vec::SoaVecStorage;
// use crate::simulation::particle::col;

// // AoS — tests both base Storage and AosStorage contracts
// base::test_storage!(AosVecStorage<Particle2D>, Particle2D);
// base::test_aos_storage!(AosVecStorage<Particle2D>, Particle2D);

// // SoA — tests base Storage and SoaStorage contracts
// base::test_storage!(SoaVecStorage<Particle2D>, Particle2D);
// base::test_soa_storage!(SoaVecStorage<Particle2D>, Particle2D, col::X, f32);