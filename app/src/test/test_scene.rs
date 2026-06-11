use std::sync::{Arc, Mutex};

use base::{
    mesh::Mesh,
    prelude::Locale,
    ui::{
        layout::{layout_params::LayoutParams, rect::Rect, size::Size}, 
        text::params::{TextGroup, TextParams}, widget::{Widget, collect_text}, 
        widgets::property_panel::PropertyPanel
    },
    unit::{UnitSettings, UnitSystem},
};
use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};

use crate::{
    engine::{input::InputState, scene::Scene},
    graphics_context::{
        pass::RenderPass,
        renderer::{mesh_renderer::MeshRenderer, text_renderer::TextRenderer},
        GraphicsContext,
    },
    test::test_part::TestPart,
    ui::{
        mesh_builder::UiMeshBuilder, 
        text_measurer::GlyphonTextMeasurer,
    },
};

#[derive(Default)]
pub struct TestScene;

impl TestScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Scene for TestScene {
    fn update(&mut self, _input: &InputState) {}

    fn build_passes(&self, renderer: &mut GraphicsContext) {
        if renderer.pass_count() > 0 {
            return;
        }

        // --- UI Logic: Create widgets and calculate layout ---
        let params = LayoutParams::default();
        let part = Arc::new(Mutex::new(TestPart::new()));
        let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());

        let mut panel = PropertyPanel::new(part, &units, Locale::EnUs);

        let mut measurer = GlyphonTextMeasurer::new();
        let screen = Size {
            w: renderer.width() as f32,
            h: renderer.height() as f32,
        };

        let measured = panel.measure(screen, &params, &mut measurer);
        panel.arrange(
            Rect {
                x: 16.0,
                y: 16.0,
                w: measured.w,
                h: measured.h,
            },
            &params,
            &mut measurer,
        );

        // --- Data Collection: Extract renderable data from widgets ---

        // 1. Collect mesh data
       let ui_mesh_data: Vec<Mesh> = UiMeshBuilder::new(&params).build(&panel);
        println!("UI meshes collected: {}", ui_mesh_data.len());

        // 2. Collect text data using the correct types from `base`
        let mut text_params: Vec<TextGroup> = Vec::new();
        collect_text(&panel, &mut text_params, &params);
        let ui_text_data = TextParams {
            groups: text_params,
        };
        println!("UI text groups collected: {}", ui_text_data.groups.len());


        // --- Pass Creation: Combine data and renderers into passes ---

        // Create and add the UI Mesh Pass
        let mesh_renderer = MeshRenderer::new();
        let mesh_pass = RenderPass::new(ui_mesh_data, mesh_renderer);
        renderer.add_pass(mesh_pass);

        // Create and add the UI Text Pass
        let text_renderer = TextRenderer::new();
        let text_pass = RenderPass::new(ui_text_data, text_renderer);
        renderer.add_pass(text_pass);
    }

    fn update_passes(&self, _renderer: &mut GraphicsContext) {
        // In a real application, you would update the `data` fields of the
        // existing passes here instead of rebuilding them every frame.
        // For example:
        // 1. Find the mesh pass
        // 2. Re-run the UI logic and `UiMeshBuilder`
        // 3. Set `mesh_pass.data = new_mesh_data`
    }
}


// use std::sync::{Arc, Mutex};

// use base::{prelude::Locale, ui::{
//     layout::{layout_params::LayoutParams, rect::Rect, size::Size},
//     text::params::TextParams,
//     widget::{Widget, collect_rects, collect_text},
//     widgets::property_panel::PropertyPanel,
// }, unit::{UnitSettings, UnitSystem}};
// use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
// use crate::{
//     engine::{input::InputState, scene::Scene}, graphics_context::GraphicsContext, test::test_part::TestPart, ui::{mesh_builder::UiMeshBuilder, text_measurer::GlyphonTextMeasurer}
// };

// #[derive(Default)]
// pub struct TestScene;

// impl TestScene {
//     pub fn new() -> Self { Self::default() }
// }

// impl Scene for TestScene {
//     fn update(&mut self, _input: &InputState) {}

//     fn build_passes(&self, renderer: &mut GraphicsContext) {
//         if renderer.pass_count() == 0 {
//             let params = LayoutParams::default();
//             let part   = Arc::new(Mutex::new(TestPart::new()));
//             let units  = UnitSystem::<ExampleModelConfig>::new(
//                 ExampleUnitSettings::default());

//             let mut panel = PropertyPanel::new(part, &units, Locale::EnUs);

//             let mut measurer = GlyphonTextMeasurer::new();
//             let screen = Size {
//                 w: renderer.width() as f32,
//                 h: renderer.height() as f32,
//             };

//             let measured = panel.measure(screen, &params, &mut measurer);
//             panel.arrange(
//                 Rect { x: 16.0, y: 16.0, w: measured.w, h: measured.h },
//                 &params,
//                 &mut measurer,
//             );

//             let mut text_params = Vec::new();
//             collect_text(&panel, &mut text_params, &params);

//             let mut models = Vec::new();
//             collect_rects(&panel, &mut models);
//             println!("box models collected: {}", models.len());

//             let ui_pass = UiMeshBuilder::new(&params).build(
//                 &panel, 
//             );
//             renderer.add_pass(Box::new(ui_pass));
//             renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(text_params))));
//         }
//     }
    
//     fn update_passes(&self, renderer: &mut GraphicsContext) {
       
//     }
// }

