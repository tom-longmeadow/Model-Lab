 
use crate::prelude::{DisplayText, Propertied, PropertyConfig, PropertyError, PropertyName, PropertyValue, PropertyValueDiscriminants, UnitSettings, UnitSystem};



#[derive(Debug, Clone)]
pub struct PropertySchema<C: PropertyConfig> {
    pub name: PropertyName<C>, 
    pub kind: PropertyValueDiscriminants,  
    pub key: u64, 
    pub unit: Option<C::UnitCategory>, 
    pub read_only: bool,
}

impl<C: PropertyConfig> PropertySchema<C> {

    pub const fn hash_key(s: &str) -> u64 {
        let mut hash = 14695981039346656037u64;
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            hash ^= bytes[i] as u64;
            hash = hash.wrapping_mul(1099511628211u64);
            i += 1;
        }
        hash
    }

    
    pub fn new(
        name: impl Into<PropertyName<C>>,
        kind: PropertyValueDiscriminants,  
        unit: Option<C::UnitCategory>,
        key: u64, 
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            key,  
            unit,
            read_only: false,
        }
    }

    pub fn new_readonly(
        name: impl Into<PropertyName<C>>,
        kind: PropertyValueDiscriminants,  
        unit: Option<C::UnitCategory>,
        key: u64,  
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            key,  
            unit,
            read_only: true,
        }
    }

    pub fn new_id(
        name: impl Into<PropertyName<C>>,
        key: u64,
    ) -> Self {
        Self::new_readonly(name, PropertyValueDiscriminants::ID, None, key)
    }
 
    pub fn new_id_readonly(
        key: u64,
    ) -> Self {
        let name = PropertyName::from(DisplayText::ID);
        Self::new_readonly(name, PropertyValueDiscriminants::ID, None, key)
    }
 

    pub fn get_formatted_value(&self, component: &impl Propertied<C>, system: &UnitSystem<C>) -> String {
        match component.get_value(self.key) {
            Some(value) => self.format_value(&value, system),
            None => "N/A".to_string(),
        }
    }

    pub fn try_set_from_str(
        &self, 
        object: &mut impl Propertied<C>, 
        input: &str, 
        system: &UnitSystem<C>
    ) -> Result<(), PropertyError> {
        
        // 🌟 Guard A: Leverage the schema's own rules!
        if self.read_only {
            return Err(PropertyError::ReadOnly(self.name.to_string()));
        }

        let parsed_value = self.try_parse(input, system)?;

        let incoming_kind: PropertyValueDiscriminants = (&parsed_value).into();
        if incoming_kind != self.kind {
            return Err(PropertyError::TypeMismatch);
        }

        object.set_value(self.key, parsed_value);
        Ok(())
    }

    pub fn try_get_as_str(&self, object: &impl Propertied<C>, system: &UnitSystem<C>) -> Result<String, PropertyError> {
        match object.get_value(self.key) {
            Some(value) => Ok(self.format_value(&value, system)),
            None => Err(PropertyError::NotFound(self.name.to_string())), 
        }
    }

    fn try_parse(&self, input: &str, system: &UnitSystem<C>) -> Result<PropertyValue, PropertyError> {
        let expected_type_str: &str = self.kind.as_ref();

        let parsed_value = match self.kind {
            PropertyValueDiscriminants::Number => {
                let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
                    expected: expected_type_str.to_string(),
                    received: input.to_string(),
                })?;
                
                if let Some(cat) = self.unit {
                    PropertyValue::Number(system.display.get(cat).to_base(val))
                } else {
                    PropertyValue::Number(val)
                }
            }
            PropertyValueDiscriminants::Percent => {
                let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
                    expected: expected_type_str.to_string(),
                    received: input.to_string(),
                })?;
                PropertyValue::Percent(val / 100.0)
            }
            PropertyValueDiscriminants::Integer => {
                let val: i64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
                    expected: expected_type_str.to_string(),
                    received: input.to_string(),
                })?;
                PropertyValue::Integer(val)
            }
            PropertyValueDiscriminants::Unsigned => {
                let val: u64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
                    expected: expected_type_str.to_string(),
                    received: input.to_string(),
                })?;
                PropertyValue::Unsigned(val)
            }
            PropertyValueDiscriminants::Boolean => {
                let s = input.to_lowercase();
                if s == "true" || s == "1" || s == "yes" {
                    PropertyValue::Boolean(true)
                } else if s == "false" || s == "0" || s == "no" {
                    PropertyValue::Boolean(false)
                } else {
                    return Err(PropertyError::InvalidFormat {
                        expected: "Boolean (true/false, 1/0, yes/no)".to_string(),
                        received: input.to_string(),
                    });
                }
            }
            PropertyValueDiscriminants::Text => PropertyValue::Text(input.to_string()),
            PropertyValueDiscriminants::ID => PropertyValue::ID(input.to_string()),
        };

        Ok(parsed_value)
    }

    fn format_value(&self, value: &PropertyValue, system: &UnitSystem<C>) -> String {
        let incoming_kind: PropertyValueDiscriminants = value.into();
        if incoming_kind != self.kind {
            return "Type Mismatch".to_string(); 
        }

        match value {
            PropertyValue::Number(n) => {
                if let Some(cat) = self.unit {
                    let display_kind = system.display.get(cat);
                    let converted = display_kind.from_base(*n);
                    format!("{:.2} {}", converted, system.symbol(cat))
                } else {
                    format!("{:.2}", n)
                }
            }
            PropertyValue::Percent(n) => format!("{:.1}%", n * 100.0),
            PropertyValue::Integer(i) => i.to_string(),
            PropertyValue::Unsigned(u) => u.to_string(),
            PropertyValue::Boolean(b) => b.to_string(),
            PropertyValue::Text(t) => t.clone(),
            PropertyValue::ID(t) => t.clone(),
        }
    }
}



