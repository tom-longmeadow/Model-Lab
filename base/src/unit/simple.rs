
 
use crate::unit::TemperatureUnit;

use super::{
    MolarUnit, BaseUnit, CurrentUnit, LengthUnit, 
    LuminousIntensityUnit, MassUnit, TimeUnit, Unit
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleUnit {
    Length      { unit: LengthUnit,      exponent: i8 },
    Mass        { unit: MassUnit,        exponent: i8 },
    Time        { unit: TimeUnit,        exponent: i8 }, 
    Current     { unit: CurrentUnit,     exponent: i8 },
    Molar      { unit: MolarUnit,      exponent: i8 },
    LuminousIntensity { unit: LuminousIntensityUnit, exponent: i8 },
    Temperature { unit: TemperatureUnit, exponent: i8 },
}



impl SimpleUnit {

    pub const fn length_si() -> Self {
        Self::Length { unit: LengthUnit::DEFAULT, exponent: 1 }
    }
    pub const fn length(unit: LengthUnit, exponent: i8) -> Self {
        Self::Length { unit, exponent }
    }
 
    pub const fn mass_si() -> Self {
        Self::Mass { unit: MassUnit::DEFAULT, exponent: 1 }
    }
    pub const fn mass(unit: MassUnit, exponent: i8) -> Self {
        Self::Mass { unit, exponent }
    }
 
    pub const fn time_si() -> Self {
        Self::Time { unit: TimeUnit::DEFAULT, exponent: 1 }
    }
    pub const fn time(unit: TimeUnit, exponent: i8) -> Self {
        Self::Time { unit, exponent }
    }

 
    pub const fn current_si() -> Self {
        Self::Current { unit: CurrentUnit::DEFAULT, exponent: 1 }
    }
    pub const fn current(unit: CurrentUnit, exponent: i8) -> Self {
        Self::Current { unit, exponent }
    }

 
    pub const fn molar_si() -> Self {
        Self::Molar { unit: MolarUnit::DEFAULT, exponent: 1 }
    }
    pub const fn molar(unit: MolarUnit, exponent: i8) -> Self {
        Self::Molar { unit, exponent }
    }

 
    pub const fn luminous_intensity_si() -> Self {
        Self::LuminousIntensity { unit: LuminousIntensityUnit::DEFAULT, exponent: 1 }
    }
    pub const fn luminous_intensity(unit: LuminousIntensityUnit, exponent: i8) -> Self {
        Self::LuminousIntensity { unit, exponent }
    }

  
    pub const fn area_si() -> Self {
        Self::Length { unit: LengthUnit::DEFAULT, exponent: 2 }
    }
    pub const fn volume_si() -> Self {
        Self::Length { unit: LengthUnit::DEFAULT, exponent: 3 }
    }
    pub const fn frequency_si() -> Self {
        Self::Time { unit: TimeUnit::DEFAULT, exponent: -1 }
    } 

    pub const fn base(&self) -> BaseUnit {
        match self {
            Self::Length      { .. } => BaseUnit::Length,
            Self::Mass        { .. } => BaseUnit::Mass,
            Self::Time        { .. } => BaseUnit::Time, 
            Self::Current     { .. } => BaseUnit::Current,
            Self::Molar      { .. } => BaseUnit::Molar,
            Self::LuminousIntensity { .. } => BaseUnit::LuminousIntensity,
            Self::Temperature { .. } => BaseUnit::Temperature,
        }
    }

    pub const fn index(&self) -> usize {
        self.base() as usize
    }

    pub const fn exponent(&self) -> i8 {
        match self {
            Self::Length { exponent, .. }      => *exponent,
            Self::Mass   { exponent, .. }      => *exponent,
            Self::Time   { exponent, .. }      => *exponent, 
            Self::Current     { exponent, .. } => *exponent,
            Self::Molar      { exponent, .. } => *exponent,
            Self::LuminousIntensity { exponent, .. } => *exponent,
            Self::Temperature { exponent, .. } => *exponent,
        }
    }

    pub fn to_base(&self, val: f64) -> f64 {
        match self {
            Self::Length { unit, exponent } => unit.to_base(val, *exponent),
            Self::Mass { unit, exponent } => unit.to_base(val, *exponent),
            Self::Time { unit, exponent } => unit.to_base(val, *exponent), 
            Self::Current { unit, exponent } => unit.to_base(val, *exponent),
            Self::Molar { unit, exponent } => unit.to_base(val, *exponent),
            Self::LuminousIntensity { unit, exponent } => unit.to_base(val, *exponent),
            Self::Temperature { unit, exponent } => unit.to_base_delta(val, *exponent),
        }
    }

    pub fn from_base(&self, val: f64) -> f64 {
        match self {
            Self::Length { unit, exponent } => unit.from_base(val, *exponent),
            Self::Mass { unit, exponent } => unit.from_base(val, *exponent),
            Self::Time { unit, exponent } => unit.from_base(val, *exponent), 
            Self::Current { unit, exponent } => unit.from_base(val, *exponent),
            Self::Molar { unit, exponent } => unit.from_base(val, *exponent),
            Self::LuminousIntensity { unit, exponent } => unit.from_base(val, *exponent),
            Self::Temperature { unit, exponent } => unit.from_base_delta(val, *exponent),
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Length { unit, .. } => unit.symbol(),
            Self::Mass { unit, .. } => unit.symbol(),
            Self::Time { unit, .. } => unit.symbol(), 
            Self::Current { unit, .. } => unit.symbol(),
            Self::Molar { unit, .. } => unit.symbol(),
            Self::LuminousIntensity { unit, .. } => unit.symbol(),
            Self::Temperature { unit, .. } => unit.symbol(),
        }
    }

}