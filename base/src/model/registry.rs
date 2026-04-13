use super::{ComponentId, ComponentData};

/// Represents the registry for components, where the 
/// component is stored as ID, Data, rather than the component struct
pub trait ComponentRegistry {
    type Id: ComponentId;
    type Data: ComponentData;

    /// Inserts the data for the component and returns the old data if the component existed.
    fn insert(&mut self, id: Self::Id, data: Self::Data) -> Option<Self::Data>;

    /// Deletes the data for the component and returns the data.
    fn remove(&mut self, id: &Self::Id, kind: <Self::Data as ComponentData>::Kind) -> Option<Self::Data>;

    /// ReadOnly reference.
    fn get(&self, id: &Self::Id, kind: <Self::Data as ComponentData>::Kind) -> Option<&Self::Data>;
    
    /// Mutable reference
    fn get_mut(&mut self, id: &Self::Id, kind: <Self::Data as ComponentData>::Kind) -> Option<&mut Self::Data>;

    fn contains(&self, id: &Self::Id, kind: <Self::Data as ComponentData>::Kind) -> bool;

    /// Get all components as readonly
    fn values(&self) -> impl Iterator<Item = &Self::Data>;

    /// Get all the components as mutable
    fn values_mut(&mut self) -> impl Iterator<Item = &mut Self::Data>;

    
    /// To loop over components by kind
    fn values_by_kind(&self, kind: <Self::Data as ComponentData>::Kind) -> impl Iterator<Item = &Self::Data> {
        self.values().filter(move |d| d.kind() == kind)
    }

    /// To loop over components by kind
    fn values_mut_by_kind(&mut self, kind: <Self::Data as ComponentData>::Kind) -> impl Iterator<Item = &mut Self::Data> {
        self.values_mut().filter(move |d| d.kind() == kind)
    }
    
}
  

use std::collections::HashMap;
use super::{ComponentKey};
pub struct HashMapRegistry<I, D> 
where 
    I: ComponentId, 
    D: ComponentData 
{
    items: HashMap<ComponentKey<I, D::Kind>, D>,
}

impl<I, D> HashMapRegistry<I, D> 
where 
    I: ComponentId, 
    D: ComponentData 
{
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}

impl<I, D> ComponentRegistry for HashMapRegistry<I, D>
where
    I: ComponentId,
    D: ComponentData,
{
    type Id = I;
    type Data = D;

    fn insert(&mut self, id: Self::Id, data: Self::Data) -> Option<Self::Data> {
        let key = ComponentKey { id, kind: data.kind() };
        self.items.insert(key, data)
    }

    fn remove(&mut self, id: &Self::Id, kind: D::Kind) -> Option<Self::Data> {
        let key = ComponentKey { id: id.clone(), kind };
        self.items.remove(&key)
    }

    fn get(&self, id: &Self::Id, kind: D::Kind) -> Option<&Self::Data> {
        let key = ComponentKey { id: id.clone(), kind };
        self.items.get(&key)
    }

    fn get_mut(&mut self, id: &Self::Id, kind: D::Kind) -> Option<&mut Self::Data> {
        let key = ComponentKey { id: id.clone(), kind };
        self.items.get_mut(&key)
    }

    fn contains(&self, id: &Self::Id, kind: D::Kind) -> bool {
        let key = ComponentKey { id: id.clone(), kind };
        self.items.contains_key(&key)
    }

    fn values(&self) -> impl Iterator<Item = &Self::Data> {
        self.items.values()
    }

    fn values_mut(&mut self) -> impl Iterator<Item = &mut Self::Data> {
        self.items.values_mut()
    }
}
