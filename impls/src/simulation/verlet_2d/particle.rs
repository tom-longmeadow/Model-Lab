use base::{math::DVec2, ui::layout::color::Color};

#[repr(C)]
#[derive(Debug, Clone, Copy)] 
pub struct Particle {
    pub pos:     DVec2,
    pub pos_old: DVec2,
    pub acc:     DVec2,
    pub radius: f64,
    pub color: Color
}

impl Particle {

    pub fn new<V>(pos: V) -> Self 
    where 
        V: Into<DVec2> 
    {
        let p = pos.into();
        Self {
            pos: p,  
            pos_old: p,
            ..Default::default()  
        }
    }

    pub fn with_velocity<V>(mut self, vel: V) -> Self 
    where 
        V: Into<DVec2> 
    {
        self.pos_old = self.pos - vel.into();
        self
    }

    pub fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
 
    
}

impl Default for Particle {
     fn default() -> Self {
        Self {
            pos: DVec2::ZERO,
            pos_old: DVec2::ZERO,
            acc: DVec2::ZERO,
            radius: 0.0,  
            color: Color::WHITE,
        }
    }
}