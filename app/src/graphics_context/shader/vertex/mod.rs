#[derive(Clone, Copy)]
pub enum VertexFunction {
    ParticleAosColor,
    ParticleAosRadiusColor,
    ParticleAosLit,
    ParticleAosRadiusLit,
    ParticleSoaColor,
    ParticleSoaRadiusColor,
    ParticleSoaLit,
    ParticleSoaRadiusLit,
    Ui,
}

impl VertexFunction {
    pub fn source(&self) -> &'static str {
        match self {
            Self::ParticleAosColor        => include_str!("particle_aos_color.wgsl"),
            Self::ParticleAosRadiusColor  => include_str!("particle_aos_radius_color.wgsl"),
            Self::ParticleAosLit          => include_str!("particle_aos_lit.wgsl"),
            Self::ParticleAosRadiusLit    => include_str!("particle_aos_radius_lit.wgsl"),
            Self::ParticleSoaColor        => include_str!("particle_soa_color.wgsl"),
            Self::ParticleSoaRadiusColor  => include_str!("particle_soa_radius_color.wgsl"),
            Self::ParticleSoaLit          => include_str!("particle_soa_lit.wgsl"),
            Self::ParticleSoaRadiusLit    => include_str!("particle_soa_radius_lit.wgsl"),
            Self::Ui                      => include_str!("ui.wgsl"),
        }
    }
}