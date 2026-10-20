//! Offline annual rain attenuation from caller-supplied climate and path data.
//!
//! Implements ITU-R P.838-3 (03/2005), equations 1–5 and Tables 1–4, and
//! P.618-14 (08/2023), section 2.2.1.1, steps 1–10. No climate maps are bundled
//! or fetched. These are rain-only annual quantiles, not total weather loss,
//! outage durations, satellite-pass availability, or a temporal rain process.

use crate::ModelError;

/// Specific rain attenuation in dB/km using ITU-R P.838-3 equations 1–5.
///
/// Frequency is in Hz (supported range: 1–1000 GHz), rain rate in mm/h
/// (nonnegative), and elevation in degrees (0–90 inclusive). Polarization tilt
/// is relative to horizontal: 0° horizontal, 90° vertical, 45° circular. Any
/// finite tilt is accepted and reduced modulo 180°. Zero rain returns zero.
///
/// # Errors
///
/// Returns [`ModelError::InvalidInput`] for nonfinite or invalid physical inputs,
/// [`ModelError::UnsupportedRegime`] for frequencies outside the supported band,
/// and [`ModelError::NumericalFailure`] if evaluation overflows.
pub fn specific_attenuation_db_per_km(
    frequency_hz: f64,
    rain_rate_mm_h: f64,
    elevation_deg: f64,
    polarization_tilt_deg: f64,
) -> Result<f64, ModelError> {
    validate_frequency(frequency_hz, 1000e9)?;
    validate_nonnegative(rain_rate_mm_h, "rain_rate_mm_h")?;
    validate_range(elevation_deg, 0.0, 90.0, "elevation_deg")?;
    if !polarization_tilt_deg.is_finite() {
        return Err(ModelError::InvalidInput("polarization_tilt_deg"));
    }
    if rain_rate_mm_h == 0.0 {
        return Ok(0.0);
    }

    let log_f = (frequency_hz / 1e9).log10();
    // Factual curve-fit constants from P.838-3 Tables 1–4: (a_j, b_j, c_j).
    let kh = 10.0_f64.powf(curve(
        log_f,
        &[
            (-5.33980, -0.10008, 1.13098),
            (-0.35351, 1.26970, 0.45400),
            (-0.23789, 0.86036, 0.15354),
            (-0.94158, 0.64552, 0.16817),
        ],
        -0.18961,
        0.71147,
    ));
    let kv = 10.0_f64.powf(curve(
        log_f,
        &[
            (-3.80595, 0.56934, 0.81061),
            (-3.44965, -0.22911, 0.51059),
            (-0.39902, 0.73042, 0.11899),
            (0.50167, 1.07319, 0.27195),
        ],
        -0.16398,
        0.63297,
    ));
    let ah = curve(
        log_f,
        &[
            (-0.14318, 1.82442, -0.55187),
            (0.29591, 0.77564, 0.19822),
            (0.32177, 0.63773, 0.13164),
            (-5.37610, -0.96230, 1.47828),
            (16.1721, -3.29980, 3.43990),
        ],
        0.67849,
        -1.95537,
    );
    let av = curve(
        log_f,
        &[
            (-0.07771, 2.33840, -0.76284),
            (0.56727, 0.95545, 0.54039),
            (-0.20238, 1.14520, 0.26809),
            (-48.2991, 0.791669, 0.116226),
            (48.5833, 0.791459, 0.116479),
        ],
        -0.053739,
        0.83433,
    );
    let geometry = elevation_deg.to_radians().cos().powi(2)
        * (2.0 * (polarization_tilt_deg % 180.0)).to_radians().cos();
    let k = (kh + kv + (kh - kv) * geometry) / 2.0;
    let alpha = (kh * ah + kv * av + (kh * ah - kv * av) * geometry) / (2.0 * k);
    let rain_power = rain_rate_mm_h.powf(alpha);
    if !rain_power.is_finite() {
        return Err(ModelError::NumericalFailure("specific rain attenuation"));
    }
    let attenuation = k * rain_power;
    if attenuation.is_finite() {
        Ok(attenuation)
    } else {
        Err(ModelError::NumericalFailure("specific rain attenuation"))
    }
}

