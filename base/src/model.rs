pub mod component;
pub mod registry;

pub use component::*;
pub use registry::ComponentRegistry;

pub mod prelude {
    pub use super::component::*;
    pub use super::registry::ComponentRegistry;
    pub use super::Model;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModelError<D: ComponentData, I: ComponentId> {
    NotFound(I, D::Kind),
    AlreadyExists(I, D::Kind),
    ValidationError(I, D::Kind, String),
}

impl<D, I> std::fmt::Display for ModelError<D, I> 
where 
    D: ComponentData, 
    I: ComponentId,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id, kind) => {
                write!(f, "{:?} {:?} not found", kind, id)
            }
            Self::AlreadyExists(id, kind) => {
                write!(f, "{:?} {:?} already exists", kind, id)
            }
            Self::ValidationError(id, kind, msg) => {
                write!(f, "Validation error for {:?} {:?}: {}", kind, id, msg)
            }
        }
    }
}


pub struct Model<I, D, R> 
where 
    I: ComponentId, 
    D: ComponentData, 
    R: ComponentRegistry<Id = I, Data = D>, 
{
    registry: R, 
}

impl<I, D, R> Model<I, D, R>
where 
    I: ComponentId,
    D: ComponentData, 
    R: ComponentRegistry<Id = I, Data = D>, 
{
    pub fn new(registry: R) -> Self {
        Self {
            registry,
        }
    }

    pub fn insert(&mut self, id: I, data: D) -> Result<(), ModelError<D, I>> {
        let kind = data.kind();  
        
        if self.registry.contains(&id, kind) {
            return Err(ModelError::AlreadyExists(id, kind));
        }

        self.registry.insert(id, data);
        Ok(())
    }

    pub fn insert_comp(&mut self, comp: Component<I, D>) -> Result<(), ModelError<D, I>> {
        self.insert(comp.id, comp.data)
    }

    pub fn update(&mut self, id: &I, data: D) -> Result<(), ModelError<D, I>> {
        let kind = data.kind();  
        
        
        if !self.registry.contains(id, kind) {
            return Err(ModelError::NotFound(id.clone(), kind)); 
        }
        
        self.registry.insert(id.clone(), data);
        Ok(())
    }

    pub fn update_comp(&mut self, comp: Component<I, D>) -> Result<(), ModelError<D, I>> {
        self.update(&comp.id, comp.data)
    }

    pub fn get(&self, id: &I, kind: D::Kind) -> Result<&D, ModelError<D, I>> {
        self.registry
            .get(id, kind)
            .ok_or_else(|| ModelError::NotFound(id.clone(), kind))
    }

    pub fn get_comp(&self, comp: &Component<I, D>) -> Result<&D, ModelError<D, I>> {
        let kind = comp.data.kind();
        self.get(&comp.id, kind)
    }

    pub fn get_mut(&mut self, id: &I, kind: D::Kind) -> Result<&mut D, ModelError<D, I>> {
        self.registry
            .get_mut(id, kind)
            .ok_or_else(|| ModelError::NotFound(id.clone(), kind))
    }

    pub fn get_mut_comp(&mut self, comp: &Component<I, D>) -> Result<&mut D, ModelError<D, I>> {
        let kind = comp.data.kind();
        self.get_mut(&comp.id, kind)
    }

    pub fn get_clone(&self, id: &I, kind: D::Kind) -> Result<D, ModelError<D, I>> {
        self.get(id, kind).map(|data| data.clone())
    }

    pub fn get_clone_comp(&self, comp: &Component<I, D>) -> Result<D, ModelError<D, I>> {
        let kind = comp.data.kind();
        self.get_clone(&comp.id, kind)
    }

    pub fn delete(&mut self, id: &I, kind: D::Kind) -> Result<D, ModelError<D, I>> {
        self.registry
            .remove(id, kind)
            .ok_or_else(|| ModelError::NotFound(id.clone(), kind))
    }

    pub fn delete_comp(&mut self, comp: &Component<I, D>) -> Result<D, ModelError<D, I>> {
        let kind = comp.data.kind();
        self.delete(&comp.id, kind)
    }
  
    pub fn components(&self) -> impl Iterator<Item = &D> {
        self.registry.values()
    }

    pub fn components_by_kind(&self, kind: D::Kind) -> impl Iterator<Item = &D> {
        self.registry.values_by_kind(kind)
    }

    pub fn components_mut(&mut self) -> impl Iterator<Item = &mut D> {
        self.registry.values_mut()
    }

    pub fn components_mut_by_kind(&mut self, kind: D::Kind) -> impl Iterator<Item = &mut D> {
        self.registry.values_mut_by_kind(kind)
    }
 

}



  