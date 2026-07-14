#[cfg(test)]
mod temperature_tests {
    use crate::unit::{TemperatureUnit, Unit};
    const EPSIL: f64 = 1e-10; // Slightly larger epsilon for degree conversions

    fn test_temp_absolute_zero(unit: TemperatureUnit, zero_value: f64) -> bool {
        // If to_base is correct, converting the "zero point" of any unit
        // should equal exactly 0.0 Kelvin.
        let absolute_zero_in_kelvin = unit.to_base(zero_value, 1);
        absolute_zero_in_kelvin.abs() < EPSIL
    }

    fn test_temp_interval(unit: TemperatureUnit, known_kelvin_per_degree: f64) -> bool {
        // We compare the difference between 1.0 and 0.0 to strip the offset
        let val_at_1 = unit.to_base(1.0, 1);
        let val_at_0 = unit.to_base(0.0, 1);
        
        let degree_size = val_at_1 - val_at_0;
        (degree_size - known_kelvin_per_degree).abs() < EPSIL
    }

    fn test_temp_fixed_point(unit: TemperatureUnit, value: f64, expected_kelvin: f64) -> bool {
        let result_kelvin = unit.to_base(value, 1);
        (result_kelvin - expected_kelvin).abs() < EPSIL
    }

    fn test_temp_round_trip(unit: TemperatureUnit, start_value: f64) -> bool {
        // 1. Convert original value to Kelvin
        let kelvin = unit.to_base(start_value, 1);
        
        // 2. Convert that Kelvin value back to the original unit
        // (Assuming you have a from_base method)
        let end_value = unit.from_base(kelvin, 1);
        
        // 3. Check if we returned to the starting point
        (start_value - end_value).abs() < EPSIL
    }

    #[test]
    fn verify_temp_round_trips() {
        let units = [
            TemperatureUnit::Celsius,
            TemperatureUnit::Fahrenheit,
            TemperatureUnit::Rankine,
            TemperatureUnit::Kelvin,
        ];

        for &unit in units.iter() {
            // Test Room Temperature
            assert!(test_temp_round_trip(unit, 20.0), "Failed round trip for {:?} at 20.0", unit);
            
            // Test a very high temperature
            assert!(test_temp_round_trip(unit, 1500.0), "Failed round trip for {:?} at 1500.0", unit);
            
            // Test a negative temperature (except for Kelvin/Rankine which shouldn't be negative)
            if unit == TemperatureUnit::Celsius || unit == TemperatureUnit::Fahrenheit {
                assert!(test_temp_round_trip(unit, -40.0), "Failed round trip for {:?} at -40.0", unit);
            }
        }
    }

    #[test]
    fn verify_celsius_fahrenheit_parity() {
        // -40 Celsius must equal -40 Fahrenheit
        let c_at_neg_40 = TemperatureUnit::Celsius.to_base(-40.0, 1);
        let f_at_neg_40 = TemperatureUnit::Fahrenheit.to_base(-40.0, 1);
        
        assert!((c_at_neg_40 - f_at_neg_40).abs() < EPSIL);
    }

    #[test]
    fn verify_temp_offsets() {
        assert!(test_temp_absolute_zero(TemperatureUnit::Celsius, -273.15));
        assert!(test_temp_absolute_zero(TemperatureUnit::Fahrenheit, -459.67));
        assert!(test_temp_absolute_zero(TemperatureUnit::Rankine, 0.0));
    }

    #[test]
    fn verify_temp_scales() {
        // 1 degree Celsius is exactly 1 Kelvin
        assert!(test_temp_interval(TemperatureUnit::Celsius, 1.0));
        
        // 1 degree Fahrenheit is 5/9 (0.555...) Kelvin
        assert!(test_temp_interval(TemperatureUnit::Fahrenheit, 5.0 / 9.0));
        
        // 1 degree Rankine is 5/9 Kelvin
        assert!(test_temp_interval(TemperatureUnit::Rankine, 5.0 / 9.0));
    }

    #[test]
    fn verify_boiling_points() {
        // Water boils at 373.15 Kelvin
        assert!(test_temp_fixed_point(TemperatureUnit::Celsius, 100.0, 373.15));
        
        // Water boils at 212.0 Fahrenheit
        assert!(test_temp_fixed_point(TemperatureUnit::Fahrenheit, 212.0, 373.15));
        
        // Water boils at 671.67 Rankine
        assert!(test_temp_fixed_point(TemperatureUnit::Rankine, 671.67, 373.15));
    }

    #[test]
    fn verify_kelvin_identity() {
        // 100 Kelvin must be 100 Kelvin
        assert!(test_temp_fixed_point(TemperatureUnit::Kelvin, 100.0, 100.0));
        // 0 Kelvin must be 0 Kelvin
        assert!(test_temp_fixed_point(TemperatureUnit::Kelvin, 0.0, 0.0));
    }
 
  
    
}