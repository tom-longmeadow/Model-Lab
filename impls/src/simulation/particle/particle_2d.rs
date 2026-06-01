use base::sim::storage::{
    SoaLayout,
    aos_vec::AosVecStorage,
    soa_vec::SoaVecStorage,
    verlet::aos::AosVerletItem,
    verlet::soa::SoaVerletLayout,
};

// ---------------------------------------------------------------------------
// AoS — one struct per particle, contiguous in memory
// ---------------------------------------------------------------------------

/// 2D Verlet particle for [`AosVecStorage`].
/// Fields are public — the renderer and creator access them directly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AosParticle2d {
    pub pos:     [f64; 2],
    pub pos_old: [f64; 2],
    pub acc:     [f64; 2],
}

impl AosParticle2d {
    /// Stationary particle at `(x, y)`.
    pub fn at(x: f64, y: f64) -> Self {
        Self { pos: [x, y], pos_old: [x, y], acc: [0.0; 2] }
    }

    /// Particle at `(x, y)` with an initial velocity encoded as a Verlet offset.
    /// `pos_old = pos - vel * dt`.
    pub fn with_velocity(x: f64, y: f64, vx: f64, vy: f64, dt: f64) -> Self {
        Self {
            pos:     [x, y],
            pos_old: [x - vx * dt, y - vy * dt],
            acc:     [0.0; 2],
        }
    }
}

impl Default for AosParticle2d {
    fn default() -> Self { Self::at(0.0, 0.0) }
}

impl AosVerletItem<2> for AosParticle2d {
    fn acc_mut(&mut self) -> &mut [f64; 2] { &mut self.acc }

    fn pos_pos_old_mut_acc(&mut self) -> (&mut [f64; 2], &mut [f64; 2], &[f64; 2]) {
        (&mut self.pos, &mut self.pos_old, &self.acc)
    }
}

/// Convenience alias.
pub type AosStorage2d = AosVecStorage<AosParticle2d>;

// ---------------------------------------------------------------------------
// SoA — six f64 columns: [pos_x, pos_y, pos_old_x, pos_old_y, acc_x, acc_y]
// ---------------------------------------------------------------------------

/// Layout type for `SoaVecStorage<SoaParticle2d>`.
/// Columns: pos_x | pos_y | pos_old_x | pos_old_y | acc_x | acc_y
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SoaParticle2d {
    pub pos:     [f64; 2],
    pub pos_old: [f64; 2],
    pub acc:     [f64; 2],
}

impl SoaParticle2d {
    pub fn at(x: f64, y: f64) -> Self {
        Self { pos: [x, y], pos_old: [x, y], acc: [0.0; 2] }
    }

    pub fn with_velocity(x: f64, y: f64, vx: f64, vy: f64, dt: f64) -> Self {
        Self {
            pos:     [x, y],
            pos_old: [x - vx * dt, y - vy * dt],
            acc:     [0.0; 2],
        }
    }
}

impl SoaLayout for SoaParticle2d {
    // 6 columns, each a single f64 (8 bytes)
    const STRIDES: &'static [usize] = &[8, 8, 8, 8, 8, 8];

    fn push_cols(&self, cols: &mut [Vec<u8>]) {
        cols[0].extend_from_slice(&self.pos[0].to_ne_bytes());
        cols[1].extend_from_slice(&self.pos[1].to_ne_bytes());
        cols[2].extend_from_slice(&self.pos_old[0].to_ne_bytes());
        cols[3].extend_from_slice(&self.pos_old[1].to_ne_bytes());
        cols[4].extend_from_slice(&self.acc[0].to_ne_bytes());
        cols[5].extend_from_slice(&self.acc[1].to_ne_bytes());
    }

    fn read_cols(cols: &[Vec<u8>], i: usize) -> Self {
        let f = |col: &[u8], i: usize| {
            f64::from_ne_bytes(col[i * 8..][..8].try_into().unwrap())
        };
        Self {
            pos:     [f(&cols[0], i), f(&cols[1], i)],
            pos_old: [f(&cols[2], i), f(&cols[3], i)],
            acc:     [f(&cols[4], i), f(&cols[5], i)],
        }
    }

    fn swap_remove_cols(cols: &mut [Vec<u8>], strides: &[usize], index: usize) {
        for (col, &stride) in cols.iter_mut().zip(strides.iter()) {
            let last_start = col.len() - stride;
            let tgt_start  = index * stride;
            col.copy_within(last_start.., tgt_start);
            col.truncate(last_start);
        }
    }
}

