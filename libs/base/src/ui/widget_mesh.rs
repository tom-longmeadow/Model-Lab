use crate::{
    mesh::{
        batch::MeshBatcher,
        ui::{border, quad},
    },
    ui::{
        layout::layout_params::LayoutParams,
        widget::WidgetBase,
    },
};

pub fn build(bases: &[WidgetBase], params: &LayoutParams) -> MeshBatcher {
    let mut batcher = MeshBatcher::new();
    for base in bases {
        collect_base_mesh(base, params, &mut batcher);
    }
    batcher
}

fn collect_base_mesh(base: &WidgetBase, params: &LayoutParams, batcher: &mut MeshBatcher) {
    let rect  = base.rect();
    let style = params.control.style_for(base.kind());

    if style.background.is_visible() {
        batcher.push(&quad::quad(rect, style.background, style.corner));
    }

    if let Some(mesh) = border::border(rect, style.border, style.corner) {
        batcher.push(&mesh);
    }
}