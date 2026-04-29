use crate::sim::storage::Storage;

/// Contiguous AoS storage only
pub trait AosStorage: Storage {
    fn read(&self)      -> &[Self::Item];
    fn write(&mut self) -> &mut [Self::Item];
}