/// Earth-space rain path and local annual climate inputs for ITU-R P.618-14.
///
/// Heights share a mean-sea-level datum. Obtain rain height from a justified
/// local source or P.839, and one-minute annual R0.01 from local statistics or
/// P.837; this type performs no geographic lookup. All fields are validated
/// on each call to [`Self::attenuation_exceeded_db`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RainPath {
    /// Frequency in Hz, supported from 1 to 55 GHz inclusive.
    pub frequency_hz: f64,
    /// Earth-station elevation angle in degrees, greater than 0 and at most 90.
    pub elevation_deg: f64,
    /// Earth-station latitude in degrees, from -90 to 90 inclusive.
    pub latitude_deg: f64,
    /// Finite station height above mean sea level in meters; may be negative.
    pub station_height_m: f64,
    /// Nonnegative rain height above mean sea level in meters.
    pub rain_height_m: f64,
    /// Nonnegative point rain rate exceeded for 0.01% of an average year, mm/h.
    /// This statistic uses one-minute integration, not an instantaneous sample.
    pub rain_rate_001_mm_h: f64,
    /// Tilt relative to horizontal in degrees: H=0, V=90, circular=45.
    pub polarization_tilt_deg: f64,
}

impl RainPath {
    /// Rain attenuation in dB exceeded for the named percentage of an average year.
    ///
    /// `exceedance_percent` is a percentage, not a probability fraction or an
    /// availability: 99.9% rain-only availability corresponds to **0.1** here.
    /// Supported percentages are 0.001–5 inclusive. Implements P.618-14 section
    /// 2.2.1.1 steps 1–10 with effective Earth radius 8500 km. Zero rainfall or a
    /// station at/above rain height gives zero after validating all inputs.
    ///
    /// The result is additional signal loss, not an automatic receiver-noise
    /// update. Do not feed scintillation or nonabsorptive losses into the
    /// atmospheric sky-noise helper; a rain quantile alone does not specify
    /// atmospheric emission or the joint distribution of weather effects.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidInput`] for nonfinite or out-of-domain inputs,
    /// [`ModelError::UnsupportedRegime`] outside 1–55 GHz or 0.001–5 percent,
    /// and [`ModelError::NumericalFailure`] for nonfinite evaluation or a positive
    /// intermediate that underflows to zero. No result is NaN or infinity.
    pub fn attenuation_exceeded_db(&self, exceedance_percent: f64) -> Result<f64, ModelError> {
        validate_frequency(self.frequency_hz, 55e9)?;
        validate_range(self.elevation_deg, 0.0, 90.0, "elevation_deg")?;
        if self.elevation_deg == 0.0 {
            return Err(ModelError::InvalidInput("elevation_deg"));
        }
        validate_range(self.latitude_deg, -90.0, 90.0, "latitude_deg")?;
        if !self.station_height_m.is_finite() {
            return Err(ModelError::InvalidInput("station_height_m"));
        }
        validate_nonnegative(self.rain_height_m, "rain_height_m")?;
        validate_nonnegative(self.rain_rate_001_mm_h, "rain_rate_001_mm_h")?;
        if !self.polarization_tilt_deg.is_finite() {
            return Err(ModelError::InvalidInput("polarization_tilt_deg"));
        }
        if !exceedance_percent.is_finite() || exceedance_percent <= 0.0 {
            return Err(ModelError::InvalidInput("exceedance_percent"));
        }
        if !(0.001..=5.0).contains(&exceedance_percent) {
            return Err(ModelError::UnsupportedRegime(
                "exceedance percent must be 0.001–5",
            ));
        }
        if self.station_height_m >= self.rain_height_m || self.rain_rate_001_mm_h == 0.0 {
            return Ok(0.0);
        }

        let height_km = positive(self.rain_height_m / 1000.0 - self.station_height_m / 1000.0)?;
        let theta = self.elevation_deg.to_radians();
        let (sin_theta, cos_theta) = theta.sin_cos();
        let slant_km = positive(if self.elevation_deg >= 5.0 {
            height_km / sin_theta
        } else {
            2.0 * height_km / ((sin_theta.powi(2) + 2.0 * height_km / 8500.0).sqrt() + sin_theta)
        })?;
        let horizontal_km = slant_km * cos_theta;
        let gamma = positive(specific_attenuation_db_per_km(
            self.frequency_hz,
            self.rain_rate_001_mm_h,
            self.elevation_deg,
            self.polarization_tilt_deg,
        )?)?;
        let frequency_ghz = self.frequency_hz / 1e9;
        let horizontal_loss = positive(horizontal_km * gamma)?;
        let reduction = positive(
            1.0 / (1.0 + 0.78 * (horizontal_loss / frequency_ghz).sqrt()
                - 0.38 * (-(-2.0 * horizontal_km).exp_m1())),
        )?;
        let zeta = height_km.atan2(horizontal_km * reduction);
        // L_G / cos(theta) = L_s avoids division by a near-zero zenith cosine.
        let rain_path_km = positive(if zeta > theta {
            slant_km * reduction
        } else {
            height_km / sin_theta
        })?;
        let chi = (36.0 - self.latitude_deg.abs()).max(0.0);
        let rain_path_loss = positive(rain_path_km * gamma)?;
        let vertical = positive(
            1.0 / (1.0
                + sin_theta.sqrt()
                    * (31.0
                        * (-(-self.elevation_deg / (1.0 + chi)).exp_m1())
                        * rain_path_loss.sqrt()
                        / frequency_ghz.powi(2)
                        - 0.45)),
        )?;
        let attenuation_001 = positive(rain_path_loss * vertical)?;
        if exceedance_percent == 0.01 {
            return Ok(attenuation_001);
        }
        let beta = if exceedance_percent >= 1.0 || self.latitude_deg.abs() >= 36.0 {
            0.0
        } else if self.elevation_deg >= 25.0 {
            0.005 * chi
        } else {
            0.005 * chi + 1.8 - 4.25 * sin_theta
        };
        let exponent = -(0.655 + 0.033 * exceedance_percent.ln()
            - 0.045 * attenuation_001.ln()
            - beta * (1.0 - exceedance_percent) * sin_theta);
        positive(attenuation_001 * (exceedance_percent / 0.01).powf(exponent))
    }
}

