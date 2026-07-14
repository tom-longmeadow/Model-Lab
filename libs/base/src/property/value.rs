 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyValueKind {
    ID,
    Text,
    Number,
    Percent,
    Integer,
    Unsigned,
    Boolean,
}

impl std::fmt::Display for PropertyValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

 

#[derive(Debug, Clone, PartialEq)] 
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

impl PropertyValue {
    pub fn kind(&self) -> PropertyValueKind {
        match self {
            Self::ID(_)       => PropertyValueKind::ID,
            Self::Text(_)     => PropertyValueKind::Text,
            Self::Number(_)   => PropertyValueKind::Number,
            Self::Percent(_)  => PropertyValueKind::Percent,
            Self::Integer(_)  => PropertyValueKind::Integer,
            Self::Unsigned(_) => PropertyValueKind::Unsigned,
            Self::Boolean(_)  => PropertyValueKind::Boolean,
        }
    }
}

impl From<&PropertyValue> for PropertyValueKind {
    fn from(v: &PropertyValue) -> Self {
        match v {
            PropertyValue::ID(_)       => PropertyValueKind::ID,
            PropertyValue::Text(_)     => PropertyValueKind::Text,
            PropertyValue::Number(_)   => PropertyValueKind::Number,
            PropertyValue::Percent(_)  => PropertyValueKind::Percent,
            PropertyValue::Integer(_)  => PropertyValueKind::Integer,
            PropertyValue::Unsigned(_) => PropertyValueKind::Unsigned,
            PropertyValue::Boolean(_)  => PropertyValueKind::Boolean,
        }
    }
}


 
 