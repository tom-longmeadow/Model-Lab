
 use crate::{
    component_id_macro, 
    component_id_primitive_macro, 
    prelude::{
        DisplayLanguage, ModelConfig, Propertied, PropertyConfig, PropertyName, PropertyNode, PropertySchema, PropertyValue
    }, property_key 
};

 pub trait ComponentId: Copy + Eq + std::hash::Hash + std::fmt::Debug + std::fmt::Display + std::str::FromStr {
    /// Returns the "null" or "free" version of this ID.
    fn invalid() -> Self; 

    /// Checks if this ID is the "null" state.
    fn is_invalid(&self) -> bool {
        *self == Self::invalid()
    }

    fn to_option(self) -> Option<Self> {
        if self.is_invalid() { None } else { Some(self) }
    }
}

// Create the ability to use unsigned integers as an Id
component_id_primitive_macro!(u32, u64, u128, usize);

component_id_macro!(IDu, usize);
component_id_macro!(ID128, u128);
component_id_macro!(ID64, u64);
component_id_macro!(ID32, u32);

pub trait ComponentKind: Copy + Eq + std::hash::Hash + std::fmt::Debug {
    type Id: ComponentId;  
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ComponentKey<K: ComponentKind> {
    pub id: K::Id, // 🔒 This forces them to match perfectly!
    pub kind: K,
}


pub trait HasKind {
    type Kind: ComponentKind;
    fn kind(&self) -> Self::Kind;
}

pub trait ComponentData<C: PropertyConfig>: Clone + Propertied<C> + HasKind { 
    fn kind_name() -> PropertyName<C>;
}


type ID<D> = <<D as HasKind>::Kind as ComponentKind>::Id;

pub struct Component<C: ModelConfig, D: HasKind<Kind = C::Kind>> {
    pub id: <C::Kind as ComponentKind>::Id,
    pub data: D,
}

impl<C: ModelConfig, D: HasKind<Kind = C::Kind>> Component<C, D> {

    pub fn new(id: ID<D>, data: D) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> ID<D> {
        self.id
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn key(&self) -> ComponentKey<D::Kind> {
        ComponentKey {
            id: self.id,
            kind: self.data.kind(),
        }
    }
}

impl<C: ModelConfig, D: ComponentData<C> + HasKind<Kind = C::Kind>> Component<C, D> { 
    pub const ID_KEY: u64 = property_key!(C, ID);
}
 
impl<C: ModelConfig, D: ComponentData<C> + HasKind<Kind = C::Kind>> Propertied<C> for Component<C, D> {

    fn get_schema() -> PropertyNode<C> {
        let id_schema = PropertyNode::Leaf(
            PropertySchema::new_id_readonly(C::Display::id_label(), Self::ID_KEY)
        );
        let mut children = vec![id_schema];
        children.push(D::get_schema());
        PropertyNode::Group {
            name: D::kind_name(),
            children,
        }
    }

    fn get_value(&self, key: u64) -> Option<PropertyValue> {
        match key { 
            Self::ID_KEY => Some(PropertyValue::ID(self.id.to_string())), 
            _ => self.data.get_value(key),
        }
    }

    fn set_value(&mut self, key: u64, value: PropertyValue) {
        match key {
            Self::ID_KEY => {
                if let PropertyValue::ID(s) = value {
                    if let Ok(parsed_id) = s.parse() {
                        self.id = parsed_id;
                    }
                }
            } 
            _ => self.data.set_value(key, value),
        }
    }
}
 

 