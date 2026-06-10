
// ---------------------------------------------------------------------------
// VolumeCreator
// ---------------------------------------------------------------------------

use crate::sim::lifecycle::Creator;

/// Maintains a target count derived from volume * fill_ratio / particle_vol.
pub struct VolumeCreator {
    volume_fn: Box<dyn Fn() -> f64>,
    fill_ratio: f64,
    particle_vol: f64,
}

impl VolumeCreator {
    pub fn new(volume_fn: impl Fn() -> f64 + 'static, fill_ratio: f64, particle_vol: f64) -> Self {
        Self { volume_fn: Box::new(volume_fn), fill_ratio, particle_vol }
    }

    fn target(&self) -> usize {
        (((self.volume_fn)() * self.fill_ratio) / self.particle_vol).floor() as usize
    }
}

impl Creator for VolumeCreator {
    fn deficit(&self, current_len: usize) -> usize {
        let target = self.target();
        if current_len < target { target - current_len } else { 0 }
    }

    fn excess(&self, current_len: usize) -> usize {
        let target = self.target();
        if current_len > target { current_len - target } else { 0 }
    }
}