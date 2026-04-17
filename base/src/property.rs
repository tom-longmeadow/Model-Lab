pub mod error;
pub mod value;

pub use value::*;
pub use error::*;


pub struct Property {
    pub name: String,
    pub category_id: String,  
    pub value: PropertyValue, // Measurement { value: 10.0, unit: "mm2" }
    pub description: Option<String>,
    pub is_readonly: bool,
}


// /// Represents each property
// pub struct Property {
//     pub name: String,
//     pub value: PropertyValue,
//     pub unit: Option<String>,
//     pub description: Option<String>,
//     pub is_readonly: bool,
// }

// /// The interface for getting and setting properties
// pub trait PropertyInterface {
//     /// Returns a snapshot of all properties for UI display/spreadsheets.
//     fn get_properties(&self) -> Vec<Property>;
    
//     /// Updates a specific property. 
//     /// Returns a PropertyError if the name is unknown, the type is wrong, 
//     /// or the value is out of bounds.
//     fn set_property(&mut self, key: &str, value: PropertyValue) -> Result<(), PropertyError>;
// }
 
// pub struct LimitConfig {
//     pub max_rpm: f64,
//     pub max_temp: f64,
// }

// // In your Motor's PropertyInterface implementation:
// fn get_properties(&self) -> Vec<Property> {
//     vec![
//         Property::new("id", "ID", PropertyValue::Text(self.id.clone())),
        
//         // Nesting the LimitConfig properties under a Group
//         Property::new("limits", "Safety Limits", PropertyValue::Group(vec![
//             Property::new("max_rpm", "Max RPM", PropertyValue::Number(self.limits.max_rpm)),
//             Property::new("max_temp", "Max Temperature", PropertyValue::Number(self.limits.max_temp)),
//         ])),
//     ]
// }