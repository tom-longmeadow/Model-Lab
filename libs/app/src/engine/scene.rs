 pub mod particle_aos_verlet;

use crate::{
    engine::input::InputState, 
    graphics_context::{GraphicsContext}
};

pub trait Scene {
    // /// Update scene logic based on input.
    // fn update(&mut self, frame_time: f64, input: &InputState);

    /// Create the render passes needed for this scene.
    /// This is typically called only once at setup.
    fn build_passes(&mut self, renderer: &mut GraphicsContext);

    /// Update the render passes with the current scene state.
    /// This is called every frame before rendering.
    fn update(&mut self, frame_time: f64, input: &InputState, renderer: &mut GraphicsContext);
}

