use crate::sim::{
    creator::{Creator, bulk_remove, slowest_n},
    storage::Storage,
};

/// A volume-filling particle creator.
///
/// Maintains a target particle count derived from `bounds_volume * fill_ratio / particle_vol`.
/// - **Grow**: calls `spawn()` once per tick until target is reached.
/// - **Shrink**: scores all particles via `score_fn`, selects the lowest-scoring
///   (slowest / most at-rest) indices, then bulk-removes them in one pass.
///
/// `score_fn` is the only AoS/SoA-sensitive piece — supply an appropriate closure:
///
/// ```ignore
/// // AoS newtonian 2D — iterates items
/// VolumeCreator::new(vol_fn, 0.8, 1.0, spawn, |s: &MyAosStorage| {
///     s.iter().map(|p| vel_sq_2d(p.vel[0], p.vel[1])).collect()
/// });
///
/// // SoA newtonian 2D — reads columns directly (cache-hot)
/// VolumeCreator::new(vol_fn, 0.8, 1.0, spawn, |s: &MySoaStorage| {
///     let vx = s.vel_col(0);
///     let vy = s.vel_col(1);
///     (0..s.len()).map(|i| vel_sq_2d(vx[i], vy[i])).collect()
/// });
/// ```
///
/// - `VF` — `Fn() -> f64`        — current bounds volume (e.g. from a shared `Bounds`)
/// - `SF` — `Fn() -> S::Item`    — spawns one new particle
/// - `SC` — `Fn(&S) -> Vec<f64>` — scores all current particles; lower = removed first
pub struct VolumeCreator<S, VF, SF, SC>
where
    S:  Storage,
    VF: Fn() -> f64,
    SF: Fn() -> S::Item,
    SC: Fn(&S) -> Vec<f64>,
{
    volume_fn:    VF,
    fill_ratio:   f64,
    particle_vol: f64,
    spawn:        SF,
    score_fn:     SC,
    _marker:      std::marker::PhantomData<S>,
}

impl<S, VF, SF, SC> VolumeCreator<S, VF, SF, SC>
where
    S:  Storage,
    VF: Fn() -> f64,
    SF: Fn() -> S::Item,
    SC: Fn(&S) -> Vec<f64>,
{
    /// - `volume_fn`    — closure returning current bounds volume
    /// - `fill_ratio`   — target fill fraction, e.g. `0.8`
    /// - `particle_vol` — volume of one particle
    /// - `spawn`        — factory for a new particle
    /// - `score_fn`     — scores all particles; lower score = removed first on shrink
    pub fn new(
        volume_fn:    VF,
        fill_ratio:   f64,
        particle_vol: f64,
        spawn:        SF,
        score_fn:     SC,
    ) -> Self {
        Self {
            volume_fn,
            fill_ratio,
            particle_vol,
            spawn,
            score_fn,
            _marker: std::marker::PhantomData,
        }
    }

    fn target(&self) -> usize {
        (((self.volume_fn)() * self.fill_ratio) / self.particle_vol).floor() as usize
    }
}

impl<S, VF, SF, SC> Creator<S> for VolumeCreator<S, VF, SF, SC>
where
    S:  Storage,
    VF: Fn() -> f64,
    SF: Fn() -> S::Item,
    SC: Fn(&S) -> Vec<f64>,
{
    fn tick(&mut self, storage: &mut S, _tick: u64) {
        let target = self.target();

        if storage.len() < target {
            storage.push((self.spawn)());
            return;
        }

        if storage.len() > target {
            let excess  = storage.len() - target;
            let scores  = (self.score_fn)(storage);
            let indices = slowest_n(&scores, excess);
            bulk_remove(storage, indices);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{
        creator::particle::score::vel_sq_2d,
        storage::{AosStorage, aos_vec::AosVecStorage},
    };

    #[derive(Default, Clone, Copy, PartialEq, Debug)]
    struct MockParticle { vx: f64, vy: f64 }

    type MockStorage = AosVecStorage<MockParticle>;

    fn make_volume_creator(
        volume: f64,
    ) -> VolumeCreator<MockStorage, impl Fn() -> f64, impl Fn() -> MockParticle, impl Fn(&MockStorage) -> Vec<f64>>
    {
        VolumeCreator::new(
            move || volume,
            1.0,
            1.0,
            || MockParticle::default(),
            |s: &MockStorage| s.iter().map(|p| vel_sq_2d(p.vx, p.vy)).collect(),
        )
    }

    #[test]
    fn grows_one_per_tick() {
        let mut s = MockStorage::new(16);
        let mut c = make_volume_creator(3.0);
        c.tick(&mut s, 0); assert_eq!(s.len(), 1);
        c.tick(&mut s, 1); assert_eq!(s.len(), 2);
        c.tick(&mut s, 2); assert_eq!(s.len(), 3);
        c.tick(&mut s, 3); assert_eq!(s.len(), 3); // at target
    }

    #[test]
    fn shrinks_to_target_removing_slowest() {
        let mut s = MockStorage::new(16);
        for v in [1.0f64, 2.0, 3.0, 4.0, 5.0] {
            s.push(MockParticle { vx: v, vy: 0.0 });
        }
        let mut c = make_volume_creator(3.0);
        c.tick(&mut s, 0);
        assert_eq!(s.len(), 3);
        let vxs: Vec<f64> = s.iter().map(|p| p.vx).collect();
        assert!(!vxs.contains(&1.0));
        assert!(!vxs.contains(&2.0));
    }

    #[test]
    fn no_op_at_target() {
        let mut s = MockStorage::new(16);
        s.push(MockParticle { vx: 1.0, vy: 0.0 });
        let mut c = make_volume_creator(1.0);
        c.tick(&mut s, 0);
        assert_eq!(s.len(), 1);
    }
}
