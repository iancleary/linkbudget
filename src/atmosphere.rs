//! Atmospheric attenuation coupled to ground-station sky brightness temperature.
//!
//! This model does not predict attenuation or total antenna/system noise.

use crate::ModelError;

/// Sky brightness temperature in kelvin from atmospheric attenuation in dB.
///
/// Implements [ITU-R P.618-14, section 3, equation 69](https://www.itu.int/dms_pubrec/itu-r/rec/p/R-REC-P.618-14-202308-I!!PDF-E.pdf):
/// `T_sky = T_m * (1 - 10^(-A/10)) + 2.7 * 10^(-A/10)`.
/// The 2.7 K background is attenuated by the same atmosphere that emits noise.
/// The absorbed fraction uses `exp_m1` to retain precision near zero attenuation.
///
/// `atmospheric_attenuation_db` excludes scintillation and all nonabsorptive
/// losses, including free-space spreading and pointing loss. The mean radiating
/// temperature is an atmospheric temperature, not receiver system temperature.
/// The recommendation suggests 275 K when local radiating-temperature data are
/// unavailable; this function requires an explicit value rather than choosing it.
///
/// The result is brightness, not a beam-integrated antenna temperature. Account
/// separately for antenna pattern, spillover, feed loss, other sky sources, and
/// receiver-added noise before interpreting it as total system temperature.
/// No receiver or link-budget state is changed automatically.
///
/// # Errors
///
/// Returns [`ModelError::InvalidInput`] unless attenuation is finite and
/// nonnegative and mean radiating temperature is finite and strictly positive.
pub fn sky_noise_temperature_k(
    atmospheric_attenuation_db: f64,
    mean_radiating_temperature_k: f64,
) -> Result<f64, ModelError> {
    if !atmospheric_attenuation_db.is_finite() || atmospheric_attenuation_db < 0.0 {
        return Err(ModelError::InvalidInput("atmospheric_attenuation_db"));
    }
    if !mean_radiating_temperature_k.is_finite() || mean_radiating_temperature_k <= 0.0 {
        return Err(ModelError::InvalidInput("mean_radiating_temperature_k"));
    }

    let log_transmission = -atmospheric_attenuation_db * (std::f64::consts::LN_10 / 10.0);
    let absorbed_fraction = -log_transmission.exp_m1();
    Ok(mean_radiating_temperature_k * absorbed_fraction + 2.7 * log_transmission.exp())
}
