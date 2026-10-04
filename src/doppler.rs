//! Doppler shift calculations for satellite communications.
//!
//! Uses the first-order, one-way relation `Δf = f * v_closing / c`.
//! See `docs/physics-geometry-and-errors.md` for the range-rate sign convention
//! and the limits of the circular-orbit projection helper.

/// Doppler shift in Hz for a given transmitted frequency and radial velocity.
///
/// `radial_velocity_m_s`: positive = approaching, negative = receding.
/// This is minus the rate of change of separation, not total orbital speed.
/// Supply a positive carrier frequency in Hz and relative speeds much below c.
#[doc(alias = "Doppler")]
#[must_use]
pub fn doppler_shift_hz(frequency_hz: f64, radial_velocity_m_s: f64) -> f64 {
    frequency_hz * radial_velocity_m_s / 299_792_458.0
}

/// Received frequency accounting for Doppler shift.
#[doc(alias = "Doppler")]
#[must_use]
pub fn doppler_received_frequency(frequency_hz: f64, radial_velocity_m_s: f64) -> f64 {
    frequency_hz + doppler_shift_hz(frequency_hz, radial_velocity_m_s)
}

/// Approximate radial-speed magnitude using `orbital_speed * cos(elevation)`.
///
/// For nonnegative speed and elevations from 0° to 90°, this assumes motion
/// parallel to the station's local horizon and toward the satellite's azimuth.
/// It returns the supplied speed at the horizon and approximately zero at zenith.
/// It omits spherical geometry, ground-station motion, and the approach/recede
/// sign; it is not an exact maximum for a spherical circular orbit.
#[doc(alias = "Doppler")]
#[must_use]
pub fn max_radial_velocity_circular(orbital_speed_m_s: f64, elevation_angle_degrees: f64) -> f64 {
    let elevation_rad = elevation_angle_degrees * std::f64::consts::PI / 180.0;
    orbital_speed_m_s * elevation_rad.cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doppler_shift_leo_ku_band() {
        let freq = 12.0e9;
        let velocity = 7600.0;
        let shift = doppler_shift_hz(freq, velocity);
        assert!((shift - 304_210.0).abs() < 100.0);
    }

    #[test]
    fn doppler_zero_at_zenith() {
        let orbital_speed = 7600.0;
        let radial_v = max_radial_velocity_circular(orbital_speed, 90.0);
        assert!(radial_v.abs() < 1e-10);
    }

    #[test]
    fn doppler_max_at_horizon() {
        let orbital_speed = 7600.0;
        let radial_v = max_radial_velocity_circular(orbital_speed, 0.0);
        assert!((radial_v - 7600.0).abs() < 1e-10);
    }

    #[test]
    fn received_frequency_approaching() {
        let freq = 12.0e9;
        let velocity = 7600.0;
        let received = doppler_received_frequency(freq, velocity);
        assert!(received > freq);
    }

    #[test]
    fn received_frequency_receding() {
        let freq = 12.0e9;
        let velocity = -7600.0;
        let received = doppler_received_frequency(freq, velocity);
        assert!(received < freq);
    }
}
