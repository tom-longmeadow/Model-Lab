use crate::{
    engine::scene::Scene, 
    graphics_context::config::RendererConfig
};
use std::sync::Arc;
use winit::event_loop::EventLoop;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::Window};
use crate::{engine::Engine}; 
use winit::application::ApplicationHandler; 
use winit::window::{WindowId};
 

pub trait AppLogic {
    fn create_scene(&self)  -> Box<dyn Scene>;
    fn create_config(&self) -> RendererConfig;
    fn title(&self)         -> &str;  
}

// Keep this to clean up the struct layout
pub struct InnerApp {
    window: Arc<Window>,
    engine: Engine,
}

pub struct AppRunner { 
    logic: Box<dyn AppLogic>, 
    app: Option<InnerApp>,
}

impl AppRunner {
    pub fn new(logic: Box<dyn AppLogic>) -> Self {
        Self { logic, app: None }
    }
}

pub fn run_app<T>(logic: T) -> Result<(), Box<dyn std::error::Error>>
where
    T: AppLogic + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    
    #[cfg(target_arch = "wasm32")]
    tracing_wasm::set_as_global_default();

    tracing::info!("Starting Model Lab application shell");

    let event_loop = EventLoop::new()?;
    let mut runner = AppRunner::new(Box::new(logic));

    event_loop.run_app(&mut runner)?;
    
    tracing::info!("Model Lab application shutdown");
    Ok(())
}

impl ApplicationHandler for AppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 3. Simple guard clause: if already initialized, do nothing
        if self.app.is_some() { return; }

        // 4. Clean access to trait values via self.logic directly
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title(self.logic.title()))
                .unwrap()
        );

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            if let Some(canvas) = window.canvas() {
                web_sys::window().unwrap().document().unwrap().body().unwrap().append_child(&canvas).ok();
            }
        }

        let size = window.inner_size();
        let config = self.logic.create_config();
        let scene = self.logic.create_scene();

        match pollster::block_on(Engine::new(window.clone(), size.width, size.height, config, scene)) {
            Ok(engine) => {
                // 5. Instantly transition to ready by populating the Option
                self.app = Some(InnerApp { window, engine });
            }
            Err(e) => {
                tracing::error!("Engine runtime initialization failed: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // 6. Transparent, nested pattern matching
        if let Some(app) = &mut self.app {
            app.engine.handle_event(&event, event_loop);

            if let WindowEvent::CloseRequested = event {
                event_loop.exit();
            }
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(app) = &self.app {
            app.window.request_redraw();
        }
    }
}