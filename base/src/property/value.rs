 
use strum::EnumDiscriminants; 

#[derive(Debug, Clone, PartialEq, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, strum::AsRefStr))]
pub enum PropertyValue {
    ID(String), // allows any type of id
    Text(String),
    Number(f64), 
    Percent(f64),
    Integer(i64),
    Unsigned(u64), 
    Boolean(bool),
}

impl From<String> for PropertyValue {
    fn from(s: String) -> Self { Self::Text(s) }
}
// Enables using &str literals directly
impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self { Self::Text(s.to_string()) }
}
impl From<f64> for PropertyValue {
    fn from(f: f64) -> Self { Self::Number(f) }
}
impl From<i64> for PropertyValue {
    fn from(i: i64) -> Self { Self::Integer(i) }
}
impl From<u64> for PropertyValue {
    fn from(u: u64) -> Self { Self::Unsigned(u) }
}
impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self { Self::Boolean(b) }
}

 
 