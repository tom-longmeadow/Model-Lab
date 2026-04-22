use crate::prelude::{PropertyConfig, PropertyName, PropertySchema};


 

 

#[derive(Debug, Clone)]
pub enum PropertyNode<C: PropertyConfig> { 
    Group {
        name: PropertyName<C>,
        children: Vec<PropertyNode<C>>,
    }, 
    Leaf(PropertySchema<C>),
}

impl<C: PropertyConfig> PropertyNode<C> {
     
    pub fn name(&self) -> &PropertyName<C> {
        match self {
            Self::Group { name, .. } => name,
            Self::Leaf(schema) => &schema.name,
        }
    }

    /// Helper to get the pre-computed hash key of this node
    pub fn key(&self) -> u64 {
        match self {
            Self::Group { name, .. } => { 
                PropertySchema::<C>::hash_key(&name.to_string())
            }
            Self::Leaf(schema) => schema.key,
        }
    }
}
 

// pub enum PropertyNode<C: PropertyConfig> {
    
//     Group {
//         name: PropertyName<C>,  
//         children: Vec<PropertyNode<C>>,
//     },
    
//     Text(Property<C>),
//     Number(Property<C>),
//     Percent(Property<C>),
//     Integer(Property<C>),
//     Unsigned(Property<C>),
//     Boolean(Property<C>),
// }

// impl<C: PropertyConfig> PropertyNode<C> {

//     pub fn group(text: C::Display, children: Vec<PropertyNode<C>>) -> Self {
//         Self::Group {
//             name: PropertyName::new(text),
//             children,
//         }
//     }
//     pub fn group_raw(name: impl Into<String>, children: Vec<PropertyNode<C>>) -> Self {
//         Self::Group {
//             name: PropertyName::new_str(name),
//             children,
//         }
//     }

   
//     pub fn text(key: C::Display, default_val: impl Into<String>) -> Self {
//         Self::Text(Property::localized(key, default_val.into(), None))
//     }
//     pub fn text_raw(name: impl Into<String>, default_val: impl Into<String>) -> Self {
//         Self::Text(Property::raw(name, default_val.into(), None))
//     }


//     pub fn number(key: C::Display, default_val: f64, unit: Option<C::UnitCategory>) -> Self {
//         Self::Number(Property::localized(key, default_val, unit))
//     }
//     pub fn number_raw(name: impl Into<String>, default_val: f64, unit: Option<C::UnitCategory>) -> Self {
//         Self::Number(Property::raw(name, default_val, unit))
//     }

//     pub fn percent(key: C::Display, default_val: f64) -> Self {
//         Self::Percent(Property::localized(key, default_val, None))
//     }
//      pub fn percent_raw(name: impl Into<String>, default_val: f64) -> Self {
//         Self::Percent(Property::raw(name, default_val, None))
//     }

//     pub fn integer(key: C::Display, default_val: i64) -> Self {
//         Self::Integer(Property::localized(key, default_val, None))
//     }
//     pub fn integer_raw(name: impl Into<String>, default_val: i64) -> Self {
//         Self::Integer(Property::raw(name, default_val, None))
//     }

//     pub fn unsigned(key: C::Display, default_val: u64) -> Self {
//         Self::Unsigned(Property::localized(key, default_val, None))
//     }
//     pub fn unsigned_raw(name: impl Into<String>, default_val: u64) -> Self {
//         Self::Unsigned(Property::raw(name, default_val, None))
//     }

//     pub fn boolean(key: C::Display, default_val: bool) -> Self {
//         Self::Boolean(Property::localized(key, default_val, None))
//     }
//     pub fn boolean_raw(name: impl Into<String>, default_val: bool) -> Self {
//         Self::Boolean(Property::raw(name, default_val, None))
//     }

    
//     pub fn name(&self) -> &PropertyName<C> {
//         match self {
//             Self::Group { name, .. } => name,
//             Self::Text(p) | Self::Number(p) | Self::Percent(p) | 
//             Self::Integer(p) | Self::Unsigned(p) | Self::Boolean(p) => &p.name,
//         }
//     }
// }

 

 


 
    