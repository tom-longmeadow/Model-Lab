use std::collections::HashMap;
use super::{Unit, UnitKind, UnitDimensions};

pub trait UnitCategory {
    /// The specific enum of units for this category (e.g., LengthUnit)
    type UnitType: Unit; 

    /// A stable, unique string for serialization/lookup (e.g., "length")
    fn id() -> &'static str;

    /// The physical nature of the unit (Simple, Compound, or Temperature)
    fn kind() -> UnitKind;
    
    /// The human-readable name for UI (e.g., "Length")
    fn category_name() -> &'static str;

    /// The default unit to use if none is specified
    fn default_unit() -> Self::UnitType;

    /// Precision hint for UI (defaults to 2 decimal places)
    fn default_precision() -> usize { 2 }

    /// Returns the list of all available units in this category
    fn allowed_units() -> Vec<Self::UnitType> {
        Self::UnitType::all_variants()
    }

    /// Automatically resolves the physics vector for this category
    fn dimensions() -> UnitDimensions {
        Self::kind().dimensions()
    }
}

pub trait DynamicCategory: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn kind(&self) -> UnitKind;
    fn symbols(&self) -> Vec<&'static str>;
    fn default_symbol(&self) -> &'static str;
    
    // The "Magic" conversion function
    fn to_si(&self, value: f64, symbol: &str) -> f64;
    fn from_si(&self, si_value: f64, symbol: &str) -> f64;
}

// Add Send + Sync bounds to the blanket implementation
impl<T: UnitCategory + Send + Sync> DynamicCategory for T {
    fn id(&self) -> &'static str { T::id() }
    fn name(&self) -> &'static str { T::category_name() }
    fn kind(&self) -> UnitKind { T::kind() }
    
    fn symbols(&self) -> Vec<&'static str> {
        T::UnitType::all_variants().iter().map(|u| u.symbol()).collect()
    }

    fn default_symbol(&self) -> &'static str {
        T::default_unit().symbol()
    }

    fn to_si(&self, value: f64, symbol: &str) -> f64 {
        // Look up unit; fallback to default if not found
        let unit = T::UnitType::all_variants()
            .into_iter()
            .find(|u| u.symbol() == symbol)
            .unwrap_or_else(T::default_unit);
        
        let power = match T::kind() {
            UnitKind::Simple(_, p) => p,
            _ => 1,
        };
        unit.to_si(value, power)
    }

    fn from_si(&self, si_value: f64, symbol: &str) -> f64 {
        let unit = T::UnitType::all_variants()
            .into_iter()
            .find(|u| u.symbol() == symbol)
            .unwrap_or_else(T::default_unit);

        let power = match T::kind() {
            UnitKind::Simple(_, p) => p,
            _ => 1,
        };
        unit.from_si(si_value, power)
    }
}
