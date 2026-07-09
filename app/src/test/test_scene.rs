use std::sync::{Arc, Mutex};

use base::{
    prelude::Locale,
    sim::simulation::Simulate,
    sim::storage::AosCpuStorage,
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
        pass::simulation_pass::SimulationPass,
        renderer::simulation::aos_renderer::AosSimulationRenderer,
        vertex::GpuVertex,
    },
    test::test_part::TestPart,
};

 
 

pub struct TestScene {
    ui: Option<Gui>,
    part: Arc<Mutex<TestPart>>,
    units: UnitSystem<ExampleModelConfig>,
}

impl TestScene {
    pub fn new() -> Self {
        Self {
            ui: None,
            part: Arc::new(Mutex::new(TestPart::new())),
            units: UnitSystem::new(ExampleUnitSettings::default()),
        }
    }
}

impl Scene for TestScene {
    fn build_passes(&mut self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }

       let panel = PropertyPanel::new(
        self.part.clone(), &self.units, Locale::EnUs);
 
        let result = GuiBuilder::build(Box::new(panel), renderer);

        self.ui = Some(result.gui);
        renderer.add_pass(result.mesh_pass);
        renderer.add_pass(result.text_pass);

        // Add simulation rendering pass — SimulationPass drives the sim and renderer together.
        let sim = new_simple_sim();
        let particle_renderer = AosSimulationRenderer::new(
            sim.storage().as_slice().to_vec(),
            |p: &Particle| GpuVertex {
                position: [p.pos.x as f32, p.pos.y as f32, 0.0],
                normal:   [0.0, 0.0, 1.0],
                uv:       [0.0, 0.0],
                color:    [1.0, 0.5, 0.2, 1.0],
            },
            0.05,
        );
        let sim_pass = SimulationPass::new(sim, particle_renderer, 1.0 / 60.0);
        renderer.add_pass(sim_pass);

 
        
    }

    fn update(&mut self, _input: &InputState) {
        if let Some(ui) = &mut self.ui {
            // In a real update loop, you would:
            // 1. Handle input events and update the ChangeMap in the Gui.
            // ui.handle_input(input, &mut self.part.lock().unwrap());

            // 2. Drain changes and update the model or other UI parts.
            let changes = ui.drain_changes();
            if !changes.is_empty() {
                // ...logic to react to changed properties...
                println!("Properties changed: {:?}", changes);
            }

            // 3. Re-layout the UI if needed (e.g., on window resize).
            // let mut measurer = GlyphonTextMeasurer::new();
            // ui.layout(screen_size, &mut measurer);
        }

        // // Update simulation
        // self.sim.simulate(0.016); // ~60 FPS
        
        // // Extract and pass to renderer
        // let updated_particles = self.sim.storage().as_slice().to_vec();
        // if let Some(pass) = &mut self.sim_pass {
        //     pass.renderer.update_data(updated_particles);
        // }

    }

    fn update_passes(&mut self, renderer: &mut GraphicsContext) {
        // In a real application, you would update the `data` fields of the
        // existing passes here instead of rebuilding them every frame.
        if let Some(ui) = &self.ui {
            // Example of updating passes:
            // 1. Re-collect data if UI is dirty
            // let new_mesh_data = ...
            // let new_text_data = ...

            // 2. Find the pass and update its data
            // if let Some(mesh_pass) = renderer.get_pass_mut::<RenderPass<Vec<Mesh>>>(0) {
            //     mesh_pass.data = new_mesh_data;
            // }
            // if let Some(text_pass) = renderer.get_pass_mut::<RenderPass<TextParams>>(1) {
            //     text_pass.data = new_text_data;
            // }
        }
    }
}


// use std::sync::{Arc, Mutex};

// use base::{
//     mesh::Mesh,
//     prelude::Locale,
//     ui::{
//         layout::{layout_params::LayoutParams, rect::Rect, size::Size}, 
//         text::params::{TextGroup, TextParams}, widget::{Widget, collect_text}, 
//         widgets::property_panel::PropertyPanel
//     },
//     unit::{UnitSettings, UnitSystem},
// };
// use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};

