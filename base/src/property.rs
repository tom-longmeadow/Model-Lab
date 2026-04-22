pub mod error;
pub mod value;
pub mod propertied;
pub mod node;
pub mod name;
pub mod config;
pub mod schema;

pub use value::*;
pub use propertied::*;
pub use node::*;
pub use name::*; 
pub use schema::*;

// use crate::{prelude::DisplayLanguage, unit::{UnitSettings, UnitSystem}};
// use std::mem::discriminant; 

 
// pub struct Property<C: PropertyConfig> {
//     pub key: u64, 
//     pub value: PropertyValue,  
//     pub read_only: bool, 
//     _marker: std::marker::PhantomData<C>,
// }

// impl <C: PropertyConfig> Property<C> {
//     /// Creates a live property instance directly from its blueprint schema.
//     pub fn from_schema(
//         schema: &PropertySchema<C>, 
//         value: impl Into<PropertyValue>, 
//         read_only: bool
//     ) -> Result<Self, PropertyError> {
//         let val = value.into();
        
//         // 1. Enforce type safety at the moment of creation!
//         let incoming_kind: PropertyValueDiscriminants = (&val).into();
//         if incoming_kind != schema.kind {
//             return Err(PropertyError::TypeMismatch);
//         }

//         Ok(Self {
//             key: schema.key, 
//             value: val,
//             read_only,
//             _marker: std::marker::PhantomData,
//         })
//     }

//     /// Validates a raw write attempt against the instance's read-only status 
//     /// and the schema's expected type rules.
//     pub fn validate_set(&self, schema: &PropertySchema<C>, value: &PropertyValue) -> Result<(), PropertyError> {
//         // Guard 1: Instance read-only check
//         if self.read_only {
//             return Err(PropertyError::ReadOnly(schema.name.to_string()));
//         }

//         // Guard 2: Schema type check
//         let incoming_kind: PropertyValueDiscriminants = value.into();
//         if incoming_kind != schema.kind {
//             return Err(PropertyError::TypeMismatch);
//         }

//         Ok(())
//     }
// }


// pub struct Property<C: PropertyConfig> {
//     pub key: u64, 
//     pub name: PropertyName<C>,
//     pub value: PropertyValue,  
//     pub unit: Option<C::UnitCategory>,    
//     pub read_only: bool,
// }

// impl<C: PropertyConfig> Property<C> {
    
   
//     /// Constructs a custom ad-hoc property node from raw strings.
//     pub fn raw(
//         name: impl Into<String>, 
//         value: impl Into<PropertyValue>, 
//         unit: Option<C::UnitCategory>, 
//     ) -> Self {
//         let name_str = name.into();
//         Self {
//             key: Self::hash_key(&name_str), 
//             name: PropertyName::Raw(name_str),
//             value: value.into(),
//             unit,
//             read_only: false,
//         }
//     }

//     pub fn raw_readonly(
//         name: impl Into<String>, 
//         value: impl Into<PropertyValue>, 
//         unit: Option<C::UnitCategory>, 
//     ) -> Self {
//         let name_str = name.into();
//         Self {
//             key: Self::hash_key(&name_str), 
//             name: PropertyName::Raw(name_str),
//             value: value.into(),
//             unit,
//             read_only: true
//         }
//     }

//     /// Constructs a localized property node tied strictly to the DisplayText enum.
//     pub fn localized(
//         key: C::Display, 
//         value: impl Into<PropertyValue>, 
//         unit: Option<C::UnitCategory>
//     ) -> Self {
//         let name_str = key.default_text();
//         Self {
//             key: Self::hash_key(name_str),
//             name: PropertyName::Localized(key),
//             value: value.into(),
//             unit,
//             read_only: false,
//         }
//     }

//     pub fn localized_readonly(
//         key: C::Display, 
//         value: impl Into<PropertyValue>, 
//         unit: Option<C::UnitCategory>
//     ) -> Self {
//         let name_str = key.default_text(); 
//         Self {
//             key: Self::hash_key(name_str),
//             name: PropertyName::Localized(key),
//             value: value.into(),
//             unit,
//             read_only: true,
//         }
//     }

    // pub fn parse_as<T: std::str::FromStr>(&self, value: PropertyValue) -> Result<T, PropertyError> {
    //     if let PropertyValue::Text(s) = value {
    //         s.parse::<T>().map_err(|_| PropertyError::InvalidFormat { 
    //             expected: Box::leak(self.name.to_string().into_boxed_str()), 
    //             received: s,
    //         })
    //     } else {
    //         Err(PropertyError::TypeMismatch)
    //     }
    // }

    
    // /// Validates a raw write attempt to make sure properties can't have their 
    // /// read-only status violated or have their data types cross-contaminated.
    // fn validate_set(&self, value: &PropertyValue) -> Result<(), PropertyError> {
    //     if self.read_only {
    //         return Err(PropertyError::ReadOnly(self.name.to_string()));
    //     }
    //     if std::mem::discriminant(&self.value) != std::mem::discriminant(value) {
    //         return Err(PropertyError::TypeMismatch);
    //     }
    //     Ok(())
    // }

    
