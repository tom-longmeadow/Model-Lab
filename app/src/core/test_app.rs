use base::ui::text::{
    font::TextFont,
    item::TextItem,
    params::{TextParam, TextParams},
    style::TextStyleFactory,
};

use crate::{
    core::app_logic::AppLogic,
    engine::{input::InputState, scene::Scene},
    renderer::{config::RendererConfig, pass::text::TextRenderPass, Renderer},
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
            clear_color: wgpu::Color {
                r: 1.0,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
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
            let base = TextStyleFactory::new(TextFont::Regular, [220, 220, 220, 255]).with_ratio(1.20);
            let header = TextStyleFactory::new(TextFont::Bold, [255, 255, 255, 255]).with_ratio(1.20);
            let italic = TextStyleFactory::new(TextFont::Italic, [180, 180, 255, 255]).with_ratio(1.20);

            let groups = vec![
                TextParam::new(
                    header.style(32.0),
                    vec![TextItem {
                        text: "Header".into(),
                        x: 16.0,
                        y: 20.0,
                    }],
                ),
                TextParam::new(
                    header.style(24.0),
                    vec![TextItem {
                        text: "A1".into(),
                        x: 16.0,
                        y: 60.0,
                    }],
                ),
                TextParam::new(
                    base.style(14.0),
                    vec![TextItem {
                        text: "B1".into(),
                        x: 120.0,
                        y: 60.0,
                    }],
                ),
                TextParam::new(
                    italic.style(64.0),
                    vec![TextItem {
                        text: "italic note".into(),
                        x: 16.0,
                        y: 100.0,
                    }],
                ),
            ];

            let params = TextParams::new(groups);
            renderer.add_pass(Box::new(TextRenderPass::new(params)));
        }
    }
}
