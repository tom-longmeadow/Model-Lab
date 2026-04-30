#[cfg(test)]
mod compound_tests {
    use crate::unit::CompoundUnit;
    const EPSIL: f64 = 1e-12;

 

    fn test_compound_factor(
        compound: CompoundUnit, 
        parts: Vec<(f64, i32)>, // (unit_factor, exponent)
        expected_total_factor: f64
    ) -> bool {
        // 1. Calculate factor from parts provided
        let mut calculated_factor = 1.0;
        for (factor, exponent) in parts {
            calculated_factor *= factor.powi(exponent);
        }

        // 2. Get the factor from your CompoundUnit definition
        let actual_factor = compound.to_base(1.0);

        // 3. Compare
        (actual_factor - calculated_factor).abs() / calculated_factor < EPSIL 
    }

}