use std::fmt;
use crate::prelude::{DisplayLanguage, DisplayText, PropertyConfig};



#[derive(Debug, Clone, PartialEq, Eq)] 
pub enum PropertyName<C: PropertyConfig> {
    Text(C::Display),
    String(String),
}

impl<C: PropertyConfig> fmt::Display for PropertyName<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(key) => write!(f, "{}", key.default_text()),
            Self::String(s) => write!(f, "{}", s),  
        }
    }
}

impl<C: PropertyConfig> PropertyName<C> {
    /// Ergonomic helper for localized keys.
    pub fn new(name: C::Display) -> Self {
        Self::Text(name)
    }

    /// Ergonomic helper for raw strings.
    pub fn new_str(name: impl Into<String>) -> Self {
        Self::String(name.into())
    } 
}

// =========================================================================
// SAFE TRAIT IMPLEMENTATIONS (No Coherence Conflicts)
// =========================================================================

// 1. Accept raw application DisplayText keys (Concrete, no overlap)
impl<C: PropertyConfig> From<DisplayText> for PropertyName<C> {
    fn from(d: DisplayText) -> Self {
        Self::Text(d.into()) 
    }
}

// 2. Accept hardcoded &str literals ("Custom Name")
impl<C: PropertyConfig> From<&str> for PropertyName<C> {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

// 3. Accept full heap-allocated strings
impl<C: PropertyConfig> From<String> for PropertyName<C> {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}


// #[derive(Debug, Clone, PartialEq, Eq)] 
// pub enum PropertyName<C: PropertyConfig> {
//     Text(C::Display),
//     String(String),
// }

// impl<C: PropertyConfig> fmt::Display for PropertyName<C> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::Text(key) => write!(f, "{}", key.default_text()),
//             Self::String(s) => write!(f, "{}", s),  
//         }
//     }
// }

// impl<C: PropertyConfig> PropertyName<C> {
//     /// Ergonomic helper for localized keys
//     pub fn new(name: C::Display) -> Self {
//         Self::Text(name)
//     }

//     /// Ergonomic helper for raw strings
//     pub fn new_str(name: impl Into<String>) -> Self {
//         Self::String(name.into())
//     } 
// }

// // 1. Tell Rust that any C::Display is a PropertyName
// impl<C: PropertyConfig> From<C::Display> for PropertyName<C> {
//     fn from(d: C::Display) -> Self {
//         Self::Text(d)
//     }
// }

// // 2. Tell Rust that any raw DisplayText is also a PropertyName
// impl<C: PropertyConfig> From<DisplayText> for PropertyName<C> {
//     fn from(d: DisplayText) -> Self {
//         // This relies on your trait bound: C::Display: From<DisplayText>
//         Self::Text(d.into()) 
//     }
// }

// // 3. Tell Rust that a raw String or &str is a PropertyName
// impl<C: PropertyConfig> From<String> for PropertyName<C> {
//     fn from(s: String) -> Self {
//         Self::String(s)
//     }
// }
// impl<C: PropertyConfig> From<&str> for PropertyName<C> {
//     fn from(s: &str) -> Self {
//         Self::String(s.to_string())
//     }
// }