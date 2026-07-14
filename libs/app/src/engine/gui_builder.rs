use base::{mesh::Mesh, ui::{layout::{layout_params::LayoutParams, size::Size}, 
text::params::{TextGroup, TextParams}, widget::{Widget, collect_text}}};

use crate::{engine::gui::Gui, graphics_context::{GraphicsContext, 
    pass::{RenderPass}, renderer::{mesh::MeshRenderer, text::TextRenderer}}, 
    ui::{mesh_builder::UiMeshBuilder, text_measurer::GlyphonTextMeasurer}};



/// The result of building a GUI, containing the Gui instance and its render passes.
pub struct GuiBuilderResult {
    pub gui: Gui,
    pub mesh_pass: RenderPass<MeshRenderer>,
    pub text_pass: RenderPass<TextRenderer>,
}

pub struct GuiBuilder;

impl GuiBuilder {
    /// Builds a `Gui` from a root widget, performs layout, and creates the necessary render passes.
    pub fn build(
        root: Box<dyn Widget>,
        renderer: &mut GraphicsContext,
    ) -> GuiBuilderResult {
        // --- 1. Create Gui and perform layout ---
        let mut gui = Gui::new(root);
        let mut measurer = GlyphonTextMeasurer::new();
        let screen = Size {
            w: renderer.width() as f32,
            h: renderer.height() as f32,
        };
        gui.layout(screen, &mut measurer);

        // --- 2. Collect renderable data from the widget tree ---
        let layout_params = LayoutParams::default();

        // Collect mesh data
        let ui_mesh_data: Vec<Mesh> =
            UiMeshBuilder::new(&layout_params).build(gui.get_root());

        // Collect text data
        let mut text_groups: Vec<TextGroup> = Vec::new();
        collect_text(gui.get_root(), &mut text_groups, &layout_params);
        let ui_text_data = TextParams {
            groups: text_groups,
        };

        // --- 3. Create concrete render passes ---
        let mesh_renderer = MeshRenderer::new(ui_mesh_data);
        let mesh_pass = RenderPass::new(mesh_renderer);

        let text_renderer = TextRenderer::new(ui_text_data);
        let text_pass = RenderPass::new(text_renderer);

        // --- 4. Return the structured result ---
        GuiBuilderResult {
            gui,
            mesh_pass,
            text_pass,
        }
    }
}