#[cfg(test)]
mod mass_tests {
    use crate::unit::{MassUnit, Unit};
    const EPSIL: f64 = 1e-12;

    fn unit_ratio(unit_a: MassUnit, unit_b: MassUnit) -> f64 {
        let a = unit_a.to_base(1.0, 1);
        let b = unit_b.to_base(1.0, 1);
        a / b
    }

    fn test_unit_ratio(unit_a: MassUnit, unit_b: MassUnit, known_ratio: f64) -> bool {
        let calculated_ratio = unit_ratio(unit_a, unit_b);
        (calculated_ratio - known_ratio).abs() / known_ratio < EPSIL 
    }

    macro_rules! generate_mass_test {
        ($base:ident; $($unit:ident, $expected_ratio:expr);* $(;)?) => {
            $(
                #[test]
                #[allow(non_snake_case)]
                fn $unit() {
                    assert!(test_unit_ratio(MassUnit::$unit, MassUnit::$base, $expected_ratio));
                }
            )*
        };
    }

    mod compare_to_kilogram {
        use super::*;
        generate_mass_test!(
            Kilogram;
            Gram,           0.001;
            Milligram,      0.000_001;
            MetricTon,      1000.0;
            Pound,          0.453_592_37;
            Ounce,          0.453_592_37 / 16.0;
            Slug,           14.593_902_9;  
        );
    }

    mod compare_to_pound {
        use super::*;
        generate_mass_test!(
            Pound;
            Ounce,          1.0 / 16.0;
            Kilogram,       1.0 / 0.453_592_37;
            MetricTon,      1000.0 / 0.453_592_37;
        );
    }

    #[test]
    fn mass_logic_integrity() {
        // 16 ounces in a pound
        assert!(test_unit_ratio(MassUnit::Pound, MassUnit::Ounce, 16.0));
        // 1000 grams in a kilogram
        assert!(test_unit_ratio(MassUnit::Kilogram, MassUnit::Gram, 1000.0));
        // 1 Metric Ton is 1000 kg
        assert!(test_unit_ratio(MassUnit::MetricTon, MassUnit::Kilogram, 1000.0));
        
        // Transitivity: Ton -> Pound -> Ounce
        let ton_to_lb = unit_ratio(MassUnit::MetricTon, MassUnit::Pound);
        let lb_to_oz = unit_ratio(MassUnit::Pound, MassUnit::Ounce);
        let ton_to_oz = unit_ratio(MassUnit::MetricTon, MassUnit::Ounce);
        assert!((ton_to_lb * lb_to_oz - ton_to_oz).abs() < EPSIL);
    }
}