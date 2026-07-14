  
 
use crate::prelude::{PropertyConfig, PropertyValue, PropertyNode};

 
 pub trait Propertied<C: PropertyConfig> { 
    
    fn get_schema() -> PropertyNode<C> where Self: Sized;    
    fn get_value(&self, key: u64) -> Option<PropertyValue>;   
    fn set_value(&mut self, key: u64, value: PropertyValue);
}


 