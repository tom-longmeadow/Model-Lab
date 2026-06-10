use crate::sim::{
    lifecycle::{Creator, Deletor, Lifecycle},
    storage::{SoaCpuStorage, SoaLayout},
};

/// Orchestrates a `Creator`, a `Deletor`, and a `spawn_fn` for SoA CPU storage.
/// The `spawn_fn` returns a layout value (not stored as a struct), which is
/// immediately decomposed into columns via `push_cols`.
pub struct SoaLifecycle<S, C, D, SpawnFn, ScoreFn>
where
    S: SoaCpuStorage,
    C: Creator,
    D: Deletor,
    SpawnFn: Fn() -> S::Layout,
    ScoreFn: Fn(&S) -> Vec<f64>,
{
    creator:  C,
    deletor:  D,
    spawn_fn: SpawnFn,
    score_fn: ScoreFn,
    _marker:  std::marker::PhantomData<S>,
}

impl<S, C, D, SpawnFn, ScoreFn> SoaLifecycle<S, C, D, SpawnFn, ScoreFn>
where
    S: SoaCpuStorage,
    C: Creator,
    D: Deletor,
    SpawnFn: Fn() -> S::Layout,
    ScoreFn: Fn(&S) -> Vec<f64>,
{
    pub fn new(creator: C, deletor: D, spawn_fn: SpawnFn, score_fn: ScoreFn) -> Self {
        Self { creator, deletor, spawn_fn, score_fn, _marker: std::marker::PhantomData }
    }
}

impl<S, C, D, SpawnFn, ScoreFn> Lifecycle<S> for SoaLifecycle<S, C, D, SpawnFn, ScoreFn>
where
    S: SoaCpuStorage,
    C: Creator,
    D: Deletor,
    SpawnFn: Fn() -> S::Layout,
    ScoreFn: Fn(&S) -> Vec<f64>,
{
    fn tick(&mut self, storage: &mut S, _tick: u64) {
        let excess = self.creator.excess(storage.len());
        if excess > 0 {
            let scores  = (self.score_fn)(storage);
            let indices = self.deletor.select(&scores, excess);
            storage.remove_indices(indices);
        }

        let deficit = self.creator.deficit(storage.len());
        for _ in 0..deficit {
            let layout_value = (self.spawn_fn)();
            layout_value.push_cols(storage.columns_mut());
            storage.increment_len();
        }
    }
}