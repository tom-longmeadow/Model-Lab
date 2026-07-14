use base::{ 
    property::change::{
        ChangeMap, PropertyChange
    }, 
    ui::{
        layout::{
            layout_params::LayoutParams, 
            rect::Rect, size::Size, 
            text_measurer::TextMeasurer 
        }, widget::Widget
    }
};
 

pub struct Gui {
    root: Box<dyn Widget>,           // Top-level widget tree
    changes: ChangeMap,               // Collects property mutations
    params: LayoutParams,             // Shared layout/styling config 
}

impl Gui {
    pub fn new(root: Box<dyn Widget>) -> Self {
        Self {
            root,
            changes: ChangeMap::new(),
            params: LayoutParams::default()
        }
    }

    pub fn get_root(&mut self) -> &dyn Widget {
        self.root.as_ref()
    }
    
    pub fn layout(&mut self, screen: Size, measurer: &mut dyn TextMeasurer) {
        let measured = self.root.measure(screen, &self.params, measurer);
        self.root.arrange(
            Rect { x: 0.0, y: 0.0, w: measured.w, h: measured.h },
            &self.params,
            measurer,
        );
    }
    
    pub fn drain_changes(&mut self) -> Vec<PropertyChange> {
        self.changes.drain()
    }
}
 