// #[derive(Debug, Clone)]
// pub struct PropertySchema<C: PropertyConfig> {
//     pub name: PropertyName<C>, 
//     pub kind: PropertyValueDiscriminants,  
//     pub key: u64, 
//     pub unit: Option<C::UnitCategory>, 
// }

// impl<C: PropertyConfig> PropertySchema<C> {

//     /// Computes the FNV-1a hash. Marked as a const fn so it can be 
//     /// executed by the compiler to build blazing-fast lookup keys!
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

//     pub fn new(
//         text: C::Display, 
//         kind: PropertyValueDiscriminants,  
//         unit: Option<C::UnitCategory>
//     ) -> Self {
//         Self {
//             name: PropertyName::new(text),
//             kind,
//             key: Self::hash_key(text.default_text()),
//             unit, 
//         }
//     }

//     pub fn new_str(
//         str: impl Into<String>, 
//         kind: PropertyValueDiscriminants,  
//         unit: Option<C::UnitCategory>
//     ) -> Self { 
//         let name_str = str.into(); 
//         Self {
//             key: Self::hash_key(&name_str),
//             name: PropertyName::new_str(name_str),
//             kind,
//             unit,
//         }
//     } 

//      /// 1. Orchestrated GET: Fetches from the component and processes it for the UI
//     pub fn get_formatted_value(&self, component: &impl Propertied<C>, system: &UnitSystem<C>) -> String {
//         match component.get_value(self.key) {
//             Some(value) => self.format_value(&value, system),
//             None => "N/A".to_string(), // Gracefully handle missing properties!
//         }
//     }

//     /// 2. Orchestrated SET: Parses raw UI/Spreadsheet text, validates it, and updates state
//     pub fn try_set_from_str(
//         &self, 
//         component: &mut impl Propertied<C>, 
//         input: &str, 
//         system: &UnitSystem<C>,
//         read_only_override: bool
//     ) -> Result<(), PropertyError> {
        
//         // Guard A: Check mutability rules
//         if read_only_override {
//             return Err(PropertyError::ReadOnly(self.name.to_string()));
//         }

//         // Guard B: Let the schema handle the heavy parsing and unit base-conversion!
//         let parsed_value = self.try_parse(input, system)?;

