use crate::{
    engine::scene::Scene, 
    renderer::config::RendererConfig
};
use std::sync::Arc;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};
use crate::{engine::Engine, renderer::error::RendererError};

  
pub trait AppLogic {
    fn create_scene(&self)  -> Box<dyn Scene>;
    fn create_config(&self) -> RendererConfig;
    fn title(&self)         -> &str;  
}

 
pub struct App {
    engine: Engine,
}

impl App {
    pub async fn new(
        window: Arc<Window>,
        width:  u32,
        height: u32,
        logic:  &dyn AppLogic,
    ) -> Result<Self, RendererError> {
        let config = logic.create_config();
        let scene  = logic.create_scene();
        let engine = Engine::new(window, width, height, config, scene).await?;
        Ok(Self { engine })
    }

    pub fn handle_event(&mut self, event: &WindowEvent, event_loop: &ActiveEventLoop) {
        self.engine.handle_event(event, event_loop);
    }
}