/// Column order matches `SoaVerletLayout`: pos (2) | pos_old (2) | acc (2).
impl SoaVerletLayout for SoaParticle2d {
    const N: usize = 2;
}

/// Convenience alias — `SoaVecStorage<SoaParticle2d>` automatically implements
/// `SoaVerletStorage` via the blanket impl on `SoaVerletLayout`.
pub type SoaStorage2d = SoaVecStorage<SoaParticle2d>;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use base::sim::storage::{Storage, verlet::soa::SoaVerletStorage};

    mod aos { use super::*; base::test_storage!(AosStorage2d, AosParticle2d); }
    mod soa { use super::*; base::test_storage!(SoaStorage2d, SoaParticle2d); }
    base::test_soa_verlet_storage!(SoaStorage2d, SoaParticle2d, 2);
    base::test_aos_verlet_storage!(AosStorage2d, AosParticle2d, 2);

    #[test]
    fn aos_at_sets_pos_and_rest() {
        let p = AosParticle2d::at(3.0, 4.0);
        assert_eq!(p.pos, [3.0, 4.0]);
        assert_eq!(p.pos_old, [3.0, 4.0]);
    }

    #[test]
    fn aos_with_velocity_encodes_offset() {
        let dt = 0.1;
        let p = AosParticle2d::with_velocity(0.0, 0.0, 1.0, 2.0, dt);
        assert!((p.pos_old[0] - (-0.1)).abs() < 1e-12);
        assert!((p.pos_old[1] - (-0.2)).abs() < 1e-12);
    }

    #[test]
    fn soa_column_values_match_pushed_particle() {
        let mut s = SoaStorage2d::new(4);
        s.push(SoaParticle2d::at(1.0, 2.0));
        assert_eq!(s.pos_col(0)[0], 1.0);
        assert_eq!(s.pos_col(1)[0], 2.0);
        assert_eq!(s.pos_old_col(0)[0], 1.0);
        assert_eq!(s.pos_old_col(1)[0], 2.0);
        assert_eq!(s.acc_col(0)[0], 0.0);
        assert_eq!(s.acc_col(1)[0], 0.0);
    }
}
// /// Stores position, previous position (Verlet), physical properties, and appearance.
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub struct Particle2D {
//     /// Current position in world space.
//     pub x:      f32,
//     pub y:      f32,

//     /// Previous position — used by Verlet integration.
//     pub px:     f32,
//     pub py:     f32,

//     /// Radius in world units.
//     pub radius: f32,

//     /// Mass — use 0.0 for a pinned/static particle.
//     pub mass:   f32,

//     /// RGBA colour for rendering.
//     pub color:  [u8; 4],
// }

// impl Particle2D {
//     /// Creates a stationary particle at (x, y).
//     pub fn new(x: f32, y: f32, radius: f32, mass: f32, color: [u8; 4]) -> Self {
//         Self { x, y, px: x, py: y, radius, mass, color }
//     }

//     /// Creates a particle with an initial velocity by offsetting previous position.
//     pub fn with_velocity(mut self, vx: f32, vy: f32, dt: f32) -> Self {
//         self.px = self.x - vx * dt;
//         self.py = self.y - vy * dt;
//         self
//     }

//     /// Returns true if this particle is pinned (zero mass).
//     pub fn is_static(&self) -> bool { self.mass == 0.0 }

//     /// Velocity estimate from Verlet positions and dt.
//     pub fn velocity(&self, dt: f32) -> [f32; 2] {
//         [(self.x - self.px) / dt, (self.y - self.py) / dt]
//     }
// }

// impl Default for Particle2D {
//     fn default() -> Self {
//         Self::new(0.0, 0.0, 1.0, 1.0, [255, 255, 255, 255])
//     }
// }

// // use crate::simulation::particle::particle_2d::Particle2D;
// // use crate::simulation::storage::aos_vec::AosVecStorage;
// // use crate::simulation::storage::soa_vec::SoaVecStorage;
// // use crate::simulation::particle::col;

// // // AoS — tests both base Storage and AosStorage contracts
// // base::test_storage!(AosVecStorage<Particle2D>, Particle2D);
// // base::test_aos_storage!(AosVecStorage<Particle2D>, Particle2D);

// // // SoA — tests base Storage and SoaStorage contracts
// // base::test_storage!(SoaVecStorage<Particle2D>, Particle2D);
// // base::test_soa_storage!(SoaVecStorage<Particle2D>, Particle2D, col::X, f32);