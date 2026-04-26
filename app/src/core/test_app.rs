use crate::{  
    engine::scene::{Scene, TestScene}, 
    core::app_logic::AppLogic, 
    renderer::config::RendererConfig
};


pub struct  TestApp;

impl TestApp{
    pub fn new() -> Self {
        Self {}
    }

}

impl AppLogic for TestApp {

   
    fn create_scene(&self) -> Box<dyn Scene> {
        Box::new(TestScene::new())
    }
    fn create_config(&self) -> RendererConfig {
        RendererConfig {
            clear_color: wgpu::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
            ..RendererConfig::default()
        }
    }
    fn title(&self) -> &str { "Model Lab" }
}