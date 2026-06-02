use base::sim::storage::{
    SoaLayout,
    aos_vec::AosVecStorage,
    soa_vec::SoaVecStorage,
    verlet::aos::AosVerletItem,
    verlet::soa::SoaVerletLayout,
};

 
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerletParticle2d {
    pub pos:     [f64; 2],
    pub pos_old: [f64; 2],
    pub acc:     [f64; 2],
}

impl VerletParticle2d { 
    pub fn new(x: f64, y: f64) -> Self {
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

impl Default for VerletParticle2d {
    fn default() -> Self { Self::new(0.0, 0.0) }
}

impl AosVerletItem<2> for VerletParticle2d {
    fn acc_mut(&mut self) -> &mut [f64; 2] { &mut self.acc }

    fn pos_pos_old_mut_acc(&mut self) -> (&mut [f64; 2], &mut [f64; 2], &[f64; 2]) {
        (&mut self.pos, &mut self.pos_old, &self.acc)
    }
}

 
pub type AosStorage2d = AosVecStorage<VerletParticle2d>;

 
impl SoaLayout for VerletParticle2d {
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

 
impl SoaVerletLayout for VerletParticle2d {
    const N: usize = 2;
}

/// Convenience alias — `SoaVecStorage<VerletParticle2d>` automatically implements
/// `SoaVerletStorage` via the blanket impl on `SoaVerletLayout`.
pub type SoaStorage2d = SoaVecStorage<VerletParticle2d>;
 
#[cfg(test)]
mod tests {
    use super::*;
    use base::sim::storage::{Storage, verlet::soa::SoaVerletStorage};

    mod aos { use super::*; base::test_storage!(AosStorage2d, VerletParticle2d); }
    mod soa { use super::*; base::test_storage!(SoaStorage2d, VerletParticle2d); }
    base::test_soa_verlet_storage!(SoaStorage2d, VerletParticle2d, 2);
    base::test_aos_verlet_storage!(AosStorage2d, VerletParticle2d, 2);

    #[test]
    fn aos_at_sets_pos_and_rest() {
        let p = VerletParticle2d::new(3.0, 4.0);
        assert_eq!(p.pos, [3.0, 4.0]);
        assert_eq!(p.pos_old, [3.0, 4.0]);
    }

    #[test]
    fn aos_with_velocity_encodes_offset() {
        let dt = 0.1;
        let p = VerletParticle2d::with_velocity(0.0, 0.0, 1.0, 2.0, dt);
        assert!((p.pos_old[0] - (-0.1)).abs() < 1e-12);
        assert!((p.pos_old[1] - (-0.2)).abs() < 1e-12);
    }

    #[test]
    fn soa_column_values_match_pushed_particle() {
        let mut s = SoaStorage2d::new(4);
        s.push(VerletParticle2d::new(1.0, 2.0));
        assert_eq!(s.pos_col(0)[0], 1.0);
        assert_eq!(s.pos_col(1)[0], 2.0);
        assert_eq!(s.pos_old_col(0)[0], 1.0);
        assert_eq!(s.pos_old_col(1)[0], 2.0);
        assert_eq!(s.acc_col(0)[0], 0.0);
        assert_eq!(s.acc_col(1)[0], 0.0);
    }
} 