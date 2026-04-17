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