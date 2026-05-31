use crate::sim::{solver::StepModel, storage::Storage};

/// No-op [`StepModel`] — use when no behavior is needed.
pub struct NoModel;
impl<S: Storage> StepModel<S> for NoModel {}

/// Set `acc = 0.0` for one component of one particle.
/// Always apply first in a force accumulation chain.
/// Works for any integrator — only touches `acc`.
pub struct ClearAcc;
impl ClearAcc {
    #[inline(always)]
    pub fn apply(acc: &mut f64) { *acc = 0.0; }
}

/// Add a constant to `acc` for one component.
/// Works for any integrator — only touches `acc`.
pub struct ConstantAccel {
    pub value: f64,
}
impl ConstantAccel {
    pub fn new(value: f64) -> Self { Self { value } }
    #[inline(always)]
    pub fn apply(&self, acc: &mut f64) { *acc += self.value; }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_acc_zeroes_any_value() {
        let mut acc = 42.0;
        ClearAcc::apply(&mut acc);
        assert_eq!(acc, 0.0);
    }

    #[test]
    fn clear_acc_zeroes_negative() {
        let mut acc = -7.5;
        ClearAcc::apply(&mut acc);
        assert_eq!(acc, 0.0);
    }

    #[test]
    fn constant_accel_adds_value() {
        let model = ConstantAccel::new(9.8);
        let mut acc = 0.0;
        model.apply(&mut acc);
        assert!((acc - 9.8).abs() < 1e-12);
    }

    #[test]
    fn constant_accel_accumulates_onto_existing() {
        let model = ConstantAccel::new(1.0);
        let mut acc = 5.0;
        model.apply(&mut acc);
        assert!((acc - 6.0).abs() < 1e-12);
    }

    #[test]
    fn clear_then_constant_accel() {
        let mut acc = 999.0;
        ClearAcc::apply(&mut acc);
        ConstantAccel::new(-9.8).apply(&mut acc);
        assert!((acc - (-9.8)).abs() < 1e-12);
    }
}
