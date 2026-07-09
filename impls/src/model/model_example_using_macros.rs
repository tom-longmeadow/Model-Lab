use base::prelude::*;
use crate::language::DisplayText;
use crate::model::registry_hashmap::HashMapRegistry;

/*
    This Example shows how to declare the types and trait implementations to make a model 
    using predefined macros.
    A more verbose way that doesn't use macros to create the same types and traits can be found in the file:
    example_model, where there are explanations of the process
*/
pub struct ExampleModelConfig;

impl PropertyConfig for ExampleModelConfig {
    type Display = DisplayText;
    type Lang = Locale;
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleUnitCategory {
    Length, LengthSmall, Area, Force, DynamicViscosity, Temperature,
}
impl UnitCategory for ExampleUnitCategory {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExampleUnitSettings {
    pub length: SimpleUnit,
    pub length_small: SimpleUnit,
    pub area: SimpleUnit,
    pub force: CompoundUnit,
    pub dynamic_viscosity: CompoundUnit,
    pub temperature: TemperatureUnit,
}

impl UnitSettings<ExampleUnitCategory> for ExampleUnitSettings {
    fn default() -> Self {
        Self {
            length: SimpleUnit::length_si(),
            length_small: SimpleUnit::length(LengthUnit::Millimeter, 1),
            area: SimpleUnit::area_si(),
            force: CompoundUnit::force(),
            dynamic_viscosity: CompoundUnit::new()
                .with_mass(MassUnit::Kilogram, 1)
                .with_length(LengthUnit::Meter, -1)
                .with_time(TimeUnit::Second, -1),
            temperature: TemperatureUnit::Celsius,
        }
    }
    fn get(&self, category: ExampleUnitCategory) -> UnitKind {
        match category {
            ExampleUnitCategory::Length           => UnitKind::Simple(self.length),
            ExampleUnitCategory::LengthSmall      => UnitKind::Simple(self.length_small),
            ExampleUnitCategory::Area             => UnitKind::Simple(self.area),
            ExampleUnitCategory::Force            => UnitKind::Compound(self.force),
            ExampleUnitCategory::DynamicViscosity => UnitKind::Compound(self.dynamic_viscosity),
            ExampleUnitCategory::Temperature      => UnitKind::Temperature(self.temperature),
        }
    }
}

impl UnitConfig for ExampleModelConfig {
    type UnitCategory = ExampleUnitCategory;
    type UnitSetting = ExampleUnitSettings;
}

 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleKind { Point, Line }

impl ComponentKind for ExampleKind {
    type Id = ID64;
}
 
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExampleData { Point(PointData), Line(LineData) }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointData { pub x: f64, pub y: f64, pub z: f64 }

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineData { pub i: ID64, pub j: ID64 }

// this macro sets up properties for the x, y and z directions and 
// couples them with the unit and the display label.
component_data_macro!(
    PointData, ExampleModelConfig, ExampleKind, ExampleKind::Point,
    group: DisplayText::Point,
    number_fields: {
        x: X_KEY = "PointData::x" => DisplayText::X, ExampleUnitCategory::Length,
        y: Y_KEY = "PointData::y" => DisplayText::Y, ExampleUnitCategory::Length,
        z: Z_KEY = "PointData::z" => DisplayText::Z, ExampleUnitCategory::Length,
    }
);

component_data_macro!(
    LineData, ExampleModelConfig, ExampleKind, ExampleKind::Line,
    group: DisplayText::Line,
    id_fields: {
        i: I_KEY = "LineData::i" => DisplayText::I,
        j: J_KEY = "LineData::j" => DisplayText::J,
    }
);

model_config_macro!(
    ExampleModelConfig, ExampleKind, ExampleData, HashMapRegistry,
    ExampleUnitSettings,
    [
        Point(PointData) => ExampleKind::Point,
        Line(LineData)   => ExampleKind::Line,
    ]
);

pub type ExampleModel = ModelAlias;