use crate::sim::lifecycle::Creator;


/// Maintains a fixed particle count.
pub struct FixedCountCreator {
    target: usize,
}

impl FixedCountCreator {
    pub fn new(target: usize) -> Self { Self { target } }
}

impl Creator for FixedCountCreator {
    fn deficit(&self, current_len: usize) -> usize {
        if current_len < self.target { self.target - current_len } else { 0 }
    }

    fn excess(&self, current_len: usize) -> usize {
        if current_len > self.target { current_len - self.target } else { 0 }
    }
}