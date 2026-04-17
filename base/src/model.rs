// 1. Declare the sub-modules (files/folders inside src/model/)
pub mod component;
pub mod registry;
pub mod error;

// 3. Handle feature-gated modules
#[cfg(feature = "testing")]
pub mod test_model;

pub use component::*;
pub use registry::*;
pub use error::*;

use crate::unit::{UnitCategory, UnitSetting, UnitSettings};

 
pub trait ModelConfig: 'static {
    // Data Types
    type Id: ComponentId;
    type Data: ComponentData;
    
    // Storage Type (Linked to Data Types)
    type Registry: ComponentRegistry<Id = Self::Id, Data = Self::Data>;

    // Unit Types
    type Category: UnitCategory;
    type Setting: UnitSetting<Self::Category>;
}
 
pub struct Model<C> 
where 
    C: ModelConfig
{
    pub registry: C::Registry, 
    pub settings: UnitSettings<C>,
}

impl<C: ModelConfig> Model<C> {
    pub fn new(registry: C::Registry, settings: UnitSettings<C>) -> Self {
        Self { registry, settings }
    }

    pub fn insert(&mut self, id: C::Id, data: C::Data) -> Result<(), ModelError<C::Data, C::Id>> {
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

    pub fn insert_comp(&mut self, comp: Component<C::Id, C::Data>) -> Result<(), ModelError<C::Data, C::Id>> {
        self.insert(comp.id, comp.data)
    }

    pub fn update(&mut self, id: &C::Id, data: C::Data) -> Result<(), ModelError<C::Data, C::Id>> {
        let kind = data.kind();  
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id.clone(), kind));
        }  
        if !self.registry.contains(id, kind) {
            return Err(ModelError::NotFound(id.clone(), kind)); 
        }
        self.registry.insert(id.clone(), data);
        Ok(())
    }

    pub fn update_comp(&mut self, comp: Component<C::Id, C::Data>) -> Result<(), ModelError<C::Data, C::Id>> {
        self.update(&comp.id, comp.data)
    }

    pub fn get(&self, id: &C::Id, kind: <C::Data as ComponentData>::Kind) -> Result<&C::Data, ModelError<C::Data, C::Id>> {
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id.clone(), kind));
        }
        self.registry.get(id, kind).ok_or_else(|| ModelError::NotFound(id.clone(), kind))
    }

    pub fn get_comp(&self, comp: &Component<C::Id, C::Data>) -> Result<&C::Data, ModelError<C::Data, C::Id>> {
        self.get(&comp.id, comp.data.kind())
    }

    pub fn get_mut(&mut self, id: &C::Id, kind: <C::Data as ComponentData>::Kind) -> Result<&mut C::Data, ModelError<C::Data, C::Id>> {
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id.clone(), kind));
        }
        self.registry.get_mut(id, kind).ok_or_else(|| ModelError::NotFound(id.clone(), kind))
    }

    pub fn get_mut_comp(&mut self, comp: &Component<C::Id, C::Data>) -> Result<&mut C::Data, ModelError<C::Data, C::Id>> {
        let kind = comp.data.kind();
        self.get_mut(&comp.id, kind)
    }

    pub fn get_clone(&self, id: &C::Id, kind: <C::Data as ComponentData>::Kind) -> Result<C::Data, ModelError<C::Data, C::Id>> {
        self.get(id, kind).map(|data| data.clone())
    }

    pub fn get_clone_comp(&self, comp: &Component<C::Id, C::Data>) -> Result<C::Data, ModelError<C::Data, C::Id>> {
        self.get_clone(&comp.id, comp.data.kind())
    }

    pub fn delete(&mut self, id: &C::Id, kind: <C::Data as ComponentData>::Kind) -> Result<C::Data, ModelError<C::Data, C::Id>> {
        if id.is_invalid() {
            return Err(ModelError::InvalidId(id.clone(), kind));
        }
        self.registry.remove(id, kind).ok_or_else(|| ModelError::NotFound(id.clone(), kind))
    }

    pub fn delete_comp(&mut self, comp: &Component<C::Id, C::Data>) -> Result<C::Data, ModelError<C::Data, C::Id>> {
        self.delete(&comp.id, comp.data.kind())
    }

    // Iterators
    pub fn components(&self) -> impl Iterator<Item = &C::Data> {
        self.registry.values()
    }

    pub fn components_by_kind(&self, kind: <C::Data as ComponentData>::Kind) -> impl Iterator<Item = &C::Data> {
        self.registry.values_by_kind(kind)
    }

    pub fn components_mut(&mut self) -> impl Iterator<Item = &mut C::Data> {
        self.registry.values_mut()
    }

    pub fn components_mut_by_kind(&mut self, kind: <C::Data as ComponentData>::Kind) -> impl Iterator<Item = &mut C::Data> {
        self.registry.values_mut_by_kind(kind)
    }
}