//         // Guard C: Ensure the parsed result matches what this schema demands
//         let incoming_kind: PropertyValueDiscriminants = (&parsed_value).into();
//         if incoming_kind != self.kind {
//             return Err(PropertyError::TypeMismatch);
//         }

//         // Execution: All guards passed! Pass the raw payload straight to the trait
//         component.set_value(self.key, parsed_value);
        
//         Ok(())
//     }

//     pub fn parse_as<T: std::str::FromStr>(&self, value: PropertyValue) -> Result<T, PropertyError> {
//         if let PropertyValue::Text(s) = value {
//             s.parse::<T>().map_err(|_| PropertyError::InvalidFormat {  
//                 expected: self.name.to_string(), 
//                 received: s,
//             })
//         } else {
//             Err(PropertyError::TypeMismatch)
//         }
//     }

//     /// Parses unformatted UI text and immediately applies SI unit conversions 
//     /// while fully verifying that the requested payload obeys the template schema.
//    pub fn try_parse(&self, input: &str, system: &UnitSystem<C>) -> Result<PropertyValue, PropertyError> {
    
//         // 🌟 Grab the string representation of the expected variant once (e.g., "Number", "Integer")
//         let expected_type_str: &str = self.kind.as_ref();

//         let parsed_value = match self.kind {
//             PropertyValueDiscriminants::Number => {
//                 let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: expected_type_str.to_string(),
//                     received: input.to_string(),
//                 })?;
                
//                 if let Some(cat) = self.unit {
//                     PropertyValue::Number(system.display.get(cat).to_base(val))
//                 } else {
//                     PropertyValue::Number(val)
//                 }
//             }
//             PropertyValueDiscriminants::Percent => {
//                 let val: f64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: expected_type_str.to_string(),
//                     received: input.to_string(),
//                 })?;
//                 PropertyValue::Percent(val / 100.0)
//             }
//             PropertyValueDiscriminants::Integer => {
//                 let val: i64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: expected_type_str.to_string(),
//                     received: input.to_string(),
//                 })?;
//                 PropertyValue::Integer(val)
//             }
//             PropertyValueDiscriminants::Unsigned => {
//                 let val: u64 = input.parse().map_err(|_| PropertyError::InvalidFormat {
//                     expected: expected_type_str.to_string(),
//                     received: input.to_string(),
//                 })?;
//                 PropertyValue::Unsigned(val)
//             }
//             PropertyValueDiscriminants::Boolean => {
//                 let s = input.to_lowercase();
//                 if s == "true" || s == "1" || s == "yes" {
//                     PropertyValue::Boolean(true)
//                 } else if s == "false" || s == "0" || s == "no" {
//                     PropertyValue::Boolean(false)
//                 } else {
//                     return Err(PropertyError::InvalidFormat {
//                         expected: "Boolean (true/false, 1/0, yes/no)".to_string(),
//                         received: input.to_string(),
//                     });
//                 }
//             }
//             PropertyValueDiscriminants::Text => PropertyValue::Text(input.to_string()),
//         };

//         Ok(parsed_value)
//     }

//     /// Translates SI base math from the database back to human-readable text 
//     /// according to the active units and translation preferences.
//     pub fn format_value(&self, value: &PropertyValue, system: &UnitSystem<C>) -> String {
//         // Guard against UI bugs requesting format rendering on misaligned variants
//         let incoming_kind: PropertyValueDiscriminants = value.into();
//         if incoming_kind != self.kind {
//             return "Type Mismatch".to_string(); 
//         }

//         match value {
//             PropertyValue::Number(n) => {
//                 if let Some(cat) = self.unit {
//                     let display_kind = system.display.get(cat);
//                     let converted = display_kind.from_base(*n);
//                     format!("{:.2} {}", converted, system.symbol(cat))
//                 } else {
//                     format!("{:.2}", n)
//                 }
//             }
//             PropertyValue::Percent(n) => format!("{:.1}%", n * 100.0),
//             PropertyValue::Integer(i) => i.to_string(),
//             PropertyValue::Unsigned(u) => u.to_string(),
//             PropertyValue::Boolean(b) => b.to_string(),
//             PropertyValue::Text(t) => t.clone(),
//         }
//     }
 
// }