// use crate::{
//     engine::{input::InputState, scene::Scene, gui::Gui},
//     graphics_context::{
//         GraphicsContext, pass::RenderPass, renderer::{mesh_renderer::MeshRenderer, text_renderer::TextRenderer}
//     },
//     test::test_part::TestPart,
//     ui::{
//         mesh_builder::UiMeshBuilder, 
//         text_measurer::GlyphonTextMeasurer,
//     },
// };

// #[derive(Default)]
// pub struct TestScene{
//     pub ui : Option<Gui>
// }

// impl TestScene {
//     pub fn new() -> Self {
//         Self::default()
//     }
//     pub fn set_ui(&mut self, widget: Widget){
//         self.ui = Some(Gui::new(Box::new(widget)));
//     }
// }

// impl Scene for TestScene {
//     fn update(&mut self, _input: &InputState) {}

//     fn build_passes(&self, renderer: &mut GraphicsContext) {
//         if renderer.pass_count() > 0 {
//             return;
//         }

//         // --- UI Logic: Create widgets and calculate layout ---
//         let params = LayoutParams::default();
//         let part = Arc::new(Mutex::new(TestPart::new()));
//         let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());

//         let mut panel = PropertyPanel::new(part, &units, Locale::EnUs);
//         self.ui.set_ui(panel);

//         let mut measurer = GlyphonTextMeasurer::new();
//         let screen = Size {
//             w: renderer.width() as f32,
//             h: renderer.height() as f32,
//         };

//         let measured = panel.measure(screen, &params, &mut measurer);
//         panel.arrange(
//             Rect {
//                 x: 16.0,
//                 y: 16.0,
//                 w: measured.w,
//                 h: measured.h,
//             },
//             &params,
//             &mut measurer,
//         );

   

//         // 1. Collect mesh data
//        let ui_mesh_data: Vec<Mesh> = UiMeshBuilder::new(&params).build(&panel);
//         println!("UI meshes collected: {}", ui_mesh_data.len());

//         // 2. Collect text data using the correct types from `base`
//         let mut text_params: Vec<TextGroup> = Vec::new();
//         collect_text(&panel, &mut text_params, &params);
//         let ui_text_data = TextParams {
//             groups: text_params,
//         };
//         println!("UI text groups collected: {}", ui_text_data.groups.len());


        

//         // Create and add the UI Mesh Pass
//         let mesh_renderer = MeshRenderer::new();
//         let mesh_pass = RenderPass::new(ui_mesh_data, mesh_renderer);
//         renderer.add_pass(mesh_pass);

//         // Create and add the UI Text Pass
//         let text_renderer = TextRenderer::new();
//         let text_pass = RenderPass::new(ui_text_data, text_renderer);
//         renderer.add_pass(text_pass);
//     }

//     fn update_passes(&self, _renderer: &mut GraphicsContext) {
//         // In a real application, you would update the `data` fields of the
//         // existing passes here instead of rebuilding them every frame.
//         // For example:
//         // 1. Find the mesh pass
//         // 2. Re-run the UI logic and `UiMeshBuilder`
//         // 3. Set `mesh_pass.data = new_mesh_data`
//     }
// }


// // use std::sync::{Arc, Mutex};

// // use base::{prelude::Locale, ui::{
// //     layout::{layout_params::LayoutParams, rect::Rect, size::Size},
// //     text::params::TextParams,
// //     widget::{Widget, collect_rects, collect_text},
// //     widgets::property_panel::PropertyPanel,
// // }, unit::{UnitSettings, UnitSystem}};
// // use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
// // use crate::{
// //     engine::{input::InputState, scene::Scene}, graphics_context::GraphicsContext, test::test_part::TestPart, ui::{mesh_builder::UiMeshBuilder, text_measurer::GlyphonTextMeasurer}
// // };

// // #[derive(Default)]
// // pub struct TestScene;

// // impl TestScene {
// //     pub fn new() -> Self { Self::default() }
// // }

// // impl Scene for TestScene {
// //     fn update(&mut self, _input: &InputState) {}

// //     fn build_passes(&self, renderer: &mut GraphicsContext) {
// //         if renderer.pass_count() == 0 {
// //             let params = LayoutParams::default();
// //             let part   = Arc::new(Mutex::new(TestPart::new()));
// //             let units  = UnitSystem::<ExampleModelConfig>::new(
// //                 ExampleUnitSettings::default());

// //             let mut panel = PropertyPanel::new(part, &units, Locale::EnUs);

