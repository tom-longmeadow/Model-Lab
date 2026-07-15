use std::sync::Arc;
use std::time::Instant;

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
    last_frame_time: Instant,
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
            last_frame_time: Instant::now()
        })
    }

    // pub fn handle_event(&mut self, event: &WindowEvent, event_loop: &ActiveEventLoop) {
    //     self.input.handle(event);
    //     match event {
    //         WindowEvent::CloseRequested  => {
    //             tracing::info!("Close requested");
    //             event_loop.exit();
    //         }
    //         WindowEvent::Resized(size)   => {
    //             tracing::info!("Resize {}x{}", size.width, size.height);
    //             self.graphics.resize(size.width, size.height);
    //         }
    //         WindowEvent::RedrawRequested => self.render(event_loop),
    //         _                            => {}
    //     }
    // }

    pub fn handle_event(&mut self, event: &WindowEvent, event_loop: &ActiveEventLoop) {
        self.input.handle(event);
        
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                tracing::info!("Resize {}x{}", size.width, size.height);
                self.graphics.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                // 1. Calculate time passed since the last frame
                let now = Instant::now();
                let delta_time = now.duration_since(self.last_frame_time);
                
                // 2. Update the last frame time for the next frame
                self.last_frame_time = now;

                // 3. Pass delta time to your render logic
                let dt_seconds = delta_time.as_secs_f64();
                self.render(dt_seconds, event_loop, ); 
               
            }
            _ => {}
        }
    }
    

     fn render(&mut self, frame_time: f64, event_loop: &ActiveEventLoop) {
        tracing::trace!("Rendering...");

        // Begin frame, update scene logic
        self.input.begin_frame();
        self.scenes.update(frame_time, &self.input, &mut self.graphics);   

        // Renderer performs the render  
        match self.graphics.render(frame_time) {
            Ok(())                          => {}
            Err(RendererError::Outdated) |
            Err(RendererError::Lost)        => {
                tracing::warn!("Surface lost/outdated — reconfiguring");
                self.graphics.reconfigure();
                return; // 👈 Add this to drop out of the render function immediately!
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
 