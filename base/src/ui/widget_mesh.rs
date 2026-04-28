use crate::{
    mesh::{
        batch::MeshBatcher,
        ui::{border, quad},
    }, ui::{layout::layout_params::LayoutParams, widget::{WidgetBase, WidgetRole}},
     
};

pub fn build(bases: &[WidgetBase], params: &LayoutParams) -> MeshBatcher {
    let mut batcher = MeshBatcher::new();
    for base in bases {
        collect_base_mesh(base, params, &mut batcher);
    }
    batcher
}

fn collect_base_mesh(base: &WidgetBase, params: &LayoutParams, batcher: &mut MeshBatcher) {
    let rect = base.rect();
    let role = base.role();

    let background = base.resolved_background(params.background_for(role));
    let border_style = base.resolved_border(params.border_for(role));
    let corner = params.corner_for(role);

    if background.is_visible() {
        batcher.push(&quad::quad(rect, background, corner));
    }

    if let Some(mesh) = border::border(rect, border_style, corner) {
        batcher.push(&mesh);
    }
}