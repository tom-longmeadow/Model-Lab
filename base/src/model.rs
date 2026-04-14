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

/*************
* UNIT TESTS *
*************/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::registry::HashMapRegistry;

    type MockId = u32;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum MockKind {
        Kind1, 
        Kind2, 
    }
    impl ComponentKind for MockKind {}

    #[derive(Debug, Clone, PartialEq)]
    enum MockData {
        Kind1{state: f32}, 
        Kind2{other: u32}, 
    }

    impl ComponentData for MockData {
        type Kind = MockKind;
        
        fn kind(&self) -> Self::Kind {
            match self {
                Self::Kind1 { .. } => MockKind::Kind1,
                Self::Kind2 { .. } => MockKind::Kind2,
            }
        }
    }

    fn get_model() -> Model<MockId, MockData, HashMapRegistry<MockId, MockData>> {
        let registry = HashMapRegistry::new();
        Model::new(registry)
    }

    
    #[test]
    fn test_flow() {
        
        
        let mut model = get_model();

        let id = 1;
        let data = MockData::Kind1 { state: 10.0 };

        // --- TEST INSERT ---
        assert!(model.insert(id, data.clone()).is_ok());
        
        // Ensure collision detection works
        let err = model.insert(id, data.clone()).unwrap_err();
        assert!(matches!(err, ModelError::AlreadyExists(..)));

        // --- TEST GET & UPDATE ---
        // Match on the result to verify the inner state
        let fetched = model.get(&id, MockKind::Kind1).unwrap();
        if let MockData::Kind1 { state } = fetched {
            assert_eq!(*state, 10.0);
        } else {
            panic!("Expected Kind1 variant");
        }

        // Update in place
        let new_data = MockData::Kind1 { state: 1.0 };
        assert!(model.update(&id, new_data).is_ok());
        
        let updated = model.get(&id, MockKind::Kind1).unwrap();
        if let MockData::Kind1 { state } = updated {
            assert_eq!(*state, 1.0);
        }

        // --- TEST DELETE ---
        let removed = model.delete(&id, MockKind::Kind1).unwrap();
        if let MockData::Kind1 { state } = removed {
            assert_eq!(state, 1.0);
        }
        
        // Verify it's truly gone
        assert!(model.get(&id, MockKind::Kind1).is_err());
    }

     #[test]
    fn test_kinds() {
        let mut model = get_model();

        let id = 1;
        let data1 = MockData::Kind1 { state: 10.0 };
        let data2 = MockData::Kind2 { other: 99 };

        // 1. Insert Kind1 with ID 1
        assert!(model.insert(id, data1).is_ok());

        // 2. Insert Kind2 with the SAME ID 1
        // This should SUCCEED because they have different Kinds (ComponentKey is different)
        assert!(model.insert(id, data2).is_ok(), "Should allow same ID for different Kinds");

        // 3. Verify both exist independently
        let res1 = model.get(&id, MockKind::Kind1).unwrap();
        let res2 = model.get(&id, MockKind::Kind2).unwrap();

        if let MockData::Kind1 { state } = res1 { assert_eq!(*state, 10.0); }
        if let MockData::Kind2 { other } = res2 { assert_eq!(*other, 99); }

        // 4. Verify collision still happens if BOTH ID and Kind match
        let duplicate = MockData::Kind1 { state: 20.0 };
        let err = model.insert(id, duplicate).unwrap_err();
        assert!(matches!(err, ModelError::AlreadyExists(..)));
    }

    #[test]
    fn test_iteration_and_filtering() {
        let mut model = get_model();
        model.insert(1, MockData::Kind1 { state: 4.0 }).unwrap();
        model.insert(2, MockData::Kind1 { state: 2.0 }).unwrap();
        model.insert(1, MockData::Kind2 { other: 99 }).unwrap();

        // Test Kind1 collection
        let kind1_count = model.components_by_kind(MockKind::Kind1).count();
        assert_eq!(kind1_count, 2);

        // Test total collection
        let total_count = model.components().count();
        assert_eq!(total_count, 3);
    }

    #[test]
    fn test_mutable_access() {
        let mut model = get_model();
        model.insert(1, MockData::Kind1 { state: 10.0 }).unwrap();

        // Get mutably and modify
        if let Ok(data) = model.get_mut(&1, MockKind::Kind1) {
            if let MockData::Kind1 { state } = data {
                *state += 5.0;
            }
        }

        // Verify the change stuck
        if let Ok(MockData::Kind1 { state }) = model.get(&1, MockKind::Kind1) {
            assert_eq!(*state, 15.0);
        }
    }

    #[test]
    fn test_error_on_missing() {
        let mut model = get_model();
        
        let res = model.update(&99, MockData::Kind1 { state: 0.0 });
        assert!(matches!(res, Err(ModelError::NotFound(99, MockKind::Kind1))));

        let res = model.delete(&99, MockKind::Kind1);
        assert!(matches!(res, Err(ModelError::NotFound(99, MockKind::Kind1))));
    }

    #[test]
    fn test_wrong_kind_lookup() {
        let mut model = get_model();
        model.insert(1, MockData::Kind1 { state: 10.0 }).unwrap();

        // ID 1 exists, but it is NOT Kind2.
        let res = model.get(&1, MockKind::Kind2);
        assert!(matches!(res, Err(ModelError::NotFound(1, MockKind::Kind2))));
    }

    #[test]
    fn test_bulk_mutation() {
        let mut model = get_model();
        model.insert(1, MockData::Kind1 { state: 10.0 }).unwrap();
        model.insert(2, MockData::Kind1 { state: 20.0 }).unwrap();

        // Multiply all Kind1 states by 2
        for data in model.components_mut_by_kind(MockKind::Kind1) {
            if let MockData::Kind1 { state } = data {
                *state *= 2.0;
            }
        }

        // Verify both updated
        if let Ok(MockData::Kind1 { state }) = model.get(&1, MockKind::Kind1) {
            assert_eq!(*state, 20.0);
        }
        if let Ok(MockData::Kind1 { state }) = model.get(&2, MockKind::Kind1) {
            assert_eq!(*state, 40.0);
        }
    }

    #[test]
    fn test_clone_independence() {
        let mut model = get_model();
        model.insert(1, MockData::Kind1 { state: 10.0 }).unwrap();

        let mut cloned_data = model.get_clone(&1, MockKind::Kind1).unwrap();
        if let MockData::Kind1 { state } = &mut cloned_data {
            *state = 99.0;
        }

        // Original should still be 10.0
        if let Ok(MockData::Kind1 { state }) = model.get(&1, MockKind::Kind1) {
            assert_eq!(*state, 10.0);
        }
    }
}
  