use crate::sim::lifecycle::Creator;

/// Maintains a fixed item count.
pub struct FixedCount {
    target: usize,
}

impl FixedCount {
    pub fn new(target: usize) -> Self { Self { target } }
}

impl Creator for FixedCount {
    fn deficit(&self, current_len: usize) -> usize {
        self.target.saturating_sub(current_len)
    }
    fn excess(&self, current_len: usize) -> usize {
        current_len.saturating_sub(self.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deficit_when_below_target() {
        let c = FixedCount::new(10);
        assert_eq!(c.deficit(7), 3);
    }

    #[test]
    fn deficit_zero_when_at_or_above_target() {
        let c = FixedCount::new(10);
        assert_eq!(c.deficit(10), 0);
        assert_eq!(c.deficit(12), 0);
    }

    #[test]
    fn excess_when_above_target() {
        let c = FixedCount::new(10);
        assert_eq!(c.excess(13), 3);
    }

    #[test]
    fn excess_zero_when_at_or_below_target() {
        let c = FixedCount::new(10);
        assert_eq!(c.excess(10), 0);
        assert_eq!(c.excess(8), 0);
    }
}
