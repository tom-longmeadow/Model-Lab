use std::sync::Arc;
use std::env;

use app::core::{App, AppLogic, TestApp};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct DesktopApp {
    logic: Box<dyn AppLogic>,
    app: Option<App>,
    window: Option<Arc<Window>>,
}

impl DesktopApp {
    pub fn new(logic: Box<dyn AppLogic>) -> Self {
        Self {
            logic,
            app: None,
            window: None,
        }
    }
}

impl ApplicationHandler for DesktopApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes().with_title(self.logic.title())
                )
                .unwrap()
        );

        let size = window.inner_size();
        self.window = Some(window.clone());

        match pollster::block_on(App::new(window, size.width, size.height, self.logic.as_ref())) {
            Ok(a) => self.app = Some(a),
            Err(e) => {
                tracing::error!("App init failed: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(app) = &mut self.app {
            app.handle_event(&event, event_loop);
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let app_name = env::args().nth(1).unwrap_or_else(|| "test".to_string());
    
    // pick which app to run with cargo run --bin desktop -- test
    let logic: Box<dyn AppLogic> = match app_name.as_str() {
        "test" => Box::new(TestApp::new()),
        //"model" => Box::new(ModelApp::new()),  // Add more apps here
        _ => {
            eprintln!("Unknown app: {}", app_name);
            return;
        }
    };

    tracing::info!("Starting Model Lab desktop: {}", app_name);

    let event_loop = EventLoop::new().unwrap();
    let mut desktop = DesktopApp::new(logic);

    if let Err(e) = event_loop.run_app(&mut desktop) {
        tracing::error!("Event loop error: {}", e);
    }

    tracing::info!("Model Lab desktop shutdown");
}




// use app::core::{App, AppLogic, TestApp};

// use winit::{
//     application::ApplicationHandler,
//     event::WindowEvent,
//     event_loop::{ActiveEventLoop, EventLoop},
//     window::{Window, WindowId},
// };


// struct DesktopApp {
//     logic: Box<dyn AppLogic>,
//     app:   Option<App>,
// }

// impl DesktopApp {
//     pub fn new(logic: Box<dyn AppLogic>) -> Self {
//         Self { logic, app: None }
//     }
// }

// impl ApplicationHandler for DesktopApp {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         tracing::info!("Window resumed — creating window");

//         let window = Arc::new(
//             event_loop.create_window(
//                 Window::default_attributes().with_title(self.logic.title())
//             ).unwrap()
//         );
//         let size = window.inner_size();
//         tracing::debug!("Window size: {}x{}", size.width, size.height);

//         match pollster::block_on(App::new(window, size.width, size.height, self.logic.as_ref())) {
//             Ok(a)  => { tracing::info!("App ready"); self.app = Some(a); }
//             Err(e) => { tracing::error!("App init failed: {}", e); event_loop.exit(); }
//         }
//     }

//     fn suspended(&mut self, _: &ActiveEventLoop) {
//         tracing::info!("Window suspended");
//     }

//     fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
//         match &event {
//             WindowEvent::CloseRequested  => tracing::info!("Close requested"),
//             WindowEvent::Resized(size)   => tracing::debug!("Resized to {}x{}", size.width, size.height),
//             WindowEvent::Focused(true)   => tracing::debug!("Window focused"),
//             WindowEvent::Focused(false)  => tracing::debug!("Window unfocused"),
//             _                            => {}
//         }
//         if let Some(app) = &mut self.app {
//             app.handle_event(&event, event_loop);
//         }
        
//         // Request continuous redraws
//         event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
//     }
// }
 