//     /// Parses unformatted UI text and immediately applies SI unit conversions 
//     /// while fully verifying that the requested payload obeys the template schema.
//     pub fn try_parse(&self, input: &str, system: &UnitSystem<C>) -> Result<PropertyValue, PropertyError> {
//         // Parse the string according to the type established in the template
//         let parsed_value = match &self.value {
//             PropertyValue::Number(_) => {
//                 let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: "Number",
//                     received: input.to_string(),
//                 })?;
                
//                 if let Some(cat) = self.unit {
//                     PropertyValue::Number(system.display.get(cat).to_base(val))
//                 } else {
//                     PropertyValue::Number(val)
//                 }
//             }
//             PropertyValue::Percent(_) => {
//                 let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: "Percentage",
//                     received: input.to_string(),
//                 })?;
//                 PropertyValue::Percent(val / 100.0)
//             }
//             PropertyValue::Integer(_) => {
//                 let val: i64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: "Integer",
//                     received: input.to_string(),
//                 })?;
//                 PropertyValue::Integer(val)
//             }
//             PropertyValue::Unsigned(_) => {
//                 let val: u64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: "Unsigned Integer",
//                     received: input.to_string(),
//                 })?;
//                 PropertyValue::Unsigned(val)
//             }
//             PropertyValue::Boolean(_) => {
//                 let s = input.to_lowercase();
//                 if s == "true" || s == "1" || s == "yes" {
//                     PropertyValue::Boolean(true)
//                 } else if s == "false" || s == "0" || s == "no" {
//                     PropertyValue::Boolean(false)
//                 } else {
//                     return Err(PropertyError::InvalidFormat {
//                         expected: "Boolean (true/false, 1/0, yes/no)",
//                         received: input.to_string(),
//                     });
//                 }
//             }
//             PropertyValue::Text(_) => PropertyValue::Text(input.to_string()),
//         };

//         // Leverage the machine validator to execute read_only and discriminant guards
//         self.validate_set(&parsed_value)?;

//         Ok(parsed_value)
//     }

    // /// Translates SI base math from the database back to human-readable text 
    // /// according to the active units and translation preferences.
    // pub fn format_value(&self, value: PropertyValue, system: &UnitSystem<C>) -> String {
    //     // Guard against UI bugs requesting format rendering on misaligned variants
    //     if discriminant(&self.value) != discriminant(&value) {
    //         return "Type Mismatch".to_string(); 
    //     }

    //     match value {
    //         PropertyValue::Number(n) => {
    //             if let Some(cat) = self.unit {
    //                 let display_kind = system.display.get(cat);
    //                 let converted = display_kind.from_base(n);
    //                 format!("{:.2} {}", converted, system.symbol(cat))
    //             } else {
    //                 format!("{:.2}", n)
    //             }
    //         }
    //         PropertyValue::Percent(n) => format!("{:.1}%", n * 100.0),
    //         PropertyValue::Integer(i) => i.to_string(),
    //         PropertyValue::Unsigned(u) => u.to_string(),
    //         PropertyValue::Boolean(b) => b.to_string(),
    //         PropertyValue::Text(t) => t,
    //     }
    // }
// }


 
// pub struct Property<C: PropertyConfig> {
//     pub key: u64, 
//     pub name: PropertyName<C>,
//     pub value: PropertyValue,  
//     pub unit: Option<C::UnitCategory>,   
//     pub read_only: bool, 
// }

// impl<C: PropertyConfig> Property<C> {

//     pub const fn hash_key(s: &str) -> u64 {
//         let mut hash = 14695981039346656037u64;
//         let bytes = s.as_bytes();
//         let mut i = 0;
//         while i < bytes.len() {
//             hash ^= bytes[i] as u64;
//             hash = hash.wrapping_mul(1099511628211u64);
//             i += 1;
//         }
//         hash
//     }
   
//     pub fn raw(
//         name: impl Into<String>, 
//         value: impl Into<PropertyValue>, 
//         unit: Option<C::UnitCategory>
//     ) -> Self {
//         let name_str = name.into();
//         Self {
//             // Auto-generate the fast integer key from the string!
//             key: Self::hash_key(&name_str), 
//             name: PropertyName::Raw(name_str),
//             value: value.into(),
//             unit,
//             read_only: false
//         }
//     }

