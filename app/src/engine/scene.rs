use crate::{
    engine::input::InputState, 
    renderer::{Renderer, pass::text::TextRenderPass}
};



pub trait Scene {
    fn update(&mut self, input: &InputState);
    fn build_passes(&self, renderer: &mut Renderer);
}


#[derive(Default)]
pub struct TestScene;

impl TestScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Scene for TestScene {
    fn update(&mut self, _input: &InputState) {}

    fn build_passes(&self, renderer: &mut Renderer) {
        if renderer.pass_count() == 0 {
            renderer.add_pass(Box::new(TextRenderPass::new("Hello from TestApp")));
        }
    }
}