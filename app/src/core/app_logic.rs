use crate::{
    engine::scene::Scene, 
    renderer::config::RendererConfig
};


pub trait AppLogic {
    fn create_scene(&self)  -> Box<dyn Scene>;
    fn create_config(&self) -> RendererConfig;
    fn title(&self)         -> &str { "Model Lab" }  // optional default
}