fn curve(log_f: f64, terms: &[(f64, f64, f64)], slope: f64, intercept: f64) -> f64 {
    terms
        .iter()
        .map(|&(a, b, c)| a * (-((log_f - b) / c).powi(2)).exp())
        .sum::<f64>()
        + slope * log_f
        + intercept
}

fn validate_frequency(frequency_hz: f64, upper_hz: f64) -> Result<(), ModelError> {
    if !frequency_hz.is_finite() || frequency_hz <= 0.0 {
        return Err(ModelError::InvalidInput("frequency_hz"));
    }
    if !(1e9..=upper_hz).contains(&frequency_hz) {
        return Err(ModelError::UnsupportedRegime(
            "frequency outside rain model band",
        ));
    }
    Ok(())
}

fn validate_nonnegative(value: f64, name: &'static str) -> Result<(), ModelError> {
    if !value.is_finite() || value < 0.0 {
        Err(ModelError::InvalidInput(name))
    } else {
        Ok(())
    }
}

fn validate_range(value: f64, min: f64, max: f64, name: &'static str) -> Result<(), ModelError> {
    if !value.is_finite() || !(min..=max).contains(&value) {
        Err(ModelError::InvalidInput(name))
    } else {
        Ok(())
    }
}

fn positive(value: f64) -> Result<f64, ModelError> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(ModelError::NumericalFailure("rain path evaluation"))
    }
}
