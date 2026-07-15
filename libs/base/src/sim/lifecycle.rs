pub mod stream_config;
use crate::{sim::storage::Storage};
 
pub trait Lifecycle<S: Storage> { 
    type Bounds;

    fn tick(&mut self, _storage: &mut S, _tick: u64, _bounds: &Self::Bounds);
}



 