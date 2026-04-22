  
 
use crate::prelude::{PropertyConfig, PropertySchema, PropertyValue, PropertyNode, PropertyError};

 
 pub trait Propertied<C: PropertyConfig> { 
    
    fn get_schema() -> PropertyNode<C> where Self: Sized;    
    fn get_value(&self, key: u64) -> Option<PropertyValue>;   
    fn set_value(&mut self, key: u64, value: PropertyValue);
}


// pub trait Propertied<C: PropertyConfig> { 
//     fn get_schema() -> PropertyNode<C> where Self: Sized;   
//     fn get_value(&self, key: u64) -> Result<PropertyValue, PropertyError>;  
//     fn set_value(&mut self, key: u64, schema: &PropertySchema<C>, value: PropertyValue) -> Result<(), PropertyError>;
// }


 