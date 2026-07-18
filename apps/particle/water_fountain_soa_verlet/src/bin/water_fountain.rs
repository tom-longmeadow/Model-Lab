 use app::{
    core::{AppLogic,  run_app}, 
    engine::scene::{Scene, 
        water_fountain::soa_verlet::WaterFountainParticleSoaVerletScene,  }, 
    graphics_context::config::RendererConfig,  
};
use base::math::{DVec2};


pub struct WaterFountainParticleSoaVerlet { 
}

impl WaterFountainParticleSoaVerlet {
    pub fn new() -> Self {
        Self { }
    }
}

impl AppLogic for WaterFountainParticleSoaVerlet {
    fn title(&self) -> &str {
        "Water Fountain SOA Verlet Particle Simulation"
    }

    fn create_config(&self) -> RendererConfig {
        RendererConfig::default()
    }

    fn create_scene(&self) -> Box<dyn Scene> { 
        Box::new(WaterFountainParticleSoaVerletScene::<DVec2>::new())
    }
}
 

fn main() {
    let my_app = WaterFountainParticleSoaVerlet::new();
    
    if let Err(e) = run_app(my_app) {
        eprintln!("Critical application crash: {}", e);
    }
}