// //             let mut measurer = GlyphonTextMeasurer::new();
// //             let screen = Size {
// //                 w: renderer.width() as f32,
// //                 h: renderer.height() as f32,
// //             };

// //             let measured = panel.measure(screen, &params, &mut measurer);
// //             panel.arrange(
// //                 Rect { x: 16.0, y: 16.0, w: measured.w, h: measured.h },
// //                 &params,
// //                 &mut measurer,
// //             );

// //             let mut text_params = Vec::new();
// //             collect_text(&panel, &mut text_params, &params);

// //             let mut models = Vec::new();
// //             collect_rects(&panel, &mut models);
// //             println!("box models collected: {}", models.len());

// //             let ui_pass = UiMeshBuilder::new(&params).build(
// //                 &panel, 
// //             );
// //             renderer.add_pass(Box::new(ui_pass));
// //             renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(text_params))));
// //         }
// //     }
    
// //     fn update_passes(&self, renderer: &mut GraphicsContext) {
       
// //     }
// // }

// // // use std::sync::{Arc, Mutex};

// // // use base::{prelude::Locale, ui::{
// // //     layout::{layout_params::LayoutParams, rect::Rect, size::Size},
// // //     text::params::TextParams,
// // //     widget::{Widget, collect_rects, collect_text},
// // //     widgets::{column::Column, panel::Panel, property_panel::PropertyPanel},
// // // }, unit::{UnitSettings, UnitSystem}};
// // // use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
// // // use crate::{ 
// // //     engine::{input::InputState, scene::Scene},
// // //     renderer::{
// // //         Renderer, pass::text::{TextRenderPass, measurer::GlyphonTextMeasurer}, ui_pass_builder::UiPassBuilder
// // //     }, test::test_part::TestPart,
// // // };

// // // #[derive(Default)]
// // // pub struct TestScene;

// // // impl TestScene {
// // //     pub fn new() -> Self {
// // //         Self::default()
// // //     }

// // //     pub fn build_ui(params: &LayoutParams) -> Panel {
// // //         let part  = Arc::new(Mutex::new(TestPart::new()));
// // //         let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());

// // //         let property_panel = PropertyPanel::new(part, &units, Locale::EnUs);

// // //         let mut root_col = Column::new();
// // //         root_col.push(Box::new(property_panel));

// // //         Panel::new()
// // //             .with_child(Box::new(root_col))
// // //     }
// // // }

// // // impl Scene for TestScene {
// // //     fn update(&mut self, _input: &InputState) {}

// // //     fn build_passes(&self, renderer: &mut Renderer) {
// // //         if renderer.pass_count() == 0 {
// // //             let params = LayoutParams::default();
// // //             let mut panel = Self::build_ui(&params);

// // //             let mut measurer = GlyphonTextMeasurer::new();
// // //             let screen = Size {
// // //                 w: renderer.width() as f32,
// // //                 h: renderer.height() as f32,
// // //             };

// // //             let measured = panel.measure(screen, &params, &mut measurer);
// // //             panel.arrange(
// // //                 Rect { x: 16.0, y: 16.0, w: measured.w, h: measured.h },
// // //                 &params,
// // //                 &mut measurer,
// // //             );

// // //             let mut groups = Vec::new();
// // //             collect_text(&panel, &mut groups, &params);

// // //             let mut models = Vec::new();
// // //             collect_rects(&panel, &mut models);
// // //             println!("box models collected: {}", models.len());

// // //             let ui_pass = UiPassBuilder::new(&params).build(
// // //                 &panel,
// // //                 renderer.width() as f32,
// // //                 renderer.height() as f32,
// // //             );
// // //             renderer.add_pass(Box::new(ui_pass));
// // //             renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(groups))));
// // //         }
// // //     }
// // // }


// // // // use base::{prelude::Locale, ui::{
// // // //     layout::{layout_params::LayoutParams, rect::Rect, size::Size},
// // // //     text::{params::TextParams},
// // // //     widget::{Widget, collect_rects, collect_text},
// // // //     widgets::{column::Column, grid::Grid, label::Label, panel::Panel, property_panel::PropertyPanel, row::Row, text_field::TextField},
// // // // }, unit::{UnitSettings, UnitSystem}};
// // // // use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
// // // // use crate::{
// // // //     core::test::part::Part,
// // // //     engine::{input::InputState, scene::Scene},
// // // //     renderer::{
// // // //         Renderer, pass::text::{TextRenderPass, measurer::GlyphonTextMeasurer}, ui_pass_builder::UiPassBuilder
// // // //     },
// // // // };

 

