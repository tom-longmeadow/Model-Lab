#[cfg(test)]
mod amount_tests {
    use crate::unit::{MolarUnit, Unit};
    const EPSIL: f64 = 1e-12;

    fn unit_ratio(unit_a: MolarUnit, unit_b: MolarUnit) -> f64 {
        let a = unit_a.to_base(1.0, 1);
        let b = unit_b.to_base(1.0, 1);
        a / b
    }

    fn test_unit_ratio(unit_a: MolarUnit, unit_b: MolarUnit, known_ratio: f64) -> bool {
        let calculated_ratio = unit_ratio(unit_a, unit_b);
        (calculated_ratio - known_ratio).abs() / known_ratio < EPSIL 
    }

    macro_rules! generate_amount_test {
        ($base:ident; $($unit:ident, $expected_ratio:expr);* $(;)?) => {
            $(
                #[test]
                #[allow(non_snake_case)]
                fn $unit() {
                    assert!(test_unit_ratio(MolarUnit::$unit, MolarUnit::$base, $expected_ratio));
                }
            )*
        };
    }

    mod compare_to_mole {
        use super::*;
        generate_amount_test!(
            Mole;
            Millimole,    0.001;
            Kilomole,     1000.0;
        );
    }

    #[test]
    fn amount_logic_integrity() {
        // Verify 1000x scaling
        assert!(test_unit_ratio(MolarUnit::Kilomole, MolarUnit::Mole, 1000.0));
        assert!(test_unit_ratio(MolarUnit::Mole, MolarUnit::Millimole, 1000.0));

        // Cross-check: Kilomole to Millimole
        assert!(test_unit_ratio(MolarUnit::Kilomole, MolarUnit::Millimole, 1_000_000.0));
    }
}