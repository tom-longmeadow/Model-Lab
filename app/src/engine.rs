use std::sync::Arc;

use winit::{event::WindowEvent, window::Window};
use winit::event_loop::ActiveEventLoop;

pub mod scene;
pub mod input;
pub mod gui;
pub mod gui_builder;

use crate::graphics_context::config::RendererConfig;
use crate::{
    engine::{
        input::InputState, 
        scene::Scene, 
    }, 
    graphics_context::{
        GraphicsContext, 
        error::RendererError
    }
};
pub struct Engine {
    graphics: GraphicsContext,
    scenes:    Box<dyn Scene>,
    input:    InputState,
}

impl Engine {
    
    pub async fn new(
        window: Arc<Window>,
        width:  u32,
        height: u32,
        config: RendererConfig,      
        mut scene:  Box<dyn Scene>,                  
    ) -> Result<Self, RendererError> {
        tracing::info!("Engine initializing {}x{}", width, height);
        let mut renderer = GraphicsContext::new(window, width, height, config).await?;

        scene.build_passes(&mut renderer);
        tracing::info!("Engine ready");
        Ok(Self {
            graphics: renderer,
            scenes: scene,
            input: InputState::new(),
        })
    }

    pub fn handle_event(&mut self, event: &WindowEvent, event_loop: &ActiveEventLoop) {
        self.input.handle(event);
        match event {
            WindowEvent::CloseRequested  => {
                tracing::info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size)   => {
                tracing::info!("Resize {}x{}", size.width, size.height);
                self.graphics.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => self.render(event_loop),
            _                            => {}
        }
    }
 

     fn render(&mut self, event_loop: &ActiveEventLoop) {
        tracing::trace!("Rendering...");

        // Begin frame, update scene logic
        self.input.begin_frame();
        self.scenes.update(&self.input); 

        // update the state of the render passes
        self.scenes.update_passes(&mut self.graphics);

        // Renderer performs the render  
        match self.graphics.render() {
            Ok(())                          => {}
            Err(RendererError::Outdated) |
            Err(RendererError::Lost)        => {
                tracing::warn!("Surface lost/outdated — reconfiguring");
                self.graphics.reconfigure();
            }
            Err(RendererError::Occluded) |
            Err(RendererError::Timeout)     => {
                tracing::trace!("Frame skipped: occluded or timeout");
            }
            Err(e)                          => {
                tracing::error!("Render failed: {}", e);
                event_loop.exit();
            }
        }
    }

}
 