/// Doppler shift in Hz for a given transmitted frequency and radial velocity.
///
/// Δf = f · v_r / c (non-relativistic approximation, valid for v << c).
///
/// Convention: positive `radial_velocity_m_s` = approaching (upshift),
/// negative = receding (downshift).
///
/// # Examples
///
/// ```
/// use linkbudget::doppler::doppler_shift_hz;
///
/// // LEO at Ku-band: ~304 kHz shift at max radial velocity
/// let shift = doppler_shift_hz(12.0e9, 7600.0);
/// assert!((shift - 304_210.0).abs() < 100.0);
///
/// // Receding gives negative shift
/// assert!(doppler_shift_hz(12.0e9, -7600.0) < 0.0);
/// ```
pub fn doppler_shift_hz(frequency_hz: f64, radial_velocity_m_s: f64) -> f64 {
    frequency_hz * radial_velocity_m_s / 299_792_458.0
}

/// Received frequency accounting for Doppler: f_rx = f_tx + Δf.
///
/// # Examples
///
/// ```
/// use linkbudget::doppler::doppler_received_frequency;
///
/// let f_rx = doppler_received_frequency(12.0e9, 7600.0);
/// assert!(f_rx > 12.0e9); // approaching → higher frequency
/// ```
pub fn doppler_received_frequency(frequency_hz: f64, radial_velocity_m_s: f64) -> f64 {
    frequency_hz + doppler_shift_hz(frequency_hz, radial_velocity_m_s)
}

/// Maximum radial velocity component for a circular orbit at a given elevation angle.
///
/// At horizon (0° elevation), the radial component equals the full orbital speed.
/// At zenith (90° elevation), the satellite moves perpendicular to the line of
/// sight, so radial velocity ≈ 0.
///
/// v_radial = v_orbital · cos(elevation)
pub fn max_radial_velocity_circular(orbital_speed_m_s: f64, elevation_angle_degrees: f64) -> f64 {
    let elevation_rad = elevation_angle_degrees * std::f64::consts::PI / 180.0;
    orbital_speed_m_s * elevation_rad.cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doppler_shift_leo_ku_band() {
        // LEO satellite at ~550 km, Ku-band 12 GHz, approaching at 7600 m/s
        let freq = 12.0e9;
        let velocity = 7600.0;
        let shift = doppler_shift_hz(freq, velocity);

        // Expected: 12e9 * 7600 / 299792458 ≈ 304,210 Hz
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
