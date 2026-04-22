use base::{prelude::*, property_key};

use crate::model::registry_hashmap::HashMapRegistry;
 


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
 
/// CONFIG
/// Now we glue all the types together so that we can start implementing
/// generic traits ergonomically.  The master config
pub struct ExampleModelConfig;

/// implements UnitConfig
impl UnitConfig for ExampleModelConfig {
    type UnitCategory = ExampleUnitCategory; 
    type UnitSetting = ExampleUnitSettings; 
}

/// implements PropertyCongig
impl PropertyConfig for ExampleModelConfig {
    type Display = DisplayText; 
    type Lang = Locale; 
}

/// COMPONENT KINDS
/// The components in the model are of different kinds, or types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleKind {
    Point,
    Line,
}

/// COMPONENT ID in Kind
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

/// Now declare the structs that hold the data
#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct PointData {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}


/// Now declare the structs that hold the data
#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct LineData {
    pub i_id: ID64, 
    pub j_id: ID64, 
}

// Property bindinds to connect a property to a key for lookups
impl PointData {
    pub const X_KEY: u64 = property_key!(ExampleModelConfig, "PointDataX");
    pub const Y_KEY: u64 = property_key!(ExampleModelConfig, "PointDataY");
    pub const Z_KEY: u64 = property_key!(ExampleModelConfig, "PointDataZ");
}

impl LineData {
    pub const I_KEY: u64 = property_key!(ExampleModelConfig, "LineDataI");
    pub const J_KEY: u64 = property_key!(ExampleModelConfig, "LineDataJ"); 
}


/// COMPONENT PROPERTIES
/// While the component data contains the fields of each component, 
/// and you can interact with those fields programatically, we need to abstract
/// the properties and assign labels and units, so that we can leverage spreadsheets
/// and other UI displays of the data.  For this we use properties.
/// Implement the Propertied for all ComponentData
impl Propertied<ExampleModelConfig> for PointData {
 
    fn get_schema() -> PropertyNode<ExampleModelConfig> {
        PropertyNode::Group {
            name: PropertyName::new(DisplayText::Point),
            children: vec![
                // 🚀 Pass your custom keys directly into the schema!
                PropertyNode::Leaf(PropertySchema::new(
                    DisplayText::X, PropertyValueDiscriminants::Number, Some(ExampleUnitCategory::Length), Self::X_KEY
                )),
                PropertyNode::Leaf(PropertySchema::new(
                    DisplayText::Y, PropertyValueDiscriminants::Number, Some(ExampleUnitCategory::Length), Self::Y_KEY
                )),
                PropertyNode::Leaf(PropertySchema::new(
                    DisplayText::Z, PropertyValueDiscriminants::Number, Some(ExampleUnitCategory::Length), Self::Z_KEY
                )),
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
            Self::X_KEY => if let PropertyValue::Number(n) = value { self.x = n; },
            Self::Y_KEY => if let PropertyValue::Number(n) = value { self.y = n; },
            Self::Z_KEY => if let PropertyValue::Number(n) = value { self.z = n; },
            _ => (), 
        }
    }
}

impl Propertied<ExampleModelConfig> for LineData {
    fn get_schema() -> PropertyNode<ExampleModelConfig> {
        PropertyNode::Group {
            name: PropertyName::new(DisplayText::Line),
            children: vec![
                // 🚀 Zero strings here either!
                PropertyNode::Leaf(PropertySchema::new_id(DisplayText::I, Self::I_KEY)),
                PropertyNode::Leaf(PropertySchema::new_id(DisplayText::J, Self::J_KEY)),
            ],
        }
    }

    fn get_value(&self, key: u64) -> Option<PropertyValue> {
        match key {
            Self::I_KEY => Some(PropertyValue::ID(self.i_id.to_string())),
            Self::J_KEY => Some(PropertyValue::ID(self.j_id.to_string())),
            _ => None,
        }
    }

    fn set_value(&mut self, key: u64, value: PropertyValue) {
        match key {
            Self::I_KEY => {
                if let PropertyValue::ID(s) = value {
                    if let Ok(parsed_id) = s.parse() { self.i_id = parsed_id; }
                }
            }
            Self::J_KEY => {
                if let PropertyValue::ID(s) = value {
                    if let Ok(parsed_id) = s.parse() { self.j_id = parsed_id; }
                }
            }
            _ => (), 
        }
    }
}


// impl<C: ModelConfig> Propertied<C> for ExampleData {
//     fn get_template() -> Vec<PropertyNode<C>> {
//         // Return only the properties specific to Point or Line
//         vec![/* property nodes for x, y, z or i_id, j_id */]
//     }

//     fn get_value(&self, prop: &Property<C>) -> PropertyValue {
//         // Match on prop and return self.x, self.y, etc.
//     }

//     fn set_value(&mut self, prop: &Property<C>, value: PropertyValue) -> Result<(), PropertyError> {
//         // Update self.x, self.y, etc.
//         Ok(())
//     }
// }

// impl ComponentData for ExampleData {
//     type Kind = ExampleKind;

//     fn kind(&self) -> Self::Kind {
//         match self {
//             Self::Point { .. } => ExampleKind::Point,
//             Self::Line { .. } => ExampleKind::Line,
//         }
//     }
// }

// impl<C, D> Propertied<C> for Component<D>
// where
//     C: ModelConfig,
//     D: ComponentData + Propertied<C>,
// {
//     fn get_template() -> Vec<PropertyNode<C>> {
//         // 1. Create the template for the ID property
//         let mut template = vec![ PropertyNode::new_id_property() ];
        
//         // 2. Append all the properties from the inner data
//         template.extend(D::get_template());
        
//         template
//     }

//     fn get_value(&self, prop: &Property<C>) -> PropertyValue {
//         if prop.is_id_property() {
//             // Intercept and return the component's actual ID
//             return PropertyValue::Id(self.id);
//         }
//         // Otherwise, delegate to the inner data
//         self.data.get_value(prop)
//     }

//     fn set_value(&mut self, prop: &Property<C>, value: PropertyValue) -> Result<(), PropertyError> {
//         if prop.is_id_property() {
//             // Prevent changing the ID if it's read-only, or handle it here
//             return Err(PropertyError::ReadOnly);
//         }
//         // Otherwise, delegate the mutation to the inner data
//         self.data.set_value(prop, value)
//     }
// }

// COMPONENT REGISTRY
// Components are stored in a registry
// for this example we will use a HasMapResgistry alread defined.

// CONFIG
// This allows us to define all the generic types the model will use in one place
// pub struct ExampleConfig;

// impl ModelConfig for ExampleConfig {
//     type Id = ID64;
//     type Data = ExampleData;
//     type Registry = HashMapRegistry<Self::Data>;

//     // Use '=' to assign the concrete types
//     type UnitCategory = ExampleUnitCategory;
//     type UnitSetting = ExampleUnitSettings;  
 
//     // type Display = CommonDisplayText;
//     // type Lang = UnitedStatesLanguage;
// }

// // MODEL
// // Now you can make the model using the configuration
// pub type ExampleModel = Model<ExampleConfig>;

// // INSTANTIATION
// /// Creates a configured model for use in examples.
// pub fn create_example_model() -> ExampleModel {
//     let registry = HashMapRegistry::default();
    
//     let file_settings = ExampleUnitSettings::default();
//     let mut display_settings = ExampleUnitSettings::default();
    
//     display_settings.length = SimpleUnit::Length { 
//         unit: LengthUnit::Foot, 
//         exponent: 1 
//     };

//     let settings = UnitSystem::new(file_settings, display_settings);
//     Model::new(registry, settings)
// }
