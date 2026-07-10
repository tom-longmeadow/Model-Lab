use base::math::DVec2;


#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub pos:     DVec2,
    pub pos_old: DVec2,
    pub acc:     DVec2,
    pub radius: f64,
}

impl Particle {
    pub fn new(x: f64, y: f64, radius: f64) -> Self {
        Self {
            pos:     DVec2::new(x, y),
            pos_old: DVec2::new(x, y),
            acc:     DVec2::ZERO,
            radius,
        }
    }
}

impl Default for Particle {
    fn default() -> Self { Self::new(0.0, 0.0, 1.0) }
}