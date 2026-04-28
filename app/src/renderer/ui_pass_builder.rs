use base::ui::{layout::layout_params::LayoutParams, widget::{Widget, collect_rects}, widget_mesh};

use crate::renderer::pass::ui::UiMeshRenderPass;

pub struct UiPassBuilder<'a> {
    params: &'a LayoutParams,
}

impl<'a> UiPassBuilder<'a> {
    pub fn new(params: &'a LayoutParams) -> Self {
        Self { params }
    }

    pub fn build(&self, root: &dyn Widget, screen_width: f32, screen_height: f32) -> UiMeshRenderPass {
        let mut models = Vec::new();
        collect_rects(root, &mut models);
        println!("UiPassBuilder — box models: {}", models.len());

        let batcher = widget_mesh::build(&models, self.params);
        let meshes = batcher.finish();
        println!("UiPassBuilder — meshes: {}", meshes.len());
        for m in &meshes {
            println!("  kind: {:?}  verts: {}  indices: {}", m.mesh_type, m.vertices.len(), m.indices.len());
        }

        UiMeshRenderPass::new(meshes, screen_width, screen_height)
    }
}