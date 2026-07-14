

use crate::property::value::PropertyValueKind;

#[derive(PartialEq)]  // remove Debug from derive
pub enum PropertyError {
    NotFound(String),
    TypeMismatch { expected: PropertyValueKind, got: PropertyValueKind },
    ParseFailed { expected: PropertyValueKind, raw: String },
    InvalidValue(String),
    ReadOnly(String),
}

impl std::fmt::Display for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => 
                write!(f, "property '{}' not found", name),
            Self::TypeMismatch { expected, got } => write!(f, "expected {}, got {}", expected, got),
            Self::ParseFailed { expected, raw } => 
                write!(f, "could not parse '{}' as {}", raw, expected),
            Self::InvalidValue(msg) => 
                write!(f, "invalid value: {}", msg),
            Self::ReadOnly(name) => 
                write!(f, "property '{}' is read-only", name),
        }
    }
}

impl std::fmt::Debug for PropertyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for PropertyError {}