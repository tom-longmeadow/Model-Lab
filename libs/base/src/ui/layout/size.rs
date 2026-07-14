use crate::ui::layout::edge_insets::EdgeInsets;

#[derive(Clone, Copy, Debug, Default)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}


impl Size {

    pub fn zero() -> Self {
        Self {
            w: 0.0,
            h: 0.0,
        }
    }


    pub fn shrink(self, padding: EdgeInsets) -> Self {
        Self {
            w: (self.w - padding.left - padding.right).max(0.0),
            h: (self.h - padding.top - padding.bottom).max(0.0),
        }
    }

    pub fn grow(self, padding: EdgeInsets) -> Self {
        Self {
            w: self.w + padding.left + padding.right,
            h: self.h + padding.top + padding.bottom,
        }
    }
}