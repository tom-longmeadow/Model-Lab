use std::sync::{Arc, Mutex};

use base::{
    // prelude::Locale,
    // ui::widgets::property_panel::PropertyPanel,
    // unit::{UnitSettings, UnitSystem},
    math::{Bounds, DVec2},   ui::layout::color::Color
};
use impls::{
    // model::model_example::{ExampleModelConfig, ExampleUnitSettings},
    simulation::verlet_2d::{sim_simple::new_verlet2d_gravity_sim, particle::Particle},
};

use crate::{
    engine::{gui::Gui, 
        // gui_builder::GuiBuilder, 
        input::InputState, scene::Scene},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState},
        simulation::{aos::AosSimulationRenderer, pass::{SimulationPass}},
    },
    //est::test_part::TestPart,
};

pub struct TestScene {
    ui: Option<Gui>,
    // part: Arc<Mutex<TestPart>>,
    // units: UnitSystem<ExampleModelConfig>,
    hud_state: Arc<Mutex<HudState>>,
}

impl TestScene {
    pub fn new() -> Self {
        Self {
            ui: None,
            // part: Arc::new(Mutex::new(TestPart::new())),
            // units: UnitSystem::new(ExampleUnitSettings::default()),
            hud_state: Arc::new(Mutex::new(HudState::default())),
        }
    }
}

impl Scene for TestScene {
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }

        // let panel = PropertyPanel::new(self.part.clone(), &self.units, Locale::EnUs);
        // let result = GuiBuilder::build(Box::new(panel), renderer);
        // self.ui = Some(result.gui);
        // renderer.add_pass(result.mesh_pass);
        // renderer.add_pass(result.text_pass);

        // Define simulation bounds in simulation space (e.g., 0-10 units)
        let sim_bounds = Bounds::new_2d((0.0, 400.0), (0.0, 400.0));
        
      let rainbow: Vec<Color> = vec![
        Color { r: 255, g: 0,   b: 0,   a: 255 }, // Vibrant Red
        Color { r: 255, g: 127, b: 0,   a: 255 }, // Vibrant Orange
        Color { r: 255, g: 255, b: 0,   a: 255 }, // Vibrant Yellow
        Color { r: 0,   g: 255, b: 0,   a: 255 }, // Vibrant Green
        Color { r: 0,   g: 0,   b: 255, a: 255 }, // Vibrant Blue
        Color { r: 139, g: 0,   b: 255, a: 255 }, // Vibrant Purple / Violet
    ];

        let sim = new_verlet2d_gravity_sim(
            60.0, 4, sim_bounds, 40.0, 1100.0, 0.5,
        10, 3, 800, DVec2 { x: 5.0, y: 2.0 }, 10.0, rainbow);

        let particle_renderer = AosSimulationRenderer::<Particle>::new();

        // let particle_renderer = AosSimulationRenderer::new(
        //     |p: &Particle| ParticleInstance {
        //         position: [p.pos.x as f32, p.pos.y as f32, 0.0],
        //         radius_x: p.radius as f32, 
        //         color: p.color.as_f32_array(), 
        //     },
        // );
        renderer.add_pass(
            SimulationPass::new(sim, particle_renderer, sim_bounds)
                .with_hud(self.hud_state.clone())
        );
        renderer.add_pass(HudPass::new(self.hud_state.clone()));
    }

    fn update(&mut self, _frame_time: f64, _input: &InputState, _renderer: &mut GraphicsContext) {
        if let Some(ui) = &mut self.ui {
            let changes = ui.drain_changes();
            if !changes.is_empty() {
                println!("Properties changed: {:?}", changes);
            }
        }
    }

    
}
