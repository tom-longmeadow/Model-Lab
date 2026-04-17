

#[derive(Debug, PartialEq)]
pub enum PropertyError {
    /// The property name doesn't exist on this object
    NotFound(String),
    /// The UI sent a Number, but the property is a Boolean
    InvalidType { expected: String, received: String },
    /// The value is the right type, but out of bounds (e.g., -5.0 for voltage)
    InvalidValue(String),
    /// The property is read-only
    ReadOnly(String),
}

impl std::fmt::Display for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "Property '{}' not found", name),
            Self::InvalidType { expected, received } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, received)
            }
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Self::ReadOnly(name) => write!(f, "Property '{}' is read-only", name),
        }
    }
}