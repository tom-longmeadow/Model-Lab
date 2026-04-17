pub mod error;
pub mod value;
pub mod propertied;
pub mod property_node;


pub use value::*;
pub use error::*;
pub use propertied::*;
pub use property_node::*;

use crate::{model::ModelConfig}; 


pub struct Property<C: ModelConfig> {
    pub label: C::Display,
    pub category: Option<C::Category>, 
    pub extractor: fn(&dyn Propertied<C>) -> PropertyValue,
}

// pub struct Property<K: UnitCategory> {
//     pub name: DisplayText,
//     pub key: Option<K>,
//     // Now extracts from any object that implements Propertied for this config
//     pub extractor: fn(&dyn Propertied<K>) -> PropertyValue,
// }


 