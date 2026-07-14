use crate::ui::layout::edge_insets::EdgeInsets;


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn inset(self, padding: EdgeInsets) -> Self {
        Self {
            x: self.x + padding.left,
            y: self.y + padding.top,
            w: (self.w - padding.left - padding.right).max(0.0),
            h: (self.h - padding.top - padding.bottom).max(0.0),
        }
    }
}