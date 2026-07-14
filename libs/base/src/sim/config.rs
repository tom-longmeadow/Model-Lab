use crate::{math::Bounds, sim::solver::constraint::Insets};

pub struct LoopConfig {
    pub hz: f64,
    pub substep_count: u64,
    pub collision_iterations: u64,
}

pub struct WorldConfig {
    pub bounds: Bounds,
    pub insets: Insets, 
}
 