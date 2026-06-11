use crate::{
    engine::input::InputState, 
    graphics_context::{pass::RenderPass, GraphicsContext}
};

pub trait Scene {
    /// Update scene logic based on input.
    fn update(&mut self, input: &InputState);

    /// Create the render passes needed for this scene.
    /// This is typically called only once at setup.
    fn build_passes(&self, renderer: &mut GraphicsContext);

    /// Update the render passes with the current scene state.
    /// This is called every frame before rendering.
    fn update_passes(&self, renderer: &mut GraphicsContext);
}