

#[derive(Debug, PartialEq)]
pub enum PropertyError {
    NotFound(String),
    TypeMismatch,
    InvalidFormat { 
        expected: String, 
        received: String 
    },
    InvalidValue(String),
    ReadOnly(String),
}

 

impl std::fmt::Display for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Property '{}' not found", name),
            Self::TypeMismatch => write!(f, "Data type mismatch"), 
            Self::InvalidFormat { expected, received } => {
                write!(f, "Invalid format: expected {}, got '{}'", expected, received)
            }
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Self::ReadOnly(name) => write!(f, "Property '{}' is read-only", name),
        }
    }
}