

pub mod prelude {
    pub use super::Property;
}

pub struct Property {
    pub name: String,         // "Motor Voltage", "Joint Stress"
    pub value: String,        // "12.5" (formatted as text)
    pub unit: Option<String>, // "V", "kN", "m/s"
    pub description: Option<String>, // Tooltip/Popup text
}