 
pub mod component;
pub mod registry;
pub mod error;
pub mod config;
 
#[cfg(feature = "testing")]
pub mod test;

pub use component::*;
pub use registry::*;

 
pub use crate::prelude::UnitSystem;
pub use crate::prelude::ModelConfig;
pub use crate::prelude::ModelError;


pub struct Model<C> 
where 
    C: ModelConfig
{
    pub registry: C::Registry, 
    pub settings: UnitSystem<C>,
}
type ID<C> = <<C as ModelConfig>::Kind as ComponentKind>::Id;
type ModelComponent<C> = Component<C, <C as ModelConfig>::Data>;

impl<C> Model<C>
where
    C: ModelConfig,
{
    pub fn new(registry: C::Registry, settings: UnitSystem<C>) -> Self {
        Self { registry, settings }
    }

    pub fn insert(&mut self, id: ID<C>, data: C::Data) -> Result<(), ModelError<C>> {
        let kind = data.kind();
        if id.is_invalid() { return Err(ModelError::InvalidId(id, kind)); }
        if self.registry.contains(&id, kind) { return Err(ModelError::AlreadyExists(id, kind)); }
        self.registry.insert(id, data);
        Ok(())
    }

    pub fn update(&mut self, id: ID<C>, data: C::Data) -> Result<(), ModelError<C>> {
        let kind = data.kind();
        if id.is_invalid() { return Err(ModelError::InvalidId(id, kind)); }
        if !self.registry.contains(&id, kind) { return Err(ModelError::NotFound(id, kind)); }
        self.registry.insert(id, data);
        Ok(())
    }

    pub fn get(&self, id: ID<C>, kind: C::Kind) -> Result<&C::Data, ModelError<C>> {
        if id.is_invalid() { return Err(ModelError::InvalidId(id, kind)); }
        self.registry.get(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
    }

    pub fn get_mut(&mut self, id: ID<C>, kind: C::Kind) -> Result<&mut C::Data, ModelError<C>> {
        if id.is_invalid() { return Err(ModelError::InvalidId(id, kind)); }
        self.registry.get_mut(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
    }

    pub fn get_clone(&self, id: ID<C>, kind: C::Kind) -> Result<C::Data, ModelError<C>> {
        self.get(id, kind).cloned()
    }

    pub fn delete(&mut self, id: ID<C>, kind: C::Kind) -> Result<C::Data, ModelError<C>> {
        if id.is_invalid() { return Err(ModelError::InvalidId(id, kind)); }
        self.registry.remove(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
    }

    pub fn insert_comp(&mut self, comp: ModelComponent<C>) -> Result<(), ModelError<C>> {
        self.insert(comp.id, comp.data)
    }

    pub fn update_comp(&mut self, comp: ModelComponent<C>) -> Result<(), ModelError<C>> {
        self.update(comp.id, comp.data)
    }

    pub fn components(&self) -> impl Iterator<Item = &C::Data> + '_ {
        self.registry.values()
    }

    pub fn components_by_kind(&self, kind: C::Kind) -> impl Iterator<Item = &C::Data> + '_ {
        self.registry.values_by_kind(kind)
    }

    pub fn components_mut(&mut self) -> impl Iterator<Item = &mut C::Data> + '_ {
        self.registry.values_mut()
    }

    pub fn components_mut_by_kind(&mut self, kind: C::Kind) -> impl Iterator<Item = &mut C::Data> + '_ {
        self.registry.values_mut_by_kind(kind)
    }
}

 