use std::f64::consts::PI;

/// Power Flux Density in dBW/m²
pub fn power_flux_density_dbw_per_m2(eirp_dbw: f64, distance_m: f64) -> f64 {
    eirp_dbw - 10.0 * (4.0 * PI * distance_m * distance_m).log10()
}

/// Power Flux Density in dBW/m²/MHz (for regulatory, spread over bandwidth)
pub fn pfd_per_mhz(eirp_dbw: f64, distance_m: f64, bandwidth_mhz: f64) -> f64 {
    power_flux_density_dbw_per_m2(eirp_dbw, distance_m) - 10.0 * bandwidth_mhz.log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pfd_geostationary() {
        // GEO satellite: EIRP = 50 dBW, distance ≈ 35,786 km
        let eirp_dbw = 50.0;
        let distance_m = 35_786_000.0;

        let pfd = power_flux_density_dbw_per_m2(eirp_dbw, distance_m);

        // PFD = 50 - 10*log10(4*pi*(35786000)^2)
        let expected = eirp_dbw - 10.0 * (4.0 * std::f64::consts::PI * distance_m * distance_m).log10();
        assert!((pfd - expected).abs() < 1e-10);
    }

    #[test]
    fn pfd_per_mhz_test() {
        let eirp_dbw = 50.0;
        let distance_m = 35_786_000.0;
        let bandwidth_mhz = 36.0;

        let pfd_mhz = pfd_per_mhz(eirp_dbw, distance_m, bandwidth_mhz);
        let pfd_total = power_flux_density_dbw_per_m2(eirp_dbw, distance_m);

        let expected = pfd_total - 10.0 * bandwidth_mhz.log10();
        assert!((pfd_mhz - expected).abs() < 1e-10);
    }
}
