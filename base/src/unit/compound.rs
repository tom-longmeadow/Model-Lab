 
use super::{
    MolarUnit, BaseUnit, CurrentUnit, LengthUnit, LuminousIntensityUnit, 
    MassUnit, SimpleUnit, TimeUnit, Unit
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompoundUnit {
    pub components: [SimpleUnit; BaseUnit::COUNT - 1], // BaseUnit::COUNT - 1 because we dont want temperature
}

impl CompoundUnit {

    pub const fn new() -> Self {
        Self {
            components: [
                SimpleUnit::Length { unit: LengthUnit::DEFAULT, exponent: 0 },
                SimpleUnit::Mass   { unit: MassUnit::DEFAULT, exponent: 0 },
                SimpleUnit::Time   { unit: TimeUnit::DEFAULT, exponent: 0 },
                SimpleUnit::Current { unit: CurrentUnit::DEFAULT, exponent: 0 },
                SimpleUnit::Amount  { unit: MolarUnit::DEFAULT, exponent: 0 },
                SimpleUnit::LuminousIntensity { unit: LuminousIntensityUnit::DEFAULT, exponent: 0 },
            ],
        }
    }

    pub const fn with(mut self, component: SimpleUnit) -> Self {
        // This works as long as component.index() is a const fn
        let idx = component.index();
        self.components[idx] = component;
        self
    }

     
    pub const fn with_length(self, unit: LengthUnit, exponent: i8) -> Self {
        self.with(SimpleUnit::Length { unit, exponent })
    } 
    pub const fn with_mass(self, unit: MassUnit, exponent: i8) -> Self {
        self.with(SimpleUnit::Mass { unit, exponent })
    }
 
    pub const fn with_time(self, unit: TimeUnit, exponent: i8) -> Self {
        self.with(SimpleUnit::Time { unit, exponent })
    }
 
    pub const fn with_current(self, unit: CurrentUnit, exponent: i8) -> Self {
        self.with(SimpleUnit::Current { unit, exponent })
    } 

    pub const fn with_amount(self, unit: MolarUnit, exponent: i8) -> Self {
        self.with(SimpleUnit::Amount { unit, exponent })
    }
 
    pub const fn with_luminous_intensity(self, unit: LuminousIntensityUnit, exponent: i8) -> Self {
        self.with(SimpleUnit::LuminousIntensity { unit, exponent })
    }


    /// Convert a value from self to SI base units
    pub fn to_base(&self, val: f64) -> f64 {
        let mut result = val;
        for component in &self.components {
            if component.exponent() != 0 {
                result = component.to_base(result);
            }
        }
        result
    }

    /// Convert a value from SI base units to self
    pub fn from_base(&self, val: f64) -> f64 {
        let mut result = val;
        for component in &self.components {
            if component.exponent() != 0 {
                result = component.from_base(result);
            }
        }
        result
    }

    /// Convert a value from one CompoundUnit to another.
    /// Both must represent the same physical dimension (e.g. Force to Force).
    pub fn convert(&self, val: f64, target: &CompoundUnit) -> f64 {
        let si = self.to_base(val);
        target.from_base(si)
    }

   pub const fn force() -> Self {
        Self::new()
            .with_mass(MassUnit::DEFAULT, 1)
            .with_length(LengthUnit::DEFAULT, 1)
            .with_time(TimeUnit::DEFAULT, -2)
    }

    pub const fn torque() -> Self {
        // Torque has the same base dimensions as Energy (N·m)
        Self::energy()
    }

    pub const fn energy() -> Self {
        Self::new()
            .with_mass(MassUnit::DEFAULT, 1)
            .with_length(LengthUnit::DEFAULT, 2) // Meter squared
            .with_time(TimeUnit::DEFAULT, -2)
    }

    pub const fn pressure() -> Self {
        Self::new()
            .with_mass(MassUnit::DEFAULT, 1)
            .with_length(LengthUnit::DEFAULT, -1)
            .with_time(TimeUnit::DEFAULT, -2)
    }

    pub const fn power() -> Self {
        Self::new()
            .with_mass(MassUnit::DEFAULT, 1)
            .with_length(LengthUnit::DEFAULT, 2)
            .with_time(TimeUnit::DEFAULT, -3)
    }

    pub const fn velocity() -> Self {
        Self::new()
            .with_length(LengthUnit::DEFAULT, 1)
            .with_time(TimeUnit::DEFAULT, -1)
    }

    pub const fn acceleration() -> Self {
        Self::new()
            .with_length(LengthUnit::DEFAULT, 1)
            .with_time(TimeUnit::DEFAULT, -2)
    }

    


     
}