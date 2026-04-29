use crate::{core::AppLogic, engine::scene::Scene, renderer::config::RendererConfig, test::test_scene::TestScene};

 

pub struct TestApp;

impl TestApp {
    pub fn new() -> Self {
        Self
    }
}

impl AppLogic for TestApp {
    fn create_scene(&self) -> Box<dyn Scene> {
        Box::new(TestScene::new())
    }

    fn create_config(&self) -> RendererConfig {
        RendererConfig {
            clear_color: wgpu::Color { r: 1.0, g: 0.1, b: 0.1, a: 1.0 },
            ..RendererConfig::default()
        }
    }

    fn title(&self) -> &str {
        "Test App"
    }
}

 