// // use std::sync::{Arc, Mutex};

// // use base::{prelude::Locale, ui::{
// //     layout::{layout_params::LayoutParams, rect::Rect, size::Size},
// //     text::params::TextParams,
// //     widget::{Widget, collect_rects, collect_text},
// //     widgets::{column::Column, panel::Panel, property_panel::PropertyPanel},
// // }, unit::{UnitSettings, UnitSystem}};
// // use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
// // use crate::{ 
// //     engine::{input::InputState, scene::Scene},
// //     renderer::{
// //         Renderer, pass::text::{TextRenderPass, measurer::GlyphonTextMeasurer}, ui_pass_builder::UiPassBuilder
// //     }, test::test_part::TestPart,
// // };

// // #[derive(Default)]
// // pub struct TestScene;

// // impl TestScene {
// //     pub fn new() -> Self {
// //         Self::default()
// //     }

// //     pub fn build_ui(params: &LayoutParams) -> Panel {
// //         let part  = Arc::new(Mutex::new(TestPart::new()));
// //         let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());

// //         let property_panel = PropertyPanel::new(part, &units, Locale::EnUs);

// //         let mut root_col = Column::new();
// //         root_col.push(Box::new(property_panel));

// //         Panel::new()
// //             .with_child(Box::new(root_col))
// //     }
// // }

// // impl Scene for TestScene {
// //     fn update(&mut self, _input: &InputState) {}

// //     fn build_passes(&self, renderer: &mut Renderer) {
// //         if renderer.pass_count() == 0 {
// //             let params = LayoutParams::default();
// //             let mut panel = Self::build_ui(&params);

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

// //             let mut groups = Vec::new();
// //             collect_text(&panel, &mut groups, &params);

// //             let mut models = Vec::new();
// //             collect_rects(&panel, &mut models);
// //             println!("box models collected: {}", models.len());

// //             let ui_pass = UiPassBuilder::new(&params).build(
// //                 &panel,
// //                 renderer.width() as f32,
// //                 renderer.height() as f32,
// //             );
// //             renderer.add_pass(Box::new(ui_pass));
// //             renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(groups))));
// //         }
// //     }
// // }


// // // use base::{prelude::Locale, ui::{
// // //     layout::{layout_params::LayoutParams, rect::Rect, size::Size},
// // //     text::{params::TextParams},
// // //     widget::{Widget, collect_rects, collect_text},
// // //     widgets::{column::Column, grid::Grid, label::Label, panel::Panel, property_panel::PropertyPanel, row::Row, text_field::TextField},
// // // }, unit::{UnitSettings, UnitSystem}};
// // // use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
// // // use crate::{
// // //     core::test::part::Part,
// // //     engine::{input::InputState, scene::Scene},
// // //     renderer::{
// // //         Renderer, pass::text::{TextRenderPass, measurer::GlyphonTextMeasurer}, ui_pass_builder::UiPassBuilder
// // //     },
// // // };

 

// // // #[derive(Default)]
// // // pub struct TestScene;

// // // impl TestScene {
// // //     pub fn new() -> Self {
// // //         Self::default()
// // //     }

// // //     pub fn build_ui(params: &LayoutParams) -> Panel {
// // //         // Row 1: app/header info
// // //         //let mut header_row = Row::new();

// // //         // header_row.push(Box::new(Label::new("Model Lab")));

// // //         // // Row 2: primary editable fields
// // //         // let mut input_row = Row::new();
// // //         // input_row.push(Box::new(Label::new("Name:")));
// // //         // input_row.push(Box::new(TextField::new("Engine Bolt")));

// // //         // // Grid: detailed properties (2 columns = label/value)
// // //         // let mut details_grid = Grid::new(2);
// // //         // details_grid.push(Box::new(Label::new("Mass")));
// // //         // details_grid.push(Box::new(TextField::new("1.42 kg")));
// // //         // details_grid.push(Box::new(Label::new("Tolerance")));
// // //         // details_grid.push(Box::new(TextField::new("±0.05 mm")));
// // //         // details_grid.push(Box::new(Label::new("Batch")));
// // //         // details_grid.push(Box::new(TextField::new("A-104")));
// // //         // details_grid.push(Box::new(Label::new("Process")));
// // //         // details_grid.push(Box::new(TextField::new("CNC")));
// // //         // details_grid.push(Box::new(Label::new("Supplier")));
// // //         // details_grid.push(Box::new(TextField::new("Acme Industrial")));

// // //         // PropertyPanel from mock Part object
// // //         let part = Part::new();
// // //         let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());
// // //         let property_panel = PropertyPanel::<ExampleModelConfig>::new(
// // //             &part,
// // //             &units,
// // //             Locale::EnUs,  
// // //             params,
// // //         );
// // //         // Column containing header, input row, grid, and property panel
// // //         let mut root_col = Column::new();
// // //         // root_col.push(Box::new(header_row));
// // //         // root_col.push(Box::new(input_row));
// // //         // root_col.push(Box::new(details_grid));
// // //         root_col.push(Box::new(property_panel.into_column()));

// // //         // Top-level panel contains the column
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
// // //                 Rect {
// // //                     x: 16.0,
// // //                     y: 16.0,
// // //                     w: measured.w,
// // //                     h: measured.h,
// // //                 },
// // //                 &params,
// // //                 &mut measurer,
// // //             );

// // //             let mut groups = Vec::new();
// // //             collect_text(&panel, &mut groups, &params);

// // //             // debug: check box models
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