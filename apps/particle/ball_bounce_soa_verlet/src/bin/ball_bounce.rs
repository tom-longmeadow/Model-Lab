 use app::{
    core::{AppLogic,  run_app}, 
    engine::scene::{Scene, ball_bounce::soa_verlet::BallBounceParticleSoaVerletScene,  }, 
    graphics_context::config::RendererConfig,  
};
use base::math::{DVec2};


pub struct BallBounceParticleSoaVerlet { 
}

impl BallBounceParticleSoaVerlet {
    pub fn new() -> Self {
        Self { }
    }
}

impl AppLogic for BallBounceParticleSoaVerlet {
    fn title(&self) -> &str {
        "Ball Bounce SOA Verlet Particle Simulation"
    }

    fn create_config(&self) -> RendererConfig {
        RendererConfig::default()
    }

    fn create_scene(&self) -> Box<dyn Scene> { 
        Box::new(BallBounceParticleSoaVerletScene::<DVec2>::new())
    }
}
 

fn main() {
    let my_app = BallBounceParticleSoaVerlet::new();
    
    if let Err(e) = run_app(my_app) {
        eprintln!("Critical application crash: {}", e);
    }
}