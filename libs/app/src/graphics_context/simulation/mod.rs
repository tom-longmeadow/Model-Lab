pub mod aos;
pub mod soa;
pub mod pass;
pub mod renderer;

// Re-export ParticleInstance for use by applications
//pub use aos::ParticleInstance;

/// Transform from simulation space to NDC (Normalized Device Coordinates).
/// NDC ranges from [-1, 1] in all axes.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub offset: [f64; 3],
    pub scale: [f64; 3],
}

impl Transform {
    /// Create a transform that maps simulation bounds to NDC bounds.
    /// For 2D simulations, z components are typically (0.0, 1.0).
    pub fn from_bounds(
        sim_x_min: f64, sim_x_max: f64,
        sim_y_min: f64, sim_y_max: f64,
        ndc_x_min: f64, ndc_x_max: f64,
        ndc_y_min: f64, ndc_y_max: f64,
    ) -> Self {
        let sim_width = sim_x_max - sim_x_min;
        let sim_height = sim_y_max - sim_y_min;
        let ndc_width = ndc_x_max - ndc_x_min;
        let ndc_height = ndc_y_max - ndc_y_min;

        let scale_x = ndc_width / sim_width;
        let scale_y = ndc_height / sim_height;

        let offset_x = ndc_x_min - sim_x_min * scale_x;
        let offset_y = ndc_y_min - sim_y_min * scale_y;

        Self {
            offset: [offset_x, offset_y, 0.0],
            scale: [scale_x, scale_y, 1.0],
        }
    }

    /// Transform a position from simulation space to NDC.
    #[inline]
    pub fn sim_to_ndc(&self, x: f64, y: f64, z: f64) -> [f64; 3] {
        [
            x * self.scale[0] + self.offset[0],
            y * self.scale[1] + self.offset[1],
            z * self.scale[2] + self.offset[2],
        ]
    }

    /// Identity transform (no-op).
    pub fn identity() -> Self {
        Self {
            offset: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}