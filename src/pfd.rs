//! Power Flux Density (PFD) calculations.

use std::f64::consts::PI;

/// Power Flux Density in dBW/m².
#[doc(alias = "EIRP")]
#[must_use]
pub fn power_flux_density_dbw_per_m2(eirp_dbw: f64, distance_m: f64) -> f64 {
    eirp_dbw - 10.0 * (4.0 * PI * distance_m * distance_m).log10()
}

/// Power Flux Density in dBW/m²/MHz (for regulatory limits, spread over bandwidth).
#[must_use]
pub fn pfd_per_mhz(eirp_dbw: f64, distance_m: f64, bandwidth_mhz: f64) -> f64 {
    power_flux_density_dbw_per_m2(eirp_dbw, distance_m) - 10.0 * bandwidth_mhz.log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pfd_geostationary() {
        let eirp_dbw = 50.0;
        let distance_m = 35_786_000.0;

        let pfd = power_flux_density_dbw_per_m2(eirp_dbw, distance_m);

        let expected =
            eirp_dbw - 10.0 * (4.0 * std::f64::consts::PI * distance_m * distance_m).log10();
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

    #[test]
    fn pfd_leo_orbit() {
        // LEO satellite at 550 km altitude, 40 dBW EIRP
        let eirp_dbw = 40.0;
        let distance_m = 550_000.0;
        let pfd = power_flux_density_dbw_per_m2(eirp_dbw, distance_m);
        // PFD = EIRP - 10*log10(4*pi*d^2)
        // 4*pi*(550e3)^2 ≈ 3.801e12 → 10*log10 ≈ 125.8 dB
        // PFD ≈ 40 - 125.8 = -85.8 dBW/m²
        assert!((pfd - (-85.8)).abs() < 0.1);
    }

    #[test]
    fn pfd_inverse_square_law() {
        // Doubling distance should reduce PFD by 6 dB
        let eirp = 50.0;
        let d1 = 1000.0;
        let d2 = 2000.0;
        let pfd1 = power_flux_density_dbw_per_m2(eirp, d1);
        let pfd2 = power_flux_density_dbw_per_m2(eirp, d2);
        assert!((pfd1 - pfd2 - 6.0206).abs() < 0.001);
    }

    #[test]
    fn pfd_unit_distance() {
        // At 1 m, PFD = EIRP - 10*log10(4*pi)
        let eirp = 0.0;
        let pfd = power_flux_density_dbw_per_m2(eirp, 1.0);
        let expected = -10.0 * (4.0 * PI).log10();
        assert!((pfd - expected).abs() < 1e-10);
    }

    #[test]
    fn pfd_per_mhz_narrowband() {
        // 1 MHz bandwidth → pfd_per_mhz equals total PFD
        let eirp = 30.0;
        let dist = 1_000_000.0;
        let pfd_total = power_flux_density_dbw_per_m2(eirp, dist);
        let pfd_1mhz = pfd_per_mhz(eirp, dist, 1.0);
        assert!((pfd_total - pfd_1mhz).abs() < 1e-10);
    }

    #[test]
    fn pfd_per_mhz_bandwidth_scaling() {
        // 10x bandwidth → 10 dB lower PFD/MHz
        let eirp = 50.0;
        let dist = 35_786_000.0;
        let pfd_10 = pfd_per_mhz(eirp, dist, 10.0);
        let pfd_100 = pfd_per_mhz(eirp, dist, 100.0);
        assert!((pfd_10 - pfd_100 - 10.0).abs() < 1e-10);
    }
}
