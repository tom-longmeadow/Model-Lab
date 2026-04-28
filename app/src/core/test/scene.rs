use std::sync::{Arc, Mutex};

use base::{prelude::Locale, ui::{
    layout::{layout_params::LayoutParams, rect::Rect, size::Size},
    text::params::TextParams,
    widget::{Widget, collect_rects, collect_text},
    widgets::{column::Column, panel::Panel, property_panel::PropertyPanel},
}, unit::{UnitSettings, UnitSystem}};
use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
use crate::{
    core::test::part::Part,
    engine::{input::InputState, scene::Scene},
    renderer::{
        Renderer, pass::text::{TextRenderPass, measurer::GlyphonTextMeasurer}, ui_pass_builder::UiPassBuilder
    },
};

#[derive(Default)]
pub struct TestScene;

impl TestScene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_ui(params: &LayoutParams) -> Panel {
        let part  = Arc::new(Mutex::new(Part::new()));
        let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());

        let property_panel = PropertyPanel::new(part, &units, Locale::EnUs);

        let mut root_col = Column::new();
        root_col.push(Box::new(property_panel));

        Panel::new()
            .with_child(Box::new(root_col))
    }
}

impl Scene for TestScene {
    fn update(&mut self, _input: &InputState) {}

    fn build_passes(&self, renderer: &mut Renderer) {
        if renderer.pass_count() == 0 {
            let params = LayoutParams::default();
            let mut panel = Self::build_ui(&params);

            let mut measurer = GlyphonTextMeasurer::new();
            let screen = Size {
                w: renderer.width() as f32,
                h: renderer.height() as f32,
            };

            let measured = panel.measure(screen, &params, &mut measurer);
            panel.arrange(
                Rect { x: 16.0, y: 16.0, w: measured.w, h: measured.h },
                &params,
                &mut measurer,
            );

            let mut groups = Vec::new();
            collect_text(&panel, &mut groups, &params);

            let mut models = Vec::new();
            collect_rects(&panel, &mut models);
            println!("box models collected: {}", models.len());

            let ui_pass = UiPassBuilder::new(&params).build(
                &panel,
                renderer.width() as f32,
                renderer.height() as f32,
            );
            renderer.add_pass(Box::new(ui_pass));
            renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(groups))));
        }
    }
}


// use base::{prelude::Locale, ui::{
//     layout::{layout_params::LayoutParams, rect::Rect, size::Size},
//     text::{params::TextParams},
//     widget::{Widget, collect_rects, collect_text},
//     widgets::{column::Column, grid::Grid, label::Label, panel::Panel, property_panel::PropertyPanel, row::Row, text_field::TextField},
// }, unit::{UnitSettings, UnitSystem}};
// use impls::examples::model::{ExampleModelConfig, ExampleUnitSettings};
// use crate::{
//     core::test::part::Part,
//     engine::{input::InputState, scene::Scene},
//     renderer::{
//         Renderer, pass::text::{TextRenderPass, measurer::GlyphonTextMeasurer}, ui_pass_builder::UiPassBuilder
//     },
// };

 

// #[derive(Default)]
// pub struct TestScene;

// impl TestScene {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn build_ui(params: &LayoutParams) -> Panel {
//         // Row 1: app/header info
//         //let mut header_row = Row::new();

//         // header_row.push(Box::new(Label::new("Model Lab")));

//         // // Row 2: primary editable fields
//         // let mut input_row = Row::new();
//         // input_row.push(Box::new(Label::new("Name:")));
//         // input_row.push(Box::new(TextField::new("Engine Bolt")));

//         // // Grid: detailed properties (2 columns = label/value)
//         // let mut details_grid = Grid::new(2);
//         // details_grid.push(Box::new(Label::new("Mass")));
//         // details_grid.push(Box::new(TextField::new("1.42 kg")));
//         // details_grid.push(Box::new(Label::new("Tolerance")));
//         // details_grid.push(Box::new(TextField::new("±0.05 mm")));
//         // details_grid.push(Box::new(Label::new("Batch")));
//         // details_grid.push(Box::new(TextField::new("A-104")));
//         // details_grid.push(Box::new(Label::new("Process")));
//         // details_grid.push(Box::new(TextField::new("CNC")));
//         // details_grid.push(Box::new(Label::new("Supplier")));
//         // details_grid.push(Box::new(TextField::new("Acme Industrial")));

//         // PropertyPanel from mock Part object
//         let part = Part::new();
//         let units = UnitSystem::<ExampleModelConfig>::new(ExampleUnitSettings::default());
//         let property_panel = PropertyPanel::<ExampleModelConfig>::new(
//             &part,
//             &units,
//             Locale::EnUs,  
//             params,
//         );
//         // Column containing header, input row, grid, and property panel
//         let mut root_col = Column::new();
//         // root_col.push(Box::new(header_row));
//         // root_col.push(Box::new(input_row));
//         // root_col.push(Box::new(details_grid));
//         root_col.push(Box::new(property_panel.into_column()));

//         // Top-level panel contains the column
//         Panel::new()
//             .with_child(Box::new(root_col))
//     }
// }

// impl Scene for TestScene {
//     fn update(&mut self, _input: &InputState) {}

//     fn build_passes(&self, renderer: &mut Renderer) {
//         if renderer.pass_count() == 0 {
            
//             let params = LayoutParams::default();
//             let mut panel = Self::build_ui(&params);

//             let mut measurer = GlyphonTextMeasurer::new();
//             let screen = Size {
//                 w: renderer.width() as f32,
//                 h: renderer.height() as f32,
//             };

//             let measured = panel.measure(screen, &params, &mut measurer);
//             panel.arrange(
//                 Rect {
//                     x: 16.0,
//                     y: 16.0,
//                     w: measured.w,
//                     h: measured.h,
//                 },
//                 &params,
//                 &mut measurer,
//             );

//             let mut groups = Vec::new();
//             collect_text(&panel, &mut groups, &params);

//             // debug: check box models
//             let mut models = Vec::new();
//             collect_rects(&panel, &mut models);
//             println!("box models collected: {}", models.len());

//             let ui_pass = UiPassBuilder::new(&params).build(
//                 &panel,
//                 renderer.width() as f32,
//                 renderer.height() as f32,
//             );
//             renderer.add_pass(Box::new(ui_pass));
//             renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(groups))));
 


//         }
//     }
// }