pub mod error;
pub mod value;
pub mod propertied;
pub mod property_node;


pub use value::*;
pub use error::*;
pub use propertied::*;
pub use property_node::*;

use crate::{language::display_text::DisplayText, model::ModelConfig}; 


pub struct Property<C: ModelConfig> {
    pub name: DisplayText,
    pub category: Option<C::Category>, // Using the Category enum from Config
    pub extractor: fn(&dyn Propertied<C>) -> PropertyValue,
}

// pub struct Property<K: UnitCategory> {
//     pub name: DisplayText,
//     pub key: Option<K>,
//     // Now extracts from any object that implements Propertied for this config
//     pub extractor: fn(&dyn Propertied<K>) -> PropertyValue,
// }


 