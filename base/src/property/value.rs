use super::Property;


pub enum PropertyValue {
    Text(String),

    /// A measurement includes the value and the specific unit symbol
    Measurement {
        value: f64,
        unit: String, // Or a more robust Unit identifier
    },

    Percent(f64),
    Number(f64),
    Integer(i64),
    Boolean(bool),

    /// For Enums: contains the current selection and a list of all possible options
    Choice { 
        selected_index: usize, 
        options: Vec<String> 
    },

    /// supports a tree of properties
    Group(Vec<Property>),
}
 