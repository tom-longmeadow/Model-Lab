 use app::{
    core::{AppLogic,  run_app}, 
    engine::scene::{Scene, ball_bounce::aos_verlet::BallBounceParticleAosVerletScene}, 
    graphics_context::config::RendererConfig,  
};
use base::math::DVec2;


pub struct BallBounceParticleAosVerlet { 
}

impl BallBounceParticleAosVerlet {
    pub fn new() -> Self {
        Self { }
    }
}

impl AppLogic for BallBounceParticleAosVerlet {
    fn title(&self) -> &str {
        "Ball Bounce AOS Verlet Particle Simulation"
    }

    fn create_config(&self) -> RendererConfig {
        RendererConfig::default()
    }

    fn create_scene(&self) -> Box<dyn Scene> { 
        Box::new(BallBounceParticleAosVerletScene::<DVec2>::new())
    }
}
 

fn main() {
    let my_app = BallBounceParticleAosVerlet::new();
    
    if let Err(e) = run_app(my_app) {
        eprintln!("Critical application crash: {}", e);
    }
}