// // // // #[derive(Default)]
// // // // pub struct TestScene;

// // // // impl TestScene {
// // // //     pub fn new() -> Self {
// // // //         Self::default()
// // // //     }

// // // //     pub fn build_ui(params: &LayoutParams) -> Panel {
// // // //         // Row 1: app/header info
// // // //         //let mut header_row = Row::new();

// // // //         // header_row.push(Box::new(Label::new("Model Lab")));

// // // //         // // Row 2: primary editable fields
// // // //         // let mut input_row = Row::new();
// // // //         // input_row.push(Box::new(Label::new("Name:")));
// // // //         // input_row.push(Box::new(TextField::new("Engine Bolt")));

// // // //         // // Grid: detailed properties (2 columns = label/value)
// // // //         // let mut details_grid = Grid::new(2);
// // // //         // details_grid.push(Box::new(Label::new("Mass")));
// // // //         // details_grid.push(Box::new(TextField::new("1.42 kg")));
// // // //         // details_grid.push(Box::new(Label::new("Tolerance")));
// // // //         // details_grid.push(Box::new(TextField::new("±0.05 mm")));
// // // //         // details_grid.push(Box::new(Label::new("Batch")));
// // // //         // details_grid.push(Box::new(TextField::new("A-104")));
// // // //         // details_grid.push(Box::new(Label::new("Process")));
// // // //         // details_grid.push(Box::new(TextField::new("CNC")));
// // // //         // details_grid.push(Box::new(Label::new("Supplier")));
// // // //         // details_grid.push(Box::new(TextField::new("Acme Industrial")));

// // // //         // PropertyPanel from mock Part object
// // // //         let part = Part::new();
// // // //         let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());
// // // //         let property_panel = PropertyPanel::<ExampleModelConfig>::new(
// // // //             &part,
// // // //             &units,
// // // //             Locale::EnUs,  
// // // //             params,
// // // //         );
// // // //         // Column containing header, input row, grid, and property panel
// // // //         let mut root_col = Column::new();
// // // //         // root_col.push(Box::new(header_row));
// // // //         // root_col.push(Box::new(input_row));
// // // //         // root_col.push(Box::new(details_grid));
// // // //         root_col.push(Box::new(property_panel.into_column()));

// // // //         // Top-level panel contains the column
// // // //         Panel::new()
// // // //             .with_child(Box::new(root_col))
// // // //     }
// // // // }

// // // // impl Scene for TestScene {
// // // //     fn update(&mut self, _input: &InputState) {}

// // // //     fn build_passes(&self, renderer: &mut Renderer) {
// // // //         if renderer.pass_count() == 0 {
            
// // // //             let params = LayoutParams::default();
// // // //             let mut panel = Self::build_ui(&params);

// // // //             let mut measurer = GlyphonTextMeasurer::new();
// // // //             let screen = Size {
// // // //                 w: renderer.width() as f32,
// // // //                 h: renderer.height() as f32,
// // // //             };

// // // //             let measured = panel.measure(screen, &params, &mut measurer);
// // // //             panel.arrange(
// // // //                 Rect {
// // // //                     x: 16.0,
// // // //                     y: 16.0,
// // // //                     w: measured.w,
// // // //                     h: measured.h,
// // // //                 },
// // // //                 &params,
// // // //                 &mut measurer,
// // // //             );

// // // //             let mut groups = Vec::new();
// // // //             collect_text(&panel, &mut groups, &params);

// // // //             // debug: check box models
// // // //             let mut models = Vec::new();
// // // //             collect_rects(&panel, &mut models);
// // // //             println!("box models collected: {}", models.len());

// // // //             let ui_pass = UiPassBuilder::new(&params).build(
// // // //                 &panel,
// // // //                 renderer.width() as f32,
// // // //                 renderer.height() as f32,
// // // //             );
// // // //             renderer.add_pass(Box::new(ui_pass));
// // // //             renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(groups))));
 


// // // //         }
// // // //     }
// // // // }