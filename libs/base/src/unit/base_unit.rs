
use crate::enum_index_macro;
use crate::base_unit_macro;
use crate::temperature_unit_macro; 

enum_index_macro!(BaseUnit {
    Length, 
    Mass,    
    Time,    
    Current,
    Molar,
    LuminousIntensity,
    Temperature,
});

// Trait for unit.
pub trait Unit: Sized + Default + Copy + PartialEq {
     
    const COUNT: usize;
    const DEFAULT: Self;
  
    fn symbol(&self) -> &'static str;
    fn all_variants() -> Vec<Self>;
    fn to_base(&self, val: f64, power: i8) -> f64;
    fn from_base(&self, val: f64, power: i8) -> f64;
     
} 


base_unit_macro!(LengthUnit {
    Meter = (1.0, "m"),
    Millimeter = (0.001, "mm"),
    Centimeter = (0.01, "cm"),
    Decimeter = (0.1, "dm"),
    Kilometer = (1000.0, "km"),
    Micrometer = (1e-6, "µm"),
    Nanometer = (1e-9, "nm"),
    Inch = (0.0254, "in"),
    Foot = (0.3048, "ft"),
    Yard = (0.9144, "yd"),
    Mile = (1609.344, "mi"),
    NauticalMile = (1852.0, "nmi"),
    Fathom = (1.8288, "fath"),
    Mil = (0.0000254, "mil"),
    AstronomicalUnit = (149_597_870_700.0, "au"),
    LightYear = (9_460_730_472_580_800.0, "ly"),
    Parsec = (30_856_775_814_913_673.0, "pc"),
    Angstrom = (1e-10, "Å"),
});
  


base_unit_macro!(MassUnit {
    Kilogram = (1.0, "kg"),
    Gram = (0.001, "g"),
    Milligram = (1e-6, "mg"),
    MetricTon = (1000.0, "t"),
    Pound = (0.45359237, "lb"),
    Ounce = (0.028349523125, "oz"),
    Slug = (14.5939029, "slug"),
});

base_unit_macro!(TimeUnit {
    Second = (1.0, "s"),
    Millisecond = (0.001, "ms"),
    Microsecond = (1e-6, "µs"),
    Nanosecond = (1e-9, "ns"),
    Minute = (60.0, "min"),
    Hour = (3600.0, "h"),
    Day = (86400.0, "d"),
    Week = (604800.0, "wk"),
});

base_unit_macro!(CurrentUnit {
    Ampere = (1.0, "A"),
    Milliampere = (0.001, "mA"),
    Microampere = (1e-6, "µA"),
    Kiloampere = (1000.0, "kA"),
});

base_unit_macro!(MolarUnit {
    Mole = (1.0, "mol"),
    Millimole = (0.001, "mmol"),
    Kilomole = (1000.0, "kmol"),
});

base_unit_macro!(LuminousIntensityUnit {
    Candela = (1.0, "cd"),
});

temperature_unit_macro!(TemperatureUnit {
    Kelvin = (1.0, 0.0, "K"),
    Celsius = (1.0, 273.15, "°C"), 
    Fahrenheit = (5.0 / 9.0, 255.37222222222222, "°F"), 
    Rankine = (5.0 / 9.0, 0.0, "°R"),
});


