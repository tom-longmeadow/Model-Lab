use crate::sim::{solver::StepModel, storage::Storage};



/// No-op [`StepModel`] — use when no behavior is needed.
pub struct NoModel;
impl<S: Storage> StepModel<S> for NoModel {}

/// Chains two [`StepModel`]s — zero allocation, fully inlined.
/// Use the [`chain!`] macro for more than two.
pub struct ModelChain<A, B>(pub A, pub B);
impl<S: Storage, A: StepModel<S>, B: StepModel<S>> StepModel<S> for ModelChain<A, B> {
    #[inline(always)]
    fn pre(&mut self, storage: &mut S, dt: f64) {
        self.0.pre(storage, dt);
        self.1.pre(storage, dt);
    }
    #[inline(always)]
    fn post(&mut self, storage: &mut S, dt: f64) {
        self.0.post(storage, dt);
        self.1.post(storage, dt);
    }
}

//// Chains any number of [`StepModel`]s into a [`ModelChain`] — zero allocation, fully inlined.
/// ```ignore
/// let model = chain!(ClearAcc, Gravity::new(9.81), Drag::new(0.01));
/// ```
#[macro_export] 
macro_rules! chain {
    ($a:expr) => { $a };
    ($a:expr, $($rest:expr),+) => {
        $crate::sim::solver::ModelChain($a, $crate::chain!($($rest),+))
    };
}
