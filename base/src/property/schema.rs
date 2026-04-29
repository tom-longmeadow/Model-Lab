 use crate::{prelude::{Propertied, PropertyConfig, PropertyError, PropertyName, PropertyValue, 
    PropertyValueKind, UnitSystem}, unit::UnitSettings};

#[derive(Debug)]
pub struct PropertySchema<C: PropertyConfig> {
    pub name: PropertyName<C>, 
    pub kind: PropertyValueKind,  
    pub key: u64, 
    pub unit: Option<C::UnitCategory>, 
    pub read_only: bool,
}

impl<C: PropertyConfig> Clone for PropertySchema<C> {
    fn clone(&self) -> Self {
        Self {
            name:      self.name.clone(),
            kind:      self.kind,
            key:       self.key,
            unit:      self.unit,
            read_only: self.read_only,
        }
    }
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

    pub fn new(name: C::Display, kind: PropertyValueKind, unit: Option<C::UnitCategory>, key: u64) -> Self {
        Self { name: PropertyName::new(name), kind, unit, key, read_only: false }
    }

    

    pub fn new_readonly(name: C::Display, kind: PropertyValueKind, unit: Option<C::UnitCategory>, key: u64) -> Self {
        Self { name: PropertyName::new(name), kind, unit, key, read_only: true }
    }

    pub fn new_number(name: C::Display, unit: C::UnitCategory, key: u64) -> Self {
        Self::new(name, PropertyValueKind::Number, Some(unit), key)
    }

    pub fn new_text(name: C::Display, key: u64) -> Self {
        Self::new(name, PropertyValueKind::Text,None, key)
    } 
    pub fn new_id(name: C::Display, key: u64) -> Self {
        Self::new(name, PropertyValueKind::ID, None, key)
    }

    pub fn new_id_readonly(name: C::Display, key: u64) -> Self {
        Self::new_readonly(name, PropertyValueKind::ID, None, key)
    }

    pub fn new_str(name: impl Into<String>, kind: PropertyValueKind, unit: Option<C::UnitCategory>, key: u64) -> Self {
        Self { name: PropertyName::new_str(name), kind, unit, key, read_only: false }
    }

     pub fn new_number_str(name: impl Into<String>, unit: C::UnitCategory, key: u64) -> Self {
        Self::new_str(name, PropertyValueKind::Number, Some(unit), key)
    }

    pub fn new_text_str(name: impl Into<String>, key: u64) -> Self {
        Self::new_str(name, PropertyValueKind::Text,None, key)
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
        if self.read_only {
            return Err(PropertyError::ReadOnly(self.name.to_string()));
        }

        let parsed_value = self.try_parse(input, system)?;

        let incoming_kind = PropertyValueKind::from(&parsed_value);
        if incoming_kind != self.kind {
            return Err(PropertyError::TypeMismatch { 
                expected: self.kind, 
                got: incoming_kind,
            });
        }

        object.set_value(self.key, parsed_value);
        Ok(())
    }

    // pub fn try_get_as_str(&self, object: &impl Propertied<C>, system: &UnitSystem<C>) -> Result<String, PropertyError> {
    //     match object.get_value(self.key) {
    //         Some(value) => Ok(self.format_value(&value, system)),
    //         None => Err(PropertyError::NotFound(self.name.to_string())), 
    //     }
    // }

      pub fn try_get_as_str(&self, object: &impl Propertied<C>, system: &UnitSystem<C>) -> Result<String, PropertyError> {
        let value = object.get_value(self.key)
            .ok_or_else(|| PropertyError::NotFound(self.name.to_string()))?;

        let incoming_kind = PropertyValueKind::from(&value);
        if incoming_kind != self.kind {
            return Err(PropertyError::TypeMismatch {
                expected: self.kind,
                got: incoming_kind,
            });
        }

        Ok(self.format_value(&value, system))
    }

    fn format_value(&self, value: &PropertyValue, system: &UnitSystem<C>) -> String {
        match value {
            PropertyValue::Number(n) => {
                if let Some(cat) = self.unit {
                    let display_kind = system.display.get(cat);
                    let converted = display_kind.from_base(*n);
                    //format!("{:.2} {}", converted, system.symbol(cat))
                    format!("{:.2}", converted)
                } else {
                    format!("{:.2}", n)
                }
            }
            PropertyValue::Percent(n)  => format!("{:.1}%", n * 100.0),
            PropertyValue::Integer(i)  => i.to_string(),
            PropertyValue::Unsigned(u) => u.to_string(),
            PropertyValue::Boolean(b)  => b.to_string(),
            PropertyValue::Text(t)     => t.clone(),
            PropertyValue::ID(t)       => t.clone(),
        }
    }


    fn try_parse(&self, input: &str, system: &UnitSystem<C>) -> Result<PropertyValue, PropertyError> {
        let parsed_value = match self.kind {
            PropertyValueKind::Number => {
                let val: f64 = input.parse().map_err(|_| PropertyError::ParseFailed { 
                    expected: self.kind, 
                    raw: input.to_string(),
                })?;
                if let Some(cat) = self.unit {
                    PropertyValue::Number(system.display.get(cat).to_base(val))
                } else {
                    PropertyValue::Number(val)
                }
            }
            PropertyValueKind::Percent => {
                let val: f64 = input.parse().map_err(|_| PropertyError::ParseFailed { 
                    expected: self.kind, 
                    raw: input.to_string(),
                })?;
                PropertyValue::Percent(val / 100.0)
            }
            PropertyValueKind::Integer => {
                let val: i64 = input.parse().map_err(|_| PropertyError::ParseFailed { 
                    expected: self.kind, 
                    raw: input.to_string(),
                })?;
                PropertyValue::Integer(val)
            }
            PropertyValueKind::Unsigned => {
                let val: u64 = input.parse().map_err(|_| PropertyError::ParseFailed { 
                    expected: self.kind, 
                    raw: input.to_string(),
                })?;
                PropertyValue::Unsigned(val)
            }
            PropertyValueKind::Boolean => {
                match input.to_lowercase().as_str() {
                    "true" | "1" | "yes" => PropertyValue::Boolean(true),
                    "false" | "0" | "no" => PropertyValue::Boolean(false),
                    _ => return Err(PropertyError::ParseFailed { 
                        expected: PropertyValueKind::Boolean, 
                        raw: input.to_string(),
                    }),
                }
            }
            PropertyValueKind::Text => PropertyValue::Text(input.to_string()),
            PropertyValueKind::ID   => PropertyValue::ID(input.to_string()),
        };

        Ok(parsed_value)
    }

    
}

 