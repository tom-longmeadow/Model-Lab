use base::prelude::*;
use crate::language::DisplayText;
use crate::model::registry_hashmap::HashMapRegistry;

/*
    This Example shows how to declare the types and trait implementations to make a model.
    A more concise way to create the same types and traits can be found in the file:
    example_model_using_macros
*/

/// CONFIG
/// This allows us to glue all the types together so that we can start implementing
/// generic traits ergonomically.  The model config is declared here and then as 
/// we declare types, we impl the unit and property cofigs.
pub struct ExampleModelConfig;

/// implements PropertyConfig to explain where to get translated text from
impl PropertyConfig for ExampleModelConfig {
    type Display = DisplayText; 
    type Lang = Locale; 
}


/// UNITS
/// Define the unit categories.  
/// The model may need small and large units of the same base type.  For instance
/// you may have large components measured in meters but small components measured
/// in millimeters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleUnitCategory {
    Length,
    LengthSmall,
    Area,
    Force,
    DynamicViscosity,
    Temperature,
}

/// Implement the UnitCategory trait to mark this as a unit category
impl UnitCategory for ExampleUnitCategory {}

/// Define the kind (type) of unit for each categories. 
/// SimpleUnit contains a base unit and exponent for a unit, like mm^2
/// CompoundUnit contains SimpleUnits for each base unit, like kg*m/s^2
/// TemperatureUnit is special and different than other units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExampleUnitSettings {
    pub length: SimpleUnit,
    pub length_small: SimpleUnit,
    pub area: SimpleUnit,
    pub force: CompoundUnit,
    pub dynamic_viscosity: CompoundUnit,
    pub temperature: TemperatureUnit
}

/// Map the unit category to a specific unit
/// You can use helper functions for SI defaults, or make the unit yourself.
impl UnitSettings<ExampleUnitCategory> for ExampleUnitSettings {
   
    // set up your default units.
    fn default() -> Self {
        Self {
            length: SimpleUnit::length_si(), // use the si default
            length_small: SimpleUnit::length(LengthUnit::Millimeter, 1), // set to millimeter
            area: SimpleUnit::area_si(),
            force: CompoundUnit::force(),
            
            // you can create your own compound units
            dynamic_viscosity: CompoundUnit::new()// Dynamic Viscosity: kg / (m · s)
                .with_mass(MassUnit::Kilogram, 1)
                .with_length(LengthUnit::Meter, -1)
                .with_time(TimeUnit::Second, -1),
            temperature: TemperatureUnit::Celsius,
        }
    }

    
    fn get(&self, category: ExampleUnitCategory) -> UnitKind {
        match category {
            ExampleUnitCategory::Length      => UnitKind::Simple(self.length),
            ExampleUnitCategory::LengthSmall => UnitKind::Simple(self.length_small),
            ExampleUnitCategory::Area        => UnitKind::Simple(self.area),
            ExampleUnitCategory::Force       => UnitKind::Compound(self.force),
            ExampleUnitCategory::DynamicViscosity      => UnitKind::Compound(self.dynamic_viscosity),
            ExampleUnitCategory::Temperature => UnitKind::Temperature(self.temperature),
        }
    }
     
}
 
/// implement UnitConfig for the model to explain what the units are
impl UnitConfig for ExampleModelConfig {
    type UnitCategory = ExampleUnitCategory; 
    type UnitSetting = ExampleUnitSettings; 
}


/// COMPONENT KINDS
/// The components in the model are of different kinds, or types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleKind {
    Point,
    Line,
}

/// COMPONENT ID 
/// This is the ID you want to use for components. You can use unsigned integers, like usize or u64,
/// but for more type saftey use an ID type.  Some ID types are defined but you can define your own.
/// We have defined ID64 as an u64, but if you wanted a custom id you could do it with this macro:
/// component_id_macro!(MyName, u32), or component_id_macro!(ID, u128)
impl ComponentKind for ExampleKind {
    type Id = ID64; // This tells the Registry/Model to expect ID64 as the ComponentId
}

/// COMPONENT DATA
/// Each component kind has associated data. There should be a one to one mapping
/// of the kind enum and data enum. First declare the CompoentData.
#[derive(Debug, Clone, Copy, PartialEq)] 
pub enum ExampleData {
    Point(PointData),
    Line(LineData),
}

/// This is the mapping from kind to data
impl HasKind for ExampleData {
    type Kind = ExampleKind;
    fn kind(&self) -> ExampleKind {
        match self {
            Self::Point(_) => ExampleKind::Point,
            Self::Line(_) => ExampleKind::Line,
        }
    }
}

