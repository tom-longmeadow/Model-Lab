use base::{
    mesh::Mesh,
    ui::{
        layout::layout_params::LayoutParams,
        widget::{collect_rects, Widget},
        widget_mesh,
    },
};

pub struct UiMeshBuilder<'a> {
    params: &'a LayoutParams,
}

impl<'a> UiMeshBuilder<'a> {
    pub fn new(params: &'a LayoutParams) -> Self {
        Self { params }
    }

    /// Builds a vector of `Mesh` objects from a UI widget tree.
    /// This is the data source for the `MeshRenderer`.
    pub fn build(&self, root: &dyn Widget) -> Vec<Mesh> {
        let mut models = Vec::new();
        collect_rects(root, &mut models);
        println!("UiMeshBuilder — box models: {}", models.len());

        let batcher = widget_mesh::build(&models, self.params);
        let meshes = batcher.finish();
        println!("UiMeshBuilder — meshes: {}", meshes.len());
        for m in &meshes {
            println!(
                "  kind: {:?}  verts: {}  indices: {}",
                m.kind,
                m.vertices.len(),
                m.indices.len()
            );
        }

        // The builder's job ends here. It returns the raw mesh data.
        meshes
    }
}

// use base::ui::{layout::layout_params::LayoutParams, widget::{Widget, collect_rects}, widget_mesh};

 

// pub struct UiMeshBuilder<'a> {
//     params: &'a LayoutParams,
// }

// impl<'a> UiMeshBuilder<'a> {
//     pub fn new(params: &'a LayoutParams) -> Self {
//         Self { params }
//     }

//     pub fn build(&self, root: &dyn Widget, screen_width: f32, screen_height: f32) -> UiMeshRenderPass {
//         let mut models = Vec::new();
//         collect_rects(root, &mut models);
//         println!("UiMeshBuilder — box models: {}", models.len());

//         let batcher = widget_mesh::build(&models, self.params);
//         let meshes = batcher.finish();
//         println!("UiMeshBuilder — meshes: {}", meshes.len());
//         for m in &meshes {
//             println!("  kind: {:?}  verts: {}  indices: {}", m.mesh_type, m.vertices.len(), m.indices.len());
//         }

//         UiMeshRenderPass::new(meshes, screen_width, screen_height)
//     }
// }