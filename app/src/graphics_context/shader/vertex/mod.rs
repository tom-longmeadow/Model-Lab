#[derive(Clone, Copy)]
pub enum VertexFunction {
    ParticleAosColor,
    ParticleSoaColor,
    Ui,
}

impl VertexFunction {
    pub fn source(&self) -> &'static str {
        match self {
            Self::ParticleAosColor => include_str!("particle_aos_color.wgsl"),
            Self::ParticleSoaColor => include_str!("particle_soa_color.wgsl"),
            Self::Ui               => include_str!("ui.wgsl"),
        }
    }
}