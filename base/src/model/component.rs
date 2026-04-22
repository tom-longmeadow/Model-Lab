
 use crate::{
    component_id_macro, 
    component_id_primitive_macro, 
    prelude::{
        DisplayText, Propertied, PropertyConfig, PropertyNode, PropertyValue,
    }, property::{PropertyName, PropertySchema}, property_key
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

pub trait ComponentData<C: PropertyConfig>: Clone + Propertied<C> {
    type Kind: ComponentKind; 
    fn kind(&self) -> Self::Kind;
    fn kind_name() -> PropertyName<C>;
}

 
pub struct Component<C: PropertyConfig, D: ComponentData<C>> {  
    pub id: <<D as ComponentData<C>>::Kind as ComponentKind>::Id,
    pub data: D,
}

impl<C: PropertyConfig, D: ComponentData<C>> Component<C, D> {
 
    pub fn id(&self) -> <<D as ComponentData<C>>::Kind as ComponentKind>::Id {
        self.id
    }
    
    pub fn data(&self) -> &D { 
        &self.data 
    }
    
    pub fn kind(&self) -> D::Kind {
        self.data.kind()
    }

    pub fn kind_name(&self) -> PropertyName<C> {
        D::kind_name()
    }

    pub fn key(&self) -> ComponentKey<D::Kind> {
        ComponentKey {
            id: self.id,
            kind: self.kind(),
        }
    }
}

impl<C: PropertyConfig, D: ComponentData<C>> Component<C, D> { 
   pub const ID_KEY: u64 = property_key!(C, ID);
}
 
 
impl<C: PropertyConfig, D: ComponentData<C>> Propertied<C> for Component<C, D> {

    fn get_schema() -> PropertyNode<C> {
         
        let kind = D::kind_name(); 
        let id_schema = PropertyNode::Leaf(
            PropertySchema::new_id_readonly(Self::ID_KEY)
        );
        
        let mut children = vec![id_schema]; 
        
        // 🌟 Push the entire group rather than unpacking it to keep folders intact
        children.push(D::get_schema()); 
         
        PropertyNode::Group {
            name: kind,
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
 


// pub trait ComponentId: Copy + Eq + std::hash::Hash + std::fmt::Debug {
//     /// Returns the "null" or "free" version of this ID.
//     /// UUID nil or for u64, 0.
//     fn invalid() -> Self; 

//     /// Checks if this ID is the "null" state.
//     fn is_invalid(&self) -> bool {
//         *self == Self::invalid()
//     }

//     fn to_option(self) -> Option<Self> {
//         if self.is_invalid() { None } else { Some(self) }
//     }
// }

// // Create the ability to use unsigned integers as an Id
// component_id_primitive_macro!(u8, u16, u32, u64, u128, usize);

// // Making a component ID of type name = IDu, primitive type = usize
// component_id_macro!(IDu, usize);

// // Making a component ID of type name = ID64, primitive type = u64
// component_id_macro!(ID64, u64);


// /// Should be a component type enum.  For instance enum StructuralType {Joint, Member}
// pub trait ComponentKind: Copy + Eq + std::hash::Hash + std::fmt::Debug {
//     type Id: ComponentId; // The ID type is "locked" to the Kind
// }
 

// /// The key for storing components of different kinds in the same collection
// /// unique with (id, type)
// #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
// pub struct ComponentKey<K: ComponentKind> {
//     pub id: K::Id,
//     pub kind: K,
// }
 
// pub trait ComponentData: Clone  {
//     type Kind: ComponentKind; 
//     fn kind(&self) -> Self::Kind;
// }

// /// A component in the model
// pub struct Component<D: ComponentData> {
//     pub id: <<D as ComponentData>::Kind as ComponentKind>::Id,
//     pub data: D,
// }

// impl<D: ComponentData> Component<D> {
    
//     pub fn id(&self) -> <<D as ComponentData>::Kind as ComponentKind>::Id {
//         self.id
//     }
//     pub fn data(&self) -> &D { &self.data }
    
//     pub fn kind(&self) -> D::Kind {
//         self.data.kind()
//     }

//     pub fn key(&self) -> ComponentKey<D::Kind> {
//         ComponentKey {
//             id: self.id,
//             kind: self.kind(),
//         }
//     }
// }

  