// impl<I, D, R> Model<I, D, R>
// where 
//     I: ComponentId,
//     D: ComponentData, 
//     R: ComponentRegistry<Id = I, Data = D>, 
// {
//     pub fn new(registry: R) -> Self {
//         Self {
//             registry,
//         }
//     }

//     pub fn insert(&mut self, id: I, data: D) -> Result<(), ModelError<D, I>> {

//         let kind = data.kind();

//         // Guard against ID 0/Nil
//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id, kind));
//         }  
        
//         if self.registry.contains(&id, kind) {
//             return Err(ModelError::AlreadyExists(id, kind));
//         }

//         self.registry.insert(id, data);
//         Ok(())
//     }

//     pub fn insert_comp(&mut self, comp: Component<I, D>) -> Result<(), ModelError<D, I>> {
//         self.insert(comp.id, comp.data)
//     }

//     pub fn update(&mut self, id: &I, data: D) -> Result<(), ModelError<D, I>> {
//         let kind = data.kind();  
        
//         // Guard against ID 0/Nil
//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id.clone(), kind));
//         }  
        
//         if !self.registry.contains(id, kind) {
//             return Err(ModelError::NotFound(id.clone(), kind)); 
//         }
        
//         self.registry.insert(id.clone(), data);
//         Ok(())
//     }

//     pub fn update_comp(&mut self, comp: Component<I, D>) -> Result<(), ModelError<D, I>> {
//         self.update(&comp.id, comp.data)
//     }

//     pub fn get(&self, id: &I, kind: D::Kind) -> Result<&D, ModelError<D, I>> {

//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id.clone(), kind));
//         }

//         self.registry
//             .get(id, kind)
//             .ok_or_else(|| ModelError::NotFound(id.clone(), kind))
//     }

//     pub fn get_comp(&self, comp: &Component<I, D>) -> Result<&D, ModelError<D, I>> {
//         let kind = comp.data.kind();
//         self.get(&comp.id, kind)
//     }

//     pub fn get_mut(&mut self, id: &I, kind: D::Kind) -> Result<&mut D, ModelError<D, I>> {

//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id.clone(), kind));
//         }

//         self.registry
//             .get_mut(id, kind)
//             .ok_or_else(|| ModelError::NotFound(id.clone(), kind))
//     }

//     pub fn get_mut_comp(&mut self, comp: &Component<I, D>) -> Result<&mut D, ModelError<D, I>> {
//         let kind = comp.data.kind();
//         self.get_mut(&comp.id, kind)
//     }

//     pub fn get_clone(&self, id: &I, kind: D::Kind) -> Result<D, ModelError<D, I>> {
//         self.get(id, kind).map(|data| data.clone())
//     }

//     pub fn get_clone_comp(&self, comp: &Component<I, D>) -> Result<D, ModelError<D, I>> {
//         let kind = comp.data.kind(); 
//         self.get_clone(&comp.id, kind)
//     }

//     pub fn delete(&mut self, id: &I, kind: D::Kind) -> Result<D, ModelError<D, I>> {

//         if id.is_invalid() {
//             return Err(ModelError::InvalidId(id.clone(), kind));
//         }

//         self.registry
//             .remove(id, kind)
//             .ok_or_else(|| ModelError::NotFound(id.clone(), kind))
//     }

//     pub fn delete_comp(&mut self, comp: &Component<I, D>) -> Result<D, ModelError<D, I>> {
//         let kind = comp.data.kind();
//         self.delete(&comp.id, kind)
//     }
  
//     pub fn components(&self) -> impl Iterator<Item = &D> {
//         self.registry.values()
//     }

//     pub fn components_by_kind(&self, kind: D::Kind) -> impl Iterator<Item = &D> {
//         self.registry.values_by_kind(kind)
//     }

//     pub fn components_mut(&mut self) -> impl Iterator<Item = &mut D> {
//         self.registry.values_mut()
//     }

//     pub fn components_mut_by_kind(&mut self, kind: D::Kind) -> impl Iterator<Item = &mut D> {
//         self.registry.values_mut_by_kind(kind)
//     }
 

// }


