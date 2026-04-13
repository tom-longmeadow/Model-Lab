
/// the type of ID to use, u64, UUID, String.
pub trait ComponentId: Copy + Eq + std::hash::Hash + std::fmt::Debug {}

/// Should be a component type enum.  For instance enum StructuralType {Joint, Member}
pub trait ComponentKind: Copy + Eq + std::hash::Hash + std::fmt::Debug {}

/// The key for storing components of different kinds in the same collection
/// unique with (id, type)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ComponentKey<I, K> 
where 
    K: ComponentKind, 
    I: ComponentId
{
    pub id: I,
    pub kind: K,
}

/// Represents the data for each component type. For instance
/// enum StructuralData {
///    Joint { x: f64, y: f64 },
///    Member { a: u64, b: u64 },
/// }
pub trait ComponentData: Clone  {
    type K: ComponentKind; 
    fn kind(&self) -> Self::K;
}

/// Represents the component interface.
pub trait ComponentInterface {
    type Id: ComponentId;
    type Data: ComponentData;

     
    fn id(&self) -> Self::Id;
    fn data(&self) -> &Self::Data;

    fn kind(&self) -> <Self::Data as ComponentData>::Kind {
        self.data().kind()
    }

    fn key(&self) -> ComponentKey<Self::Id, <Self::Data as ComponentData>::Kind> {
        ComponentKey {
            id: self.id(),
            kind: self.kind(),
        }
    }
}

/// A component in the model
pub struct Component<I, D>
where 
    I: ComponentId, 
    D: ComponentData,  
{
    pub id: I,  
    pub data: D,
}

impl<I, D> ComponentInterface for Component<I, D> 
where 
    I: ComponentId, 
    D: ComponentData, 
{
    type Id = I;
    type Data = D;

    fn id(&self) -> Self::Id { self.id }
    fn data(&self) -> &Self::Data { &self.data }
}

  