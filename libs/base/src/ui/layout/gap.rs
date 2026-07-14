
 
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    pub horizontal: f32,
    pub vertical:   f32,
}

impl Gap {
    pub fn new(horizontal: f32, vertical: f32) -> Self {
        Self { horizontal, vertical }
    }

    pub fn all(value: f32) -> Self {
        Self { horizontal: value, vertical: value }
    }

    pub fn none() -> Self {
        Self { horizontal: 0.0, vertical: 0.0 }
    }
}
 