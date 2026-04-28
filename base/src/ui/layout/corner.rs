 

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerStyle {
    pub radius: f32,
    pub segments: u32, 
}

impl CornerStyle {
    pub fn new(radius: f32, segments: u32) -> Self {
        Self { radius, segments }
    }

    pub fn none() -> Self {
        Self { radius: 0.0, segments: 0 }
    }
}