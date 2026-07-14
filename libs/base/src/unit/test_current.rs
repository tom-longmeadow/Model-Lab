#[cfg(test)]
mod current_tests {
    use crate::unit::{CurrentUnit, Unit};
    const EPSIL: f64 = 1e-12;

    fn unit_ratio(unit_a: CurrentUnit, unit_b: CurrentUnit) -> f64 {
        let a = unit_a.to_base(1.0, 1);
        let b = unit_b.to_base(1.0, 1);
        a / b
    }

    fn test_unit_ratio(unit_a: CurrentUnit, unit_b: CurrentUnit, known_ratio: f64) -> bool {
        let calculated_ratio = unit_ratio(unit_a, unit_b);
        (calculated_ratio - known_ratio).abs() / known_ratio < EPSIL 
    }

    macro_rules! generate_current_test {
        ($base:ident; $($unit:ident, $expected_ratio:expr);* $(;)?) => {
            $(
                #[test]
                #[allow(non_snake_case)]
                fn $unit() {
                    assert!(test_unit_ratio(CurrentUnit::$unit, CurrentUnit::$base, $expected_ratio));
                }
            )*
        };
    }

    mod compare_to_ampere {
        use super::*;
        generate_current_test!(
            Ampere;
            Milliampere,    0.001;
            Microampere,    0.000_001;
            Kiloampere,     1000.0;
        );
    }

    #[test]
    fn current_logic_integrity() {
        // Power of 1000 jumps
        assert!(test_unit_ratio(CurrentUnit::Kiloampere, CurrentUnit::Ampere, 1000.0));
        assert!(test_unit_ratio(CurrentUnit::Ampere, CurrentUnit::Milliampere, 1000.0));
        assert!(test_unit_ratio(CurrentUnit::Milliampere, CurrentUnit::Microampere, 1000.0));

        // Direct cross-check: Kilo to Milli
        assert!(test_unit_ratio(CurrentUnit::Kiloampere, CurrentUnit::Milliampere, 1_000_000.0));
    }
}