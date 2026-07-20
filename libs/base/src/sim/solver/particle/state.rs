use crate::math::Vector;
use crate::ui::layout::color::Color;
pub struct State<V> 
where 
    V: Vector 
{ 
    pub runtime_jitter: V,   
    pub raw_jitter: [f64; 4], 
    pub colors: &'static [Color]
}

impl<V: Vector> State<V> {
    pub fn new(colors: &'static [Color]) -> Self {
        Self {
            runtime_jitter: V::ZERO,  
            raw_jitter: [0.0; 4], 
            colors
        }
    }

     #[inline(always)]
    pub fn get_color(&self, percent: f64) -> Color { 
        if self.colors.is_empty() {
            return Color::WHITE; 
        }
        Color::get_color_at_percentage(self.colors, percent)
    }

    /// Call this once per frame before processing particle constraints.
    /// It uses a golden ratio multiplier to cycle the seed chaotically.
    pub fn update_jitter(&mut self, frame_count: u64) {
        let seed = frame_count.wrapping_add(0x9E3779B97F4A7C15); 
        for i in 0..4 {
            let mut x = seed.wrapping_add(i as u64).wrapping_mul(0xBF58476D1CE4E5B9);
            x = (x ^ (x >> 30)).wrapping_mul(0x94D049BB133111EB);
            x = (x ^ (x >> 27)).wrapping_mul(0x7305754198654329);
            let raw_float = (x ^ (x >> 31)) as f64 / u64::MAX as f64;
             
            self.raw_jitter[i] = (raw_float * 0.02) - 0.01;
        } 
        self.runtime_jitter = V::from_f64_array(self.raw_jitter);
    }
}