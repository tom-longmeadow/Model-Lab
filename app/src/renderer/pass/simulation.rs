use base::sim::simulation::Simulate;

// This struct is generic over the simulation type S
pub struct SimulationRenderPass<S: Simulate> {
    pub simulation: S,
    // ... wgpu resources
}