/// Now declare the structs that hold the data
/// Make data for point
#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct PointData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
/// implement the kind for Point data
impl HasKind for PointData {
    type Kind = ExampleKind;
    fn kind(&self) -> ExampleKind { ExampleKind::Point }
}
/// explain how to make a label for that type tht
impl ComponentData<ExampleModelConfig> for PointData {
    fn kind_name() -> PropertyName<ExampleModelConfig> { PropertyName::new(DisplayText::Point) }
}


/// Make data for Line
#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct LineData {
    pub i: ID64, 
    pub j: ID64, 
}

/// implement the kind  
impl HasKind for LineData {
    type Kind = ExampleKind;
    fn kind(&self) -> ExampleKind { ExampleKind::Line }
}

/// explain how to make a label for that type tht
impl ComponentData<ExampleModelConfig> for LineData {
    fn kind_name() -> PropertyName<ExampleModelConfig> { PropertyName::new(DisplayText::Line) }
}


/// COMPONENT PROPERTIES
/// Create Property binds and implement property get/set/template
/// While the component data contains the fields of each component, 
/// and you can interact with those fields programatically, we need to abstract
/// the properties and assign labels and units, so that we can leverage spreadsheets
/// and other UI displays of the data.  For this we use properties.
/// First make the keys for look ups
impl PointData {
    pub const X_KEY: u64 = PropertySchema::<ExampleModelConfig>::hash_key("PointData::x");
    pub const Y_KEY: u64 = PropertySchema::<ExampleModelConfig>::hash_key("PointData::y");
    pub const Z_KEY: u64 = PropertySchema::<ExampleModelConfig>::hash_key("PointData::z");
}
/// then the properties
impl Propertied<ExampleModelConfig> for PointData {
    fn get_schema() -> PropertyNode<ExampleModelConfig> {
        PropertyNode::Group {
            name: PropertyName::new(DisplayText::Point),
            children: vec![
                PropertyNode::new_number(DisplayText::X, ExampleUnitCategory::Length, Self::X_KEY),
                PropertyNode::new_number(DisplayText::Y, ExampleUnitCategory::Length, Self::Y_KEY),
                PropertyNode::new_number(DisplayText::Z, ExampleUnitCategory::Length, Self::Z_KEY),
            ],
        }
    }
    fn get_value(&self, key: u64) -> Option<PropertyValue> {
        match key {
            Self::X_KEY => Some(PropertyValue::Number(self.x)),
            Self::Y_KEY => Some(PropertyValue::Number(self.y)),
            Self::Z_KEY => Some(PropertyValue::Number(self.z)),
            _ => None,
        }
    }
    fn set_value(&mut self, key: u64, value: PropertyValue) {
        match key {
            Self::X_KEY => { if let PropertyValue::Number(n) = value { self.x = n; } }
            Self::Y_KEY => { if let PropertyValue::Number(n) = value { self.y = n; } }
            Self::Z_KEY => { if let PropertyValue::Number(n) = value { self.z = n; } }
            _ => ()
        }
    }
}

/// Keys for line properties
impl LineData {
    pub const I_KEY: u64 = PropertySchema::<ExampleModelConfig>::hash_key("LineData::i");
    pub const J_KEY: u64 = PropertySchema::<ExampleModelConfig>::hash_key("LineData::j");
}

/// Properties for lines
impl Propertied<ExampleModelConfig> for LineData {
    fn get_schema() -> PropertyNode<ExampleModelConfig> {
        PropertyNode::Group {
            name: PropertyName::new(DisplayText::Line),
            children: vec![
                PropertyNode::new_id(DisplayText::I, Self::I_KEY),
                PropertyNode::new_id(DisplayText::J, Self::J_KEY),
            ],
        }
    }
    fn get_value(&self, key: u64) -> Option<PropertyValue> {
        match key {
            Self::I_KEY => Some(PropertyValue::ID(self.i.to_string())),
            Self::J_KEY => Some(PropertyValue::ID(self.j.to_string())),
            _ => None,
        }
    }
    fn set_value(&mut self, key: u64, value: PropertyValue) {
        match key {
            Self::I_KEY => { if let PropertyValue::ID(s) = value { if let Ok(p) = s.parse() { self.i = p; } } }
            Self::J_KEY => { if let PropertyValue::ID(s) = value { if let Ok(p) = s.parse() { self.j = p; } } }
            _ => ()
        }
    }
}


/// Fill out the Model Config
impl ModelConfig for ExampleModelConfig {
    type Kind = ExampleKind;
    type Data = ExampleData;
    type Registry = HashMapRegistry<ExampleModelConfig>;  // single type arg now
}

/// Make a model type alias for convenience
pub type ExampleModel = Model<ExampleModelConfig>;

/// Create a default ExampleModel
pub fn get_model() -> ExampleModel {
    ExampleModel::new(
        HashMapRegistry::new(),
        UnitSystem::new(ExampleUnitSettings::default()),
    )
}