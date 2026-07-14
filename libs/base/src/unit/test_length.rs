

#[cfg(test)]
mod tests {
    use crate::unit::{LengthUnit, Unit};
     
    const EPSIL: f64 = 1e-12;


    fn unit_ratio(unit_a: LengthUnit, unit_b: LengthUnit) -> f64 {

        let a = unit_a.to_base(1.0, 1);
        let b = unit_b.to_base(1.0, 1);
        
        let calculated_ratio = a / b;
        calculated_ratio
    }

    fn test_unit_ratio(unit_a: LengthUnit, unit_b: LengthUnit, known_ratio: f64) -> bool {
        let calculated_ratio = unit_ratio(unit_a, unit_b);
        (calculated_ratio - known_ratio).abs() / known_ratio < EPSIL 
    }

     
 
    macro_rules! generate_unit_test {
        ($base:ident; $($unit:ident, $expected_ratio:expr);* $(;)?) => {
            $(
                #[test]
                #[allow(non_snake_case)]
                fn $unit() {
                    // Use LengthUnit:: to explicitly refer to the enum variant
                    assert!(test_unit_ratio(LengthUnit::$unit, LengthUnit::$base, $expected_ratio));
                }
            )*
        };
    }

    mod compare_to_inch {
        use super::*;
        generate_unit_test!(
            Inch;  // all the units below will be compared to Inch
            
            // Metric vs Inch
            Meter,            1.0 / 0.0254;
            Millimeter,       0.001 / 0.0254;
            Centimeter,       0.01 / 0.0254;
            Decimeter,        0.1 / 0.0254;
            Kilometer,        1000.0 / 0.0254;
            Micrometer,       0.000_001 / 0.0254;
            Nanometer,        0.000_000_001 / 0.0254;
            Angstrom,         0.000_000_000_1 / 0.0254;

            // Imperial vs Inch
            Inch,             1.0;
            Foot,             12.0;
            Yard,             36.0;
            Mile,             63360.0;
            NauticalMile,     1852.0 / 0.0254;
            Fathom,           72.0;
            Mil,              0.001;

            // Astronomical vs Inch
            AstronomicalUnit, 149_597_870_700.0 / 0.0254;
            LightYear,        9_460_730_472_580_800.0 / 0.0254;
            Parsec,           30_856_775_814_913_673.0 / 0.0254;
        );
    }

    mod compare_to_meter {
        use super::*;
        generate_unit_test!(
            Meter; // Testing against the internal base unit
            Millimeter,       0.001;
            Centimeter,       0.01;
            Decimeter,        0.1;
            Kilometer,        1000.0;
            Micrometer,       0.000_001;
            Nanometer,        0.000_000_001;
            Inch,             0.0254; 
            Foot,             0.3048;
            Yard,             0.9144;
            Mile,             1609.344;
            NauticalMile,     1852.0;
            AstronomicalUnit, 149_597_870_700.0;
        );
    }

    
     #[test]
    fn verify_logic_integrity() {
         // 1. Identity: Any unit to itself is 1.0 (Pick a few non-inch units)
        assert!(test_unit_ratio(LengthUnit::Meter, LengthUnit::Meter, 1.0));
        assert!(test_unit_ratio(LengthUnit::Mile, LengthUnit::Mile, 1.0));

        // 2. Symmetry: Reverse the ratio (Inch to Foot)
        // If Foot/Inch is 12, then Inch/Foot must be 1/12
        assert!(test_unit_ratio(LengthUnit::Inch, LengthUnit::Foot, 1.0 / 12.0));
        
        // Symmetry: Yard to Meter
        assert!(test_unit_ratio(LengthUnit::Meter, LengthUnit::Yard, 1.0 / 0.9144));

        // 3. Transitivity: Check a chain (Mile -> Yard -> Foot)
        assert!(test_unit_ratio(LengthUnit::Mile, LengthUnit::Yard, 1760.0)); 
        assert!(test_unit_ratio(LengthUnit::Yard, LengthUnit::Foot, 3.0)); 
        assert!(test_unit_ratio(LengthUnit::Mile, LengthUnit::Foot, 5280.0));

        let m_to_y = unit_ratio(LengthUnit::Mile, LengthUnit::Yard);
        let y_to_f = unit_ratio(LengthUnit::Yard, LengthUnit::Foot);
        let m_to_f = unit_ratio(LengthUnit::Mile, LengthUnit::Foot);
 
        assert!((m_to_y * y_to_f - m_to_f).abs() < EPSIL, "Transitivity chain failed");
    }

    #[test]
    fn metric_scaling() {
        assert!(test_unit_ratio(LengthUnit::Kilometer, LengthUnit::Meter, 1000.0));
        assert!(test_unit_ratio(LengthUnit::Meter, LengthUnit::Millimeter, 1000.0));
        assert!(test_unit_ratio(LengthUnit::Meter, LengthUnit::Centimeter, 100.0));
        assert!(test_unit_ratio(LengthUnit::Meter, LengthUnit::Decimeter, 10.0));
        assert!(test_unit_ratio(LengthUnit::Millimeter, LengthUnit::Micrometer, 1000.0));
        assert!(test_unit_ratio(LengthUnit::Micrometer, LengthUnit::Nanometer, 1000.0));
        assert!(test_unit_ratio(LengthUnit::Nanometer, LengthUnit::Angstrom, 10.0));
    }

  

    #[test]
    fn astronomical_cross_check() {
        // 1 ly = 9,460,730,472,580,800 m / 149,597,870,700 m/au
        // This is approx 63241.07708...
        assert!(test_unit_ratio(LengthUnit::LightYear, LengthUnit::AstronomicalUnit, 9_460_730_472_580_800.0 / 149_597_870_700.0));
        
        // 1 pc = (648,000 / PI) AU. To compare pc to ly: (pc in meters) / (ly in meters)
        let pc_in_meters = (648_000.0 / std::f64::consts::PI) * 149_597_870_700.0;
        let ly_in_meters = 9_460_730_472_580_800.0;
        assert!(test_unit_ratio(LengthUnit::Parsec, LengthUnit::LightYear, pc_in_meters / ly_in_meters));
    }

    #[test]
    fn maritime_check() {
        // 1 Fathom is exactly 6 feet
        assert!(test_unit_ratio(LengthUnit::Fathom, LengthUnit::Foot, 6.0));
        
        // 1 Nautical Mile (1852m) to Statute Mile (1609.344m)
        // 1852 / 1609.344 is approx 1.150779448...
        assert!(test_unit_ratio(LengthUnit::NauticalMile, LengthUnit::Mile, 1852.0 / 1609.344));
    }

    #[test]
    fn precision_mil_check() {
        // 1000 Mils = 1 Inch
        assert!(test_unit_ratio(LengthUnit::Inch, LengthUnit::Mil, 1000.0));
        // 1 Mil is exactly 0.0254 mm
        assert!(test_unit_ratio(LengthUnit::Mil, LengthUnit::Millimeter, 0.0254));
    }
}

    
     