 
pub mod component;
pub mod registry;
pub mod error;
pub mod config;
 
#[cfg(feature = "testing")]
pub mod test_model;

pub use component::*;
pub use registry::*;

 
use crate::prelude::UnitSystem;
use crate::prelude::ModelConfig;
use crate::prelude::ModelError;

 pub type ModelComponent<C> = Component<C, <C as ModelConfig>::Data>;
pub struct Model<C> 
where 
    C: ModelConfig
{
    pub registry: C::Registry, 
    pub settings: UnitSystem<C>,
}

type IdOf<C> = <<<C as ModelConfig>::Data as ComponentData<C>>::Kind as ComponentKind>::Id;

impl<C> Model<C>  
where 
    C: ModelConfig,  
{
    pub fn new(registry: C::Registry, settings: UnitSystem<C>) -> Self {
        Self { registry, settings }
    }

    pub fn insert(&mut self, id: IdOf<C>, data: C::Data) -> Result<(), ModelError<C, C::Data, IdOf<C>>> {
        let kind = data.kind();
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id, kind));
        }  
        if self.registry.contains(&id, kind) {
            return Err(ModelError::AlreadyExists(id, kind));
        }
        self.registry.insert(id, data);
        Ok(())
    }

    pub fn update(&mut self, id: IdOf<C>, data: C::Data) -> Result<(), ModelError<C, C::Data, IdOf<C>>> {
        let kind = data.kind();  
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id, kind));
        }  
        if !self.registry.contains(&id, kind) {
            return Err(ModelError::NotFound(id, kind)); 
        }
        self.registry.insert(id, data);
        Ok(())
    }

    pub fn get(&self, id: IdOf<C>, kind: <C::Data as ComponentData<C>>::Kind) -> Result<&C::Data, ModelError<C, C::Data, IdOf<C>>> {
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id, kind));
        }
        self.registry.get(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
    }

    pub fn get_mut(&mut self, id: IdOf<C>, kind: <C::Data as ComponentData<C>>::Kind) -> Result<&mut C::Data, ModelError<C, C::Data, IdOf<C>>> {
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id, kind));
        }
        self.registry.get_mut(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
    }

    pub fn get_clone(&self, id: IdOf<C>, kind: <C::Data as ComponentData<C>>::Kind) -> Result<C::Data, ModelError<C, C::Data, IdOf<C>>> {
        self.get(id, kind).map(|data| data.clone())
    }

    pub fn delete(&mut self, id: IdOf<C>, kind: <C::Data as ComponentData<C>>::Kind) -> Result<C::Data, ModelError<C, C::Data, IdOf<C>>> {
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id, kind));
        }
        self.registry.remove(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
    }

    // Component wrappers 
    pub fn insert_comp(&mut self, comp: ModelComponent<C>) -> Result<(), ModelError<C, C::Data, IdOf<C>>>  
    {
        self.insert(comp.id, comp.data)
    }

    pub fn update_comp(&mut self, comp: ModelComponent<C>) -> Result<(), ModelError<C, C::Data, IdOf<C>>> {
        self.update(comp.id, comp.data)
    }

    // Iterators (Uncommented and using explicit anonymous lifetimes `'_`)
    pub fn components(&self) -> impl Iterator<Item = &C::Data> + '_ {
        self.registry.values()
    }

    pub fn components_by_kind(&self, kind: <C::Data as ComponentData<C>>::Kind) -> impl Iterator<Item = &C::Data> + '_ {
        self.registry.values_by_kind(kind)
    }

    pub fn components_mut(&mut self) -> impl Iterator<Item = &mut C::Data> + '_ {
        self.registry.values_mut()
    }

    pub fn components_mut_by_kind(&mut self, kind: <C::Data as ComponentData<C>>::Kind) -> impl Iterator<Item = &mut C::Data> + '_ {
        self.registry.values_mut_by_kind(kind)
    }
}


// type IdOf<C> = <<<C as ModelConfig>::Data as ComponentData<C>>::Kind as ComponentKind>::Id;
// impl<C> Model<C>  
// where 
//     C: ModelConfig,  
// {

//     pub fn new(registry: C::Registry, settings: UnitSystem<C>) -> Self {
//         Self { registry, settings }
//     }

//     pub fn insert(&mut self, id: C::Id, data: C::Data) -> Result<(), ModelError<C, C::Data, C::Id>> {
//         let kind = data.kind();
//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id, kind));
//         }  
//         if self.registry.contains(&id, kind) {
//             return Err(ModelError::AlreadyExists(id, kind));
//         }
//         self.registry.insert(id, data);
//         Ok(())
//     }

//     pub fn update(&mut self, id: C::Id, data: C::Data) -> Result<(), ModelError<C, C::Data, C::Id>> {
//         let kind = data.kind();  
//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id, kind));
//         }  
//         if !self.registry.contains(&id, kind) {
//             return Err(ModelError::NotFound(id, kind)); 
//         }
//         self.registry.insert(id, data);
//         Ok(())
//     }

//     pub fn get(&self, id: C::Id, kind: <C::Data as ComponentData<C>>::Kind) -> Result<&C::Data, ModelError<C, C::Data, C::Id>> {
//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id, kind));
//         }
//         self.registry.get(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
//     }

//     pub fn get_mut(&mut self, id: C::Id, kind: <C::Data as ComponentData<C>>::Kind) -> Result<&mut C::Data, ModelError<C, C::Data, C::Id>> {
//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id, kind));
//         }
//         self.registry.get_mut(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
//     }

//     pub fn get_clone(&self, id: C::Id, kind: <C::Data as ComponentData<C>>::Kind) -> Result<C::Data, ModelError<C, C::Data, C::Id>> {
//         self.get(id, kind).map(|data| data.clone())
//     }

//     pub fn delete(&mut self, id: C::Id, kind: <C::Data as ComponentData<C>>::Kind) -> Result<C::Data, ModelError<C, C::Data, C::Id>> {
//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id, kind));
//         }
//         self.registry.remove(&id, kind).ok_or_else(|| ModelError::NotFound(id, kind))
//     }

//     // Component wrappers 
//     pub fn insert_comp(&mut self, comp: ModelComponent<C>) -> Result<(), ModelError<C, C::Data, C::Id>>  
//     {
//         self.insert(comp.id, comp.data)
//     }

//     pub fn update_comp(&mut self, comp: ModelComponent<C>) -> Result<(), ModelError<C, C::Data, C::Id>> {
//         self.update(comp.id, comp.data)
//     }

//     // Iterators
//     pub fn components(&self) -> impl Iterator<Item = &C::Data> {
//         self.registry.values()
//     }

//     pub fn components_by_kind(&self, kind: <C::Data as ComponentData<C>>::Kind) -> impl Iterator<Item = &C::Data> {
//         self.registry.values_by_kind(kind)
//     }

//     pub fn components_mut(&mut self) -> impl Iterator<Item = &mut C::Data> {
//         self.registry.values_mut()
//     }

//     pub fn components_mut_by_kind(&mut self, kind: <C::Data as ComponentData<C>>::Kind) -> impl Iterator<Item = &mut C::Data> {
//         self.registry.values_mut_by_kind(kind)
//     }

//      // Iterators
//     // pub fn components(&self) -> impl Iterator<Item = &C::Data> + '_ {
//     //     self.registry.values()
//     // }

//     // pub fn components_by_kind(&self, kind: <C::Data as ComponentData<C>>::Kind) -> impl Iterator<Item = &C::Data> + '_ {
//     //     self.registry.values_by_kind(kind)
//     // }

//     // pub fn components_mut(&mut self) -> impl Iterator<Item = &mut C::Data> + '_ {
//     //     self.registry.values_mut()
//     // }

//     // pub fn components_mut_by_kind(&mut self, kind: <C::Data as ComponentData<C>>::Kind) -> impl Iterator<Item = &mut C::Data> + '_ {
//     //     self.registry.values_mut_by_kind(kind)
//     // }
// }
