

/// In an application you may need a length unit in (m) for large lengths and
/// another unit in (mm) for small lengths.  Create an enum for all you unit
/// categories and implement this trait on your enum.
pub trait UnitCategory: Clone + Copy + PartialEq + Eq {}