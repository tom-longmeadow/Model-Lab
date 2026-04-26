use crate::{
    core::app_logic::AppLogic,
    engine::{input::InputState, scene::Scene},
    renderer::{
        config::RendererConfig,
        pass::text::{
            font::TextFont,
            item::TextItem,
            params::TextParams,
            style::{TextStyleFactory},
            TextRenderPass,
        },
        Renderer,
    },
};

pub struct TestApp;

impl TestApp {
    pub fn new() -> Self {
        Self
    }
}

impl AppLogic for TestApp {
    fn create_scene(&self) -> Box<dyn Scene> {
        Box::new(TestScene::new())
    }

    fn create_config(&self) -> RendererConfig {
        RendererConfig {
            clear_color: wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            ..RendererConfig::default()
        }
    }

    fn title(&self) -> &str {
        "Test App"
    }
}

#[derive(Default)]
struct TestScene;

impl TestScene {
    fn new() -> Self {
        Self::default()
    }
}

impl Scene for TestScene {
    fn update(&mut self, _input: &InputState) {}

    fn build_passes(&self, renderer: &mut Renderer) {
        if renderer.pass_count() == 0 {

            // base factory — regular weight, light grey, spreadsheet ratio
            let base = TextStyleFactory::new(
                TextFont::Regular, 
                [220, 220, 220, 255])
                .with_ratio(1.20);

            // header factory — bold, white
            let header = TextStyleFactory::new(
                TextFont::Bold, 
                [255, 255, 255, 255])
                .with_ratio(1.20);

            let params = TextParams {
                default_style: base.style(14.0),
                items: vec![
                    TextItem {
                        text: "Header".into(),
                        x: 16.0,
                        y: 20.0,
                        style: Some(header.style(32.0)),
                    },
                    TextItem {
                        text: "A1".into(),
                        x: 16.0,
                        y: 60.0,
                        style: Some(header.style(24.0)),
                    },
                    TextItem {
                        text: "B1".into(),
                        x: 120.0,
                        y: 60.0,
                        style: None,
                    },
                    TextItem {
                        text: "italic note".into(),
                        x: 16.0,
                        y: 100.0,
                        style: Some(
                            TextStyleFactory::new(TextFont::Italic, [180, 180, 255, 255])
                                .with_ratio(1.20)
                                .style(64.0),
                        ),
                    },
                ],
            };

            renderer.add_pass(Box::new(TextRenderPass::new(params)));
        }
    }
}