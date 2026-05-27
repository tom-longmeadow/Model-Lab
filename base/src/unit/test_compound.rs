use crate::unit::{CompoundUnit, CurrentUnit, LengthUnit, LuminousIntensityUnit, MassUnit, MolarUnit, TemperatureUnit, TimeUnit};

const EPSIL: f64 = 1e-12;

fn test_compound_factor(
    compound: CompoundUnit,
    parts: Vec<(f64, i32)>,
) -> bool {
    let mut calculated_factor = 1.0;
    for (factor, exponent) in parts {
        calculated_factor *= factor.powi(exponent);
    }
    // println!("Calculated = {calculated_factor}");

    let actual_factor = compound.to_base(1.0);
    //println!("to_base = {actual_factor}");

    (actual_factor - calculated_factor).abs() / calculated_factor < EPSIL 
}

#[test]
fn standard_compounds_units() { 

    // Newton: kg^1 * m^1 * s^-2
    assert!(test_compound_factor(
        CompoundUnit::force(), 
        vec![(1.0, 1), (1.0, 1), (1.0, -2)] 
    ));

    // Joule: kg^1 * m^2 * s^-2 (Newton-meter)
    assert!(test_compound_factor(
        CompoundUnit::energy(), 
        vec![(1.0, 1), (1.0, 2), (1.0, -2)] 
    ));

    // Pascal: kg^1 * m^-1 * s^-2 (Newton per square meter)
    assert!(test_compound_factor(
        CompoundUnit::pressure(), 
        vec![(1.0, 1), (1.0, -1), (1.0, -2)] 
    ));


     // Energy: Kilowatt-hours (Force * Length * Time)
    // Simplified here as (Mass * Length^2 * Time^-2)
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_mass(MassUnit::Kilogram, 1)
            .with_length(LengthUnit::Meter, 2)
            .with_time(TimeUnit::Hour, -2),
        vec![(1.0, 1), (1.0, 2), (3600.0, -2)],
    ));

     // Luminous Efficacy: Lumens per Watt
    // [LuminousIntensity]^1 * [Mass]^-1 * [Length]^-2 * [Time]^3
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_luminous_intensity(LuminousIntensityUnit::Candela, 1)
            .with_mass(MassUnit::Kilogram, -1)
            .with_length(LengthUnit::Meter, -2)
            .with_time(TimeUnit::Second, 3),
        vec![(1.0, 1), (1.0, -1), (1.0, -2), (1.0, 3)],
    ));

    // Charge: Ampere-hours
    // [Current]^1 * [Time]^1
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_current(CurrentUnit::Ampere, 1)
            .with_time(TimeUnit::Hour, 1),
        vec![(1.0, 1), (3600.0, 1)],
    ));

    
 
}

#[test]
fn verify_imperial_compounds() {

    assert!(test_compound_factor(
        CompoundUnit::new()
        .with_length(LengthUnit::Foot, 1)
        .with_mass(MassUnit::Pound, 1)
        .with_time(TimeUnit::Second, -2),
        vec![(0.3048, 1), (0.45359237, 1), (1.0, -2)],
    )); 

     // Pressure: PSI (Pounds per Square Inch)
    // [Mass]^1 * [Length]^-2 * [Time]^-2
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_mass(MassUnit::Pound, 1)
            .with_length(LengthUnit::Inch, -2)
            .with_time(TimeUnit::Second, -2),
        vec![(0.45359237, 1), (0.0254, -2), (1.0, -2)],
    ));
 
    // Density: Ounces per cubic inch
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_mass(MassUnit::Ounce, 1)
            .with_length(LengthUnit::Inch, -3),
        vec![(0.028349523125, 1), (0.0254, -3)],
    ));

    // Speed: Miles per hour
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_length(LengthUnit::Mile, 1)
            .with_time(TimeUnit::Hour, -1),
        vec![(1609.344, 1), (3600.0, -1)],
    )); 

}

#[test]
fn temperature_compounds() {
 

    // Ideal Gas Law style unit: (Pressure * Volume) / (Molar * Temp)
    // Simplified to: [Mass]^1 * [Length]^1 * [Time]^-2 * [Molar]^-1 * [Temperature]^-1
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_mass(MassUnit::Kilogram, 1)
            .with_length(LengthUnit::Meter, 1)
            .with_time(TimeUnit::Second, -2)
            .with_molar(MolarUnit::Mole, -1)
            .with_temperature(TemperatureUnit::Kelvin, -1),
        vec![(1.0, 1), (1.0, 1), (1.0, -2), (1.0, -1), (1.0, -1)],
    ));
 
    // Specific Heat Capacity (Imperial): BTU / (lb * °F)
    // [Length]^2 * [Time]^-2 * [Temperature]^-1
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_length(LengthUnit::Foot, 2)
            .with_time(TimeUnit::Second, -2)
            .with_temperature(TemperatureUnit::Fahrenheit, -1),
        vec![(0.3048, 2), (1.0, -2), (0.5555555555555556, -1)],
    ));

    // Thermal Conductivity: Watts per (meter-kelvin)
    // [Mass]^1 * [Length]^1 * [Time]^-3 * [Temperature]^-1
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_mass(MassUnit::Kilogram, 1)
            .with_length(LengthUnit::Meter, 1)
            .with_time(TimeUnit::Second, -3)
            .with_temperature(TemperatureUnit::Kelvin, -1),
        vec![(1.0, 1), (1.0, 1), (1.0, -3), (1.0, -1)],
    ));

    // Faraday Constant style: Ampere-seconds per mole
    // [Current]^1 * [Time]^1 * [Molar]^-1
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_current(CurrentUnit::Ampere, 1)
            .with_time(TimeUnit::Second, 1)
            .with_molar(MolarUnit::Mole, -1),
        vec![(1.0, 1), (1.0, 1), (1.0, -1)],
    ));

    // Molar Heat Capacity: Joules per (mole-kelvin)
    // [Mass]^1 * [Length]^2 * [Time]^-2 * [Molar]^-1 * [Temperature]^-1
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_mass(MassUnit::Kilogram, 1)
            .with_length(LengthUnit::Meter, 2)
            .with_time(TimeUnit::Second, -2)
            .with_molar(MolarUnit::Mole, -1)
            .with_temperature(TemperatureUnit::Kelvin, -1),
        vec![(1.0, 1), (1.0, 2), (1.0, -2), (1.0, -1), (1.0, -1)],
    ));

    // Photometric: Candela-seconds (Luminous Energy approximation)
    // [LuminousIntensity]^1 * [Time]^1
    assert!(test_compound_factor(
        CompoundUnit::new()
            .with_luminous_intensity(LuminousIntensityUnit::Candela, 1)
            .with_time(TimeUnit::Hour, 1),
        vec![(1.0, 1), (3600.0, 1)],
    ));

}

 