use std::collections::HashMap;
use base::prelude::{
    ComponentKey, ComponentKind, ComponentRegistry, 
    HasKind, ModelConfig, 
};

pub struct HashMapRegistry<C: ModelConfig> {
    items: HashMap<ComponentKey<<C::Data as HasKind>::Kind>, C::Data>,
    _config: std::marker::PhantomData<C>,
}

impl<C: ModelConfig> HashMapRegistry<C> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            _config: std::marker::PhantomData,
        }
    }
}

impl<C: ModelConfig> Default for HashMapRegistry<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: ModelConfig> ComponentRegistry<C> for HashMapRegistry<C> {
    type Id = <C::Kind as ComponentKind>::Id;
    type Data = C::Data;

    fn insert(&mut self, id: Self::Id, data: Self::Data) -> Option<Self::Data> {
        let key = ComponentKey { id, kind: data.kind() };
        self.items.insert(key, data)
    }

    fn remove(&mut self, id: &Self::Id, kind: C::Kind) -> Option<Self::Data> {
        let key = ComponentKey { id: *id, kind };
        self.items.remove(&key)
    }

    fn get(&self, id: &Self::Id, kind: C::Kind) -> Option<&Self::Data> {
        let key = ComponentKey { id: *id, kind };
        self.items.get(&key)
    }

    fn get_mut(&mut self, id: &Self::Id, kind: C::Kind) -> Option<&mut Self::Data> {
        let key = ComponentKey { id: *id, kind };
        self.items.get_mut(&key)
    }

    fn contains(&self, id: &Self::Id, kind: C::Kind) -> bool {
        let key = ComponentKey { id: *id, kind };
        self.items.contains_key(&key)
    }

    fn values(&self) -> impl Iterator<Item = &Self::Data> + '_ {
        self.items.values()
    }

    fn values_mut(&mut self) -> impl Iterator<Item = &mut Self::Data> + '_ {
        self.items.values_mut()
    }

    fn values_by_kind(&self, kind: C::Kind) -> impl Iterator<Item = &Self::Data> + '_ {
        self.items.iter()
            .filter(move |(k, _)| k.kind == kind)
            .map(|(_, v)| v)
    }

    fn values_mut_by_kind(&mut self, kind: C::Kind) -> impl Iterator<Item = &mut Self::Data> + '_ {
        self.items.iter_mut()
            .filter(move |(k, _)| k.kind == kind)
            .map(|(_, v)| v)
    }
}
 

 #[cfg(test)]
mod tests { 
    use super::*;
    base::test_registry!(HashMapRegistry);
    
    // Test the high-level API
    base::test_model!(HashMapRegistry);
}