use base::ui::{
    text::{font::TextFont, params::TextParams, style::TextStyleFactory},
    widget::{
        Widget, WidgetId,
        layout::{edge_insets::EdgeInsets, rect::Rect, size::Size},
        widgets::{
            column::Column,
            grid::Grid,
            label::Label,
            panel::Panel,
            row::Row,
            text_field::TextField,
        },
    },
};

use crate::{
    engine::{input::InputState, scene::Scene},
    renderer::{
        pass::text::{measurer::GlyphonTextMeasurer, TextRenderPass},
        Renderer,
    },
};

#[derive(Default)]
pub struct TestScene;

impl TestScene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_ui(style: &TextStyleFactory) -> Panel {
        // Header row
        let mut header = Row::new()
            .with_gap(20.0)
            .with_background([40, 44, 52, 255])
            .with_padding(EdgeInsets::all(10.0));

        header.push(Box::new(
            Label::new("Model Lab")
                .with_text_style(style.style(32.0)),
        ));
        header.push(Box::new(
            Label::new("Build: Debug")
                .with_text_style(style.style(32.0)),
        ));

        // Left form column
        let mut form_col = Column::new().with_gap(8.0);

        let mut name_row = Row::new().with_gap(8.0);
        name_row.push(Box::new(
            Label::new("Name").with_text_style(style.style(14.0)),
        ));
        name_row.push(Box::new(
            TextField::new("Engine Bolt")
                .with_text_style(style.style(14.0)),
        ));
        form_col.push(Box::new(name_row));

        let mut size_row = Row::new().with_gap(8.0);
        size_row.push(Box::new(
            Label::new("Size").with_text_style(style.style(14.0)),
        ));
        size_row.push(Box::new(
            TextField::new("24").with_text_style(style.style(14.0)),
        ));
        size_row.push(Box::new(
            Label::new("mm").with_text_style(style.style(14.0)),
        ));
        form_col.push(Box::new(size_row));

        let mut material_row = Row::new().with_gap(8.0);
        material_row.push(Box::new(
            Label::new("Material").with_text_style(style.style(14.0)),
        ));
        material_row.push(Box::new(
            TextField::new("Steel").with_text_style(style.style(14.0)),
        ));
        form_col.push(Box::new(material_row));

        let left_panel = Panel::new()
            .with_padding(EdgeInsets::all(10.0))
            .with_background([28, 31, 38, 255])
            .with_child(Box::new(form_col));

        // Right stats grid (2 columns: key/value)
        let mut stats_grid = Grid::new(2)
            .with_gap(6.0)
            .with_background([28, 31, 38, 255])
            .with_padding(EdgeInsets::all(10.0));

        stats_grid.push(Box::new(
            Label::new("Mass").with_text_style(style.style(13.0)),
        ));
        stats_grid.push(Box::new(
            TextField::new("1.42 kg").with_text_style(style.style(13.0)),
        ));
        stats_grid.push(Box::new(
            Label::new("Tolerance").with_text_style(style.style(13.0)),
        ));
        stats_grid.push(Box::new(
            TextField::new("±0.05 mm").with_text_style(style.style(13.0)),
        ));
        stats_grid.push(Box::new(
            Label::new("Batch").with_text_style(style.style(13.0)),
        ));
        stats_grid.push(Box::new(
            TextField::new( "A-104").with_text_style(style.style(13.0)),
        ));

        let right_panel = Panel::new()
            .with_padding(EdgeInsets::all(10.0))
            .with_background([28, 31, 38, 255])
            .with_child(Box::new(stats_grid));

        // Body row: left + right
        let mut body = Row::new().with_gap(12.0);
        body.push(Box::new(left_panel));
        body.push(Box::new(right_panel));

        // Root column
        let mut root_col = Column::new().with_gap(12.0);
        root_col.push(Box::new(header));
        root_col.push(Box::new(body));

        // Root panel
        Panel::new()
            .with_padding(EdgeInsets::all(12.0))
            .with_background([20, 22, 26, 255])
            .with_child(Box::new(root_col))
    }
}

impl Scene for TestScene {
    fn update(&mut self, _input: &InputState) {}

    fn build_passes(&self, renderer: &mut Renderer) {
        if renderer.pass_count() == 0 {
            let style = TextStyleFactory::new(TextFont::Regular, [220, 220, 220, 255])
                .with_ratio(1.20);

            let mut panel = Self::build_ui(&style);

            let mut measurer = GlyphonTextMeasurer::new();
            let screen = Size {
                w: renderer.width() as f32,
                h: renderer.height() as f32,
            };

            let measured = panel.measure(screen, &mut measurer);
            panel.arrange(
                Rect {
                    x: 16.0,
                    y: 16.0,
                    w: measured.w,
                    h: measured.h,
                },
                &mut measurer,
            );

            let mut groups = Vec::new();
            panel.collect_text(&mut groups);

            renderer.add_pass(Box::new(TextRenderPass::new(TextParams::new(groups))));
        }
    }
}


// use base::ui::{
//     text::{font::TextFont, params::TextParams, style::TextStyleFactory}, 
//     widget::{Widget, WidgetId, layout::{rect::Rect, size::Size}, 
//     widgets::panel::Panel}
// };

// use crate::{
//     engine::{input::InputState, scene::Scene}, 
//     renderer::{
//         Renderer, 
//         pass::text::{TextRenderPass, measurer::GlyphonTextMeasurer}
// }};



// #[derive(Default)]
// pub struct TestScene;

// impl TestScene {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn build_ui(style: &TextStyleFactory) -> Panel {
//         // let mut row = Row::new(WidgetId(1)).with_gap(8.0);

//         // row.push(Box::new(Label::new(
//         //     WidgetId(2),
//         //     "Size",
//         //     style.style(14.0),
//         // )));

//         // row.push(Box::new(TextField::new(
//         //     WidgetId(3),
//         //     "24",
//         //     style.style(14.0),
//         // )));

//         // row.push(Box::new(Label::new(
//         //     WidgetId(4),
//         //     "mm",
//         //     style.style(14.0),
//         // )));

//         Panel::new(WidgetId(0))
//         //     .with_padding(EdgeInsets::all(8.0))
//         //     .with_child(Box::new(row))
//     }
// }

// impl Scene for TestScene {
//     fn update(&mut self, _input: &InputState) {}

//     fn build_passes(&self, renderer: &mut Renderer) {
//         if renderer.pass_count() == 0 {
//             let style = TextStyleFactory::new(TextFont::Regular, [220, 220, 220, 255])
//                 .with_ratio(1.20);

//             let mut panel = Self::build_ui(&style);

//             // layout
//             let mut measurer = GlyphonTextMeasurer::new();
//             let screen = Size {
//                 w: renderer.width() as f32,
//                 h: renderer.height() as f32,
//             };
//             let measured = panel.measure(screen, &mut measurer);
//             panel.arrange(
//                 Rect {
//                     x: 16.0,
//                     y: 16.0,
//                     w: measured.w,
//                     h: measured.h,
//                 },
//                 &mut measurer,
//             );

//             // collect text
//             let mut groups = Vec::new();
//             panel.collect_text(&mut groups);

//             renderer.add_pass(Box::new(TextRenderPass::new(
//                 TextParams::new(groups),
//             )));
//         }
//     }
// }
