pub mod clock; 
pub mod metrics;
pub mod simulation;
pub mod solver;
pub mod storage; 
pub mod lifecycle;

/// Spatial bounds defining a rectangular region in 3D space.
/// For 2D simulations, z bounds are typically ignored.
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl Bounds {
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64, z_min: f64, z_max: f64) -> Self {
        Self { x_min, x_max, y_min, y_max, z_min, z_max }
    }

    pub fn new_2d(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self::new(x_min, x_max, y_min, y_max, 0.0, 0.0)
    }

    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }

    pub fn depth(&self) -> f64 {
        self.z_max - self.z_min
    }
}
 