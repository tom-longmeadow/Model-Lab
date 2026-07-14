#[cfg(test)]
mod time_tests {
    use crate::unit::{TimeUnit, Unit};
    const EPSIL: f64 = 1e-12;

    fn unit_ratio(unit_a: TimeUnit, unit_b: TimeUnit) -> f64 {
        let a = unit_a.to_base(1.0, 1);
        let b = unit_b.to_base(1.0, 1);
        a / b
    }

    fn test_unit_ratio(unit_a: TimeUnit, unit_b: TimeUnit, known_ratio: f64) -> bool {
        let calculated_ratio = unit_ratio(unit_a, unit_b);
        (calculated_ratio - known_ratio).abs() / known_ratio < EPSIL 
    }

    macro_rules! generate_time_test {
        ($base:ident; $($unit:ident, $expected_ratio:expr);* $(;)?) => {
            $(
                #[test]
                #[allow(non_snake_case)]
                fn $unit() {
                    assert!(test_unit_ratio(TimeUnit::$unit, TimeUnit::$base, $expected_ratio));
                }
            )*
        };
    }

    mod compare_to_second {
        use super::*;
        generate_time_test!(
            Second;
            Millisecond,    0.001;
            Microsecond,    0.000_001;
            Nanosecond,     0.000_000_001;
            Minute,         60.0;
            Hour,           3600.0;
            Day,            86400.0;
            Week,           604800.0;
        );
    }

    mod compare_to_hour {
        use super::*;
        generate_time_test!(
            Hour;
            Second,         1.0 / 3600.0;
            Minute,         1.0 / 60.0;
            Day,            24.0;
            Week,           168.0;
        );
    }

    #[test]
    fn time_logic_integrity() {
        // Basic scaling
        assert!(test_unit_ratio(TimeUnit::Minute, TimeUnit::Second, 60.0));
        assert!(test_unit_ratio(TimeUnit::Hour, TimeUnit::Minute, 60.0));
        assert!(test_unit_ratio(TimeUnit::Day, TimeUnit::Hour, 24.0));
        assert!(test_unit_ratio(TimeUnit::Week, TimeUnit::Day, 7.0));

        // Sub-second precision
        assert!(test_unit_ratio(TimeUnit::Second, TimeUnit::Millisecond, 1000.0));
        assert!(test_unit_ratio(TimeUnit::Millisecond, TimeUnit::Microsecond, 1000.0));
        assert!(test_unit_ratio(TimeUnit::Microsecond, TimeUnit::Nanosecond, 1000.0));

        // Transitivity: Week -> Day -> Hour -> Minute -> Second
        let week_to_sec = unit_ratio(TimeUnit::Week, TimeUnit::Second);
        let expected_sec = 7.0 * 24.0 * 60.0 * 60.0; // 604800
        assert!((week_to_sec - expected_sec).abs() < EPSIL);
    }
}