use crate::sim::{
    lifecycle::{Creator, Deletor, Lifecycle},
    storage::AosCpuStorage,
};

/// Orchestrates a `Creator`, a `Deletor`, and a `spawn_fn` for AoS CPU storage.
/// Owns the `score_fn` because scoring IS storage and entity-specific.
/// `Creator` and `Deletor` are both completely storage-agnostic.
pub struct AosLifecycle<S, C, D, SpawnFn, ScoreFn>
where
    S: AosCpuStorage,
    C: Creator,
    D: Deletor,
    SpawnFn: Fn() -> S::Item,
    ScoreFn: Fn(&S) -> Vec<f64>,
{
    creator:  C,
    deletor:  D,
    spawn_fn: SpawnFn,
    score_fn: ScoreFn,
    _marker:  std::marker::PhantomData<S>,
}

impl<S, C, D, SpawnFn, ScoreFn> AosLifecycle<S, C, D, SpawnFn, ScoreFn>
where
    S: AosCpuStorage,
    C: Creator,
    D: Deletor,
    SpawnFn: Fn() -> S::Item,
    ScoreFn: Fn(&S) -> Vec<f64>,
{
    pub fn new(creator: C, deletor: D, spawn_fn: SpawnFn, score_fn: ScoreFn) -> Self {
        Self { creator, deletor, spawn_fn, score_fn, _marker: std::marker::PhantomData }
    }
}

impl<S, C, D, SpawnFn, ScoreFn> Lifecycle<S> for AosLifecycle<S, C, D, SpawnFn, ScoreFn>
where
    S: AosCpuStorage,
    C: Creator,
    D: Deletor,
    SpawnFn: Fn() -> S::Item,
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
            storage.push((self.spawn_fn)());
        }
    }
}