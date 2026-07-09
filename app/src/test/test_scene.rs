use std::sync::{Arc, Mutex};

use base::{
    prelude::Locale,
    ui::widgets::property_panel::PropertyPanel,
    unit::{UnitSettings, UnitSystem},
};
use impls::{
    model::model_example::{ExampleModelConfig, ExampleUnitSettings},
    simulation::verlet_2d::{sim_simple::new_simple_sim, particle::Particle},
};

use crate::{
    engine::{gui::Gui, gui_builder::GuiBuilder, input::InputState, scene::Scene},
    graphics_context::{
        GraphicsContext,
        pass::hud::{HudPass, HudState},
        simulation::{aos::AosSimulationRenderer, pass::SimulationPass},
        vertex::GpuVertex,
    },
    test::test_part::TestPart,
};

pub struct TestScene {
    ui: Option<Gui>,
    part: Arc<Mutex<TestPart>>,
    units: UnitSystem<ExampleModelConfig>,
    hud_state: Arc<Mutex<HudState>>,
}

impl TestScene {
    pub fn new() -> Self {
        Self {
            ui: None,
            part: Arc::new(Mutex::new(TestPart::new())),
            units: UnitSystem::new(ExampleUnitSettings::default()),
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

        let sim = new_simple_sim();
        let particle_renderer = AosSimulationRenderer::new(
            |p: &Particle| GpuVertex {
                position: [p.pos.x as f32, p.pos.y as f32, 0.0],
                normal:   [0.0, 0.0, 1.0],
                uv:       [0.0, 0.0],
                color:    [1.0, 0.5, 0.2, 1.0],
            },
            0.05,
        );
        renderer.add_pass(
            SimulationPass::new(sim, particle_renderer, 1.0 / 60.0)
                .with_hud(self.hud_state.clone())
        );
        renderer.add_pass(HudPass::new(self.hud_state.clone()));
    }

    fn update(&mut self, _input: &InputState) {
        if let Ok(mut hud) = self.hud_state.try_lock() {
            hud.set("Particles", "1");
        }

        if let Some(ui) = &mut self.ui {
            let changes = ui.drain_changes();
            if !changes.is_empty() {
                println!("Properties changed: {:?}", changes);
            }
        }
    }

    fn update_passes(&mut self, _renderer: &mut GraphicsContext) {}
}
