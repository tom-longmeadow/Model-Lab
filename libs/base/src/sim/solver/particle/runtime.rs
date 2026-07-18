use crate::math::Vector;

pub struct RuntimeState<V> 
where 
    V: Vector 
{ 
    pub runtime_jitter: V,   
    pub raw_jitter: [f64; 4],
}

impl<V: Vector> RuntimeState<V> {
    pub fn new() -> Self {
        Self {
            runtime_jitter: V::ZERO,  
            raw_jitter: [0.0; 4], 
        }
    }

    /// Call this once per frame before processing particle constraints.
    /// It uses a golden ratio multiplier to cycle the seed chaotically.
    pub fn update_jitter(&mut self, frame_count: u64) {
        // let seed = frame_count.wrapping_add(0x9E3779B97F4A7C15); 
        // for i in 0..4 {
        //     let mut x = seed.wrapping_add(i as u64).wrapping_mul(0xBF58476D1CE4E5B9);
        //     x = (x ^ (x >> 30)).wrapping_mul(0x94D049BB133111EB);
        //     x = (x ^ (x >> 27)).wrapping_mul(0x7305754198654329);
        //     let raw_float = (x ^ (x >> 31)) as f64 / u64::MAX as f64;
             
        //     self.raw_jitter[i] = (raw_float * 0.02) - 0.01;
        // } 
        // self.runtime_jitter = V::from_f64_array(self.raw_jitter);
    }
}