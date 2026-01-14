//! Carbon emission estimation
//!
//! Converts energy consumption to CO2 equivalent emissions based on
//! grid carbon intensity.

/// Carbon intensity in grams CO2 per kWh for various regions
/// Source: https://www.carbonfootprint.com/international_electricity_factors.html
/// These are approximate averages and vary by time of day and season.
#[derive(Debug, Clone, Copy)]
pub struct CarbonIntensity {
    /// Region name
    pub name: &'static str,
    /// grams CO2 per kWh
    pub gco2_per_kwh: f64,
}

// Common grid carbon intensities (2023 averages)
pub const INTENSITIES: &[CarbonIntensity] = &[
    CarbonIntensity { name: "uk", gco2_per_kwh: 207.0 },
    CarbonIntensity { name: "us-avg", gco2_per_kwh: 390.0 },
    CarbonIntensity { name: "us-california", gco2_per_kwh: 230.0 },
    CarbonIntensity { name: "us-texas", gco2_per_kwh: 410.0 },
    CarbonIntensity { name: "germany", gco2_per_kwh: 350.0 },
    CarbonIntensity { name: "france", gco2_per_kwh: 56.0 },
    CarbonIntensity { name: "sweden", gco2_per_kwh: 45.0 },
    CarbonIntensity { name: "poland", gco2_per_kwh: 635.0 },
    CarbonIntensity { name: "china", gco2_per_kwh: 555.0 },
    CarbonIntensity { name: "india", gco2_per_kwh: 710.0 },
    CarbonIntensity { name: "australia", gco2_per_kwh: 510.0 },
    CarbonIntensity { name: "canada", gco2_per_kwh: 120.0 },
    CarbonIntensity { name: "japan", gco2_per_kwh: 470.0 },
    CarbonIntensity { name: "brazil", gco2_per_kwh: 96.0 },
    // World average as fallback
    CarbonIntensity { name: "world", gco2_per_kwh: 436.0 },
];


/// Look up carbon intensity by region name
pub fn get_intensity(region: &str) -> Option<CarbonIntensity> {
    let region_lower = region.to_lowercase();
    INTENSITIES.iter().find(|i| i.name == region_lower).copied()
}

/// List all available regions
pub fn available_regions() -> Vec<&'static str> {
    INTENSITIES.iter().map(|i| i.name).collect()
}

/// Calculate CO2 emissions in grams from energy in microjoules
pub fn calculate_co2_grams(energy_uj: u64, intensity: &CarbonIntensity) -> f64 {
    // Convert µJ to kWh: 1 kWh = 3.6e12 µJ
    let energy_kwh = energy_uj as f64 / 3.6e12;
    energy_kwh * intensity.gco2_per_kwh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_intensity() {
        assert!(get_intensity("uk").is_some());
        assert!(get_intensity("UK").is_some()); // case insensitive
        assert!(get_intensity("invalid").is_none());
    }

    #[test]
    fn test_co2_calculation() {
        // 1 kWh = 3.6e12 µJ
        let one_kwh_uj = 3_600_000_000_000u64;
        let uk = get_intensity("uk").unwrap();
        let co2 = calculate_co2_grams(one_kwh_uj, &uk);
        // Should be approximately 207g for UK
        assert!((co2 - 207.0).abs() < 1.0);
    }
}
