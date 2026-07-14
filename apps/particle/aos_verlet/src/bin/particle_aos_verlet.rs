 use app::{
    core::{AppLogic,  run_app}, 
    engine::scene::{Scene, particle_aos_verlet::ParticleAosVerletScene}, 
    graphics_context::config::RendererConfig,  
};


pub struct ParticleAosVerlet { 
}

impl ParticleAosVerlet {
    pub fn new() -> Self {
        Self { }
    }
}

impl AppLogic for ParticleAosVerlet {
    fn title(&self) -> &str {
        "AOS Verlet Particle Simulation"
    }

    fn create_config(&self) -> RendererConfig {
        RendererConfig::default()
    }

    fn create_scene(&self) -> Box<dyn Scene> { 
        Box::new(ParticleAosVerletScene::new())
    }
}
 

fn main() {
    let my_app = ParticleAosVerlet::new();
    
    if let Err(e) = run_app(my_app) {
        eprintln!("Critical application crash: {}", e);
    }
}