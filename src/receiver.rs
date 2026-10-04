//! Receiver model for link budget calculations.

use rfconversions::noise::{noise_power_from_bandwidth, noise_temperature_from_noise_figure};
use rfconversions::power::watts_to_dbm;

/// Input-referred noise specification for a receiver.
///
/// Use total system temperature when receiver-added noise is already included.
/// Otherwise, specify the source temperature and the receiver noise figure.
/// All temperatures refer to the same input plane as the received signal power.
/// The model assumes flat, uncorrelated thermal noise over the noise bandwidth.
/// Inputs must be finite and give a positive total temperature; values are not
/// validated at runtime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReceiverNoise {
    /// Total source and receiver-added noise, referred to the receiver input.
    SystemTemperature {
        /// Total system noise temperature in kelvin, greater than zero.
        temperature_k: f64,
    },
    /// Source noise plus receiver-added noise specified at T₀ = 290 K.
    ///
    /// The total is `T_source + 290 * (10^(NF/10) - 1)` kelvin.
    /// Multiplication of source temperature by noise factor is valid only
    /// when the source temperature is 290 K or the noise factor is 1.
    SourceAndNoiseFigure {
        /// Source noise temperature in kelvin, greater than or equal to zero.
        source_temperature_k: f64,
        /// Receiver noise figure in dB, greater than or equal to zero.
        noise_figure_db: f64,
    },
}

/// A radio receiver with antenna gain, input-referred noise, and bandwidth.
///
/// Gain, signal power, and noise temperature must use a common reference plane.
/// For example, include any feed loss consistently in both gain and noise.
#[doc(alias = "G/T")]
pub struct Receiver {
    /// Antenna gain in dBi.
    pub gain: f64,
    /// Total system temperature or source temperature plus receiver noise figure.
    pub noise: ReceiverNoise,
    /// Equivalent noise bandwidth in Hz; must be finite and greater than zero.
    pub bandwidth: f64,
}

impl Receiver {
    /// Total input-referred system noise temperature in kelvin.
    ///
    /// With noise factor `F = 10^(NF/10)`, receiver-added temperature is
    /// `T_e = 290 * (F - 1)`. Independent source and receiver noise powers add,
    /// so `T_sys = T_source + T_e`.
    ///
    /// ```
    /// use linkbudget::{Receiver, ReceiverNoise};
    /// let receiver = Receiver {
    ///     gain: 40.0,
    ///     noise: ReceiverNoise::SourceAndNoiseFigure {
    ///         source_temperature_k: 50.0,
    ///         noise_figure_db: 10.0 * 2.0_f64.log10(),
    ///     },
    ///     bandwidth: 20e6,
    /// };
    /// assert!((receiver.system_noise_temperature_k() - 340.0).abs() < 1e-10);
    /// ```
    #[must_use]
    pub fn system_noise_temperature_k(&self) -> f64 {
        match self.noise {
            ReceiverNoise::SystemTemperature { temperature_k } => temperature_k,
            ReceiverNoise::SourceAndNoiseFigure {
                source_temperature_k,
                noise_figure_db,
            } => source_temperature_k + noise_temperature_from_noise_figure(noise_figure_db),
        }
    }

    /// Total input-referred thermal noise floor in dBm.
    ///
    /// This is an alias for [`Self::calculate_noise_power`]. It includes
    /// receiver-added noise. A source-only floor cannot be recovered from a
    /// total system temperature.
    #[must_use]
    pub fn calculate_noise_floor(&self) -> f64 {
        self.calculate_noise_power()
    }

    /// Total input-referred noise power in dBm: `k * T_sys * B_noise`.
    #[must_use]
    pub fn calculate_noise_power(&self) -> f64 {
        watts_to_dbm(noise_power_from_bandwidth(
            self.system_noise_temperature_k(),
            self.bandwidth,
        ))
    }

    /// G/T (gain-to-noise-temperature ratio) in dB/K.
    #[doc(alias = "G/T")]
    #[must_use]
    pub fn g_over_t_db(&self) -> f64 {
        self.gain - 10.0 * self.system_noise_temperature_k().log10()
    }

    /// Signal-to-noise ratio in dB for a given input power (dBm).
    #[doc(alias = "SNR")]
    #[must_use]
    pub fn calculate_snr(&self, input_power: f64) -> f64 {
        input_power - self.calculate_noise_power()
    }
}

#[cfg(test)]
mod tests {
    use crate::receiver::{Receiver, ReceiverNoise};

    #[test]
    fn total_temperature_sets_both_noise_power_and_gt() {
        let receiver = Receiver {
            gain: 40.0,
            noise: ReceiverNoise::SystemTemperature {
                temperature_k: 340.0,
            },
            bandwidth: 100.0e6,
        };

        // Independent SI Boltzmann constant, with watts converted to dBm.
        let expected_dbm = 10.0 * (1.380_649e-23_f64 * 340.0 * 100e6).log10() + 30.0;
        assert!((receiver.calculate_noise_power() - expected_dbm).abs() < 1e-10);
        assert_eq!(
            receiver.calculate_noise_floor(),
            receiver.calculate_noise_power()
        );
        assert!((receiver.g_over_t_db() - (40.0 - 10.0 * 340.0_f64.log10())).abs() < 1e-10);
    }

    #[test]
    fn standard_source_recovers_noise_factor_scaling() {
        let receiver = Receiver {
            gain: 10.0, // not used
            noise: ReceiverNoise::SourceAndNoiseFigure {
                source_temperature_k: 290.0,
                noise_figure_db: 3.0,
            },
            bandwidth: 100.0e6,
        };

        let expected_dbm = 10.0 * (1.380_649e-23_f64 * 290.0 * 100e6).log10() + 30.0 + 3.0;
        assert!((receiver.calculate_noise_power() - expected_dbm).abs() < 1e-10);
        assert!((receiver.calculate_snr(-70.0) - 20.9772).abs() < 0.01);
    }

    #[test]
    fn noiseless_receiver_preserves_source_temperature() {
        let receiver = Receiver {
            gain: 40.0,
            noise: ReceiverNoise::SourceAndNoiseFigure {
                source_temperature_k: 50.0,
                noise_figure_db: 0.0,
            },
            bandwidth: 100.0e6,
        };

        assert_eq!(receiver.system_noise_temperature_k(), 50.0);
    }

    #[test]
    fn zero_source_still_has_receiver_added_noise() {
        let receiver = Receiver {
            gain: 40.0,
            noise: ReceiverNoise::SourceAndNoiseFigure {
                source_temperature_k: 0.0,
                noise_figure_db: 10.0 * 2.0_f64.log10(),
            },
            bandwidth: 100.0e6,
        };

        assert!((receiver.system_noise_temperature_k() - 290.0).abs() < 1e-10);
    }
}
