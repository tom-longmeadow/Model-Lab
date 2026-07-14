

use super::{ComponentKind, ModelConfig};


type ID<C> = <<C as ModelConfig>::Kind as ComponentKind>::Id;

#[derive(PartialEq, Eq)]
pub enum ModelError<C: ModelConfig> {
    NotFound(ID<C>, C::Kind),
    AlreadyExists(ID<C>, C::Kind),
    ValidationError(ID<C>, C::Kind, String),
    InvalidId(ID<C>, C::Kind),
}
 

impl<C: ModelConfig> std::fmt::Display for ModelError<C>
where
    ID<C>: std::fmt::Display,
    C::Kind: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id, kind) => 
                write!(f, "{} {} not found", kind, id),
            Self::AlreadyExists(id, kind) => 
                write!(f, "{} {} already exists", kind, id),
            Self::ValidationError(id, kind, msg) => 
                write!(f, "{} {} invalid: {}", kind, id, msg),
            Self::InvalidId(id, kind) => 
                write!(f, "invalid id {} for {}", id, kind),
        }
    }
}

impl<C: ModelConfig> std::fmt::Debug for ModelError<C>
where
    ID<C>: std::fmt::Display,
    C::Kind: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
 