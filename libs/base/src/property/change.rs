use std::collections::HashMap;
use crate::prelude::PropertyValue;

/// Represents a single change to a property.
#[derive(Debug, Clone)]
pub struct PropertyChange {
    pub key: u64,
    pub old_value: Option<PropertyValue>,
    pub new_value: PropertyValue,
}

/// A trait for objects that can collect property changes.
pub trait ChangeCollector {
    /// Records a change for a given property key.
    fn record_change(&mut self, key: u64, old_value: Option<PropertyValue>, new_value: PropertyValue);
}

/// A concrete implementation that stores a collection of property changes, indexed by property key.
#[derive(Debug, Clone, Default)]
pub struct ChangeMap {
    changes: HashMap<u64, PropertyChange>,
}

impl ChangeMap {
    /// Creates a new, empty `ChangeSet`.
    pub fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    /// Returns `true` if the change set contains no changes.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Clears the change set, returning all changes as a vector.
    pub fn drain(&mut self) -> Vec<PropertyChange> {
        self.changes.drain().map(|(_, v)| v).collect()
    }

    /// Checks if a change for the given property key has been recorded.
    pub fn contains_key(&self, key: u64) -> bool {
        self.changes.contains_key(&key)
    }
}

impl ChangeCollector for ChangeMap {
    fn record_change(&mut self, key: u64, old_value: Option<PropertyValue>, new_value: PropertyValue) {
        self.changes
            .entry(key)
            .and_modify(|existing| {
                // The new value is updated. 
                // The old_value remains from the first change in the series.
                existing.new_value = new_value.clone();
            })
            .or_insert_with(|| PropertyChange {
                key,
                old_value,
                new_value,
            });
    }
}
