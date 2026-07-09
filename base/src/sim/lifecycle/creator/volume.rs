use crate::sim::lifecycle::Creator;

/// Maintains a target count derived from `volume * fill_ratio / particle_volume`.
pub struct Volume {
    volume_fn:      Box<dyn Fn() -> f64>,
    fill_ratio:     f64,
    particle_volume: f64,
}

impl Volume {
    pub fn new(volume_fn: impl Fn() -> f64 + 'static, fill_ratio: f64, particle_volume: f64) -> Self {
        Self { volume_fn: Box::new(volume_fn), fill_ratio, particle_volume }
    }

    fn target(&self) -> usize {
        (((self.volume_fn)() * self.fill_ratio) / self.particle_volume).floor() as usize
    }
}

impl Creator for Volume {
    fn deficit(&self, current_len: usize) -> usize {
        self.target().saturating_sub(current_len)
    }
    fn excess(&self, current_len: usize) -> usize {
        current_len.saturating_sub(self.target())
    }
}