//     pub fn localized(
//         key: C::Display, 
//         value: impl Into<PropertyValue>, 
//         unit: Option<C::UnitCategory>
//     ) -> Self {
//         let name_str = key.default_text(); // Or use a slug identifier mapped to the enum
//         Self {
//             key: Self::hash_key(name_str),
//             name: PropertyName::Localized(key),
//             value: value.into(),
//             unit,
//             read_only: false
//         }
//     }

    
//     pub fn validate_set(&self, value: &PropertyValue) -> Result<(), PropertyError> {
//         if self.read_only {
//             return Err(PropertyError::ReadOnly(self.name.into()));
//         }
//         if std::mem::discriminant(&self.value) != std::mem::discriminant(value) {
//             return Err(PropertyError::TypeMismatch);
//         }
//         Ok(())
//     }

//     /// Parses a string input and immediately guarantees it fits this property's schema!
//     pub fn try_parse(&self, input: &str, system: &UnitSystem<C>) -> Result<PropertyValue, PropertyError> {
//         // 1. Block operations on read-only fields early
//         if self.read_only {
//             return Err(PropertyError::ReadOnly(self.name.into()));
//         }

//         // 2. Parse the string according to the requested type
//         let parsed_value = match &self.value {
//             PropertyValue::Number(_) => {
//                 let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat)?;
//                 if let Some(cat) = self.unit {
//                     PropertyValue::Number(system.display.get(cat).to_base(val))
//                 } else {
//                     PropertyValue::Number(val)
//                 }
//             }
//             PropertyValue::Percent(_) => {
//                 let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat)?;
//                 PropertyValue::Percent(val / 100.0)
//             }
//             PropertyValue::Integer(_) => {
//                 PropertyValue::Integer(input.parse().map_err(|_| PropertyError::InvalidFormat)?)
//             }
//             PropertyValue::Unsigned(_) => {
//                 PropertyValue::Unsigned(input.parse().map_err(|_| PropertyError::InvalidFormat)?)
//             }
//             PropertyValue::Boolean(_) => {
//                 let s = input.to_lowercase();
//                 PropertyValue::Boolean(s == "true" || s == "1" || s == "yes")
//             }
//             PropertyValue::Text(_) => PropertyValue::Text(input.to_string()),
//         };

//         // 3. Double-check that it successfully maps to the target discriminant
//         if std::mem::discriminant(&self.value) != std::mem::discriminant(&parsed_value) {
//             return Err(PropertyError::TypeMismatch);
//         }

//         Ok(parsed_value)
//     }
 
//     pub fn parse_value(&self, input: &str, system: &UnitSystem<C>) -> PropertyValue {
//         match &self.value {
//             PropertyValue::Number(_) => {
//                 let val: f64 = input.parse().unwrap_or(0.0);
//                 if let Some(cat) = self.unit {
//                     // Convert from Display unit to SI Base
//                     PropertyValue::Number(system.display.get(cat).to_base(val))
//                 } else {
//                     PropertyValue::Number(val)
//                 }
//             }
//             PropertyValue::Percent(_) => {
//                 // Percents are typically 0.0-1.0 internally; 
//                 // user types "50", we store 0.5
//                 let val: f64 = input.parse().unwrap_or(0.0);
//                 PropertyValue::Percent(val / 100.0)
//             }
//             PropertyValue::Integer(_) => PropertyValue::Integer(input.parse().unwrap_or(0)),
//             PropertyValue::Unsigned(_) => PropertyValue::Unsigned(input.parse().unwrap_or(0)),
//             PropertyValue::Boolean(_) => {
//                 let s = input.to_lowercase();
//                 PropertyValue::Boolean(s == "true" || s == "1" || s == "yes")
//             }
//             PropertyValue::Text(_) => PropertyValue::Text(input.to_string()),
//         }
//     }

//     pub fn format_value(&self, value: PropertyValue, system: &UnitSystem<C>) -> String {
//         // 1. Optional strict check: Ensure incoming value matches template type
//         // If you prefer, this guard can be removed if handled during `set_value`
//         if std::mem::discriminant(&self.value) != std::mem::discriminant(&value) {
//             return "Type Mismatch".to_string(); // Or handle as an error
//         }

//         // 2. Clean, single-variable match
//         match value {
//             PropertyValue::Number(n) => {
//                 if let Some(cat) = self.unit {
//                     let display_kind = system.display.get(cat);
//                     let converted = display_kind.from_base(n);
//                     format!("{:.2} {}", converted, system.symbol(cat))
//                 } else {
//                     format!("{:.2}", n)
//                 }
//             }
//             PropertyValue::Percent(n) => format!("{:.1}%", n * 100.0),
//             PropertyValue::Integer(i) => i.to_string(),
//             PropertyValue::Unsigned(u) => u.to_string(),
//             PropertyValue::Boolean(b) => b.to_string(),
//             PropertyValue::Text(t) => t,
//         }
//     }
// }

 

// // pub struct Property<K: UnitCategory> {
// //     pub name: DisplayText,
// //     pub key: Option<K>,
// //     // Now extracts from any object that implements Propertied for this config
// //     pub extractor: fn(&dyn Propertied<K>) -> PropertyValue,
// // }


 