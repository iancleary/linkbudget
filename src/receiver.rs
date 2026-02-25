use rfconversions::noise::noise_power_from_bandwidth;
use rfconversions::power::watts_to_dbm;

/// A receiver in a link budget, characterized by antenna gain, system noise
/// temperature, noise figure, and bandwidth.
///
/// # Examples
///
/// ```
/// use linkbudget::receiver::Receiver;
///
/// let rx = Receiver {
///     gain: 40.0,          // 40 dBi dish
///     temperature: 290.0,  // 290 K ambient
///     noise_figure: 1.5,   // 1.5 dB LNA
///     bandwidth: 36.0e6,   // 36 MHz
/// };
///
/// // G/T is a key figure of merit for receive systems
/// let g_over_t = rx.g_over_t_db();
/// assert!(g_over_t > 15.0); // 40 - 24.6 ≈ 15.4 dB/K
/// ```
pub struct Receiver {
    /// Receive antenna gain in dBi
    pub gain: f64,
    /// System noise temperature in Kelvin
    pub temperature: f64,
    /// Receiver noise figure in dB
    pub noise_figure: f64,
    /// Receiver bandwidth in Hz
    pub bandwidth: f64,
}

impl Receiver {
    /// Thermal noise floor in dBm: 10·log₁₀(k·T·B) converted to dBm.
    ///
    /// This is the fundamental noise power set by temperature and bandwidth,
    /// before adding the receiver's noise figure contribution.
    pub fn calculate_noise_floor(&self) -> f64 {
        let receiver_noise_floor_power =
            noise_power_from_bandwidth(self.temperature, self.bandwidth);

        watts_to_dbm(receiver_noise_floor_power)
    }

    /// Total receiver noise power in dBm (noise floor + noise figure).
    pub fn calculate_noise_power(&self) -> f64 {
        self.calculate_noise_floor() + self.noise_figure
    }

    /// Receive system figure of merit G/T in dB/K.
    ///
    /// G/T = antenna gain (dBi) − 10·log₁₀(T_sys). Higher is better.
    /// Used in link budgets to characterize receive sensitivity independent
    /// of bandwidth.
    pub fn g_over_t_db(&self) -> f64 {
        self.gain - 10.0 * self.temperature.log10()
    }

    /// Signal-to-noise ratio in dB for a given input power (dBm).
    ///
    /// SNR = input_power − (noise_floor + noise_figure)
    pub fn calculate_snr(&self, input_power: f64) -> f64 {
        let receiver_noise_floor_dbm = self.calculate_noise_floor();

        let receiver_total_noise_power = receiver_noise_floor_dbm + self.noise_figure;

        // Assumes receiver input power is spread across the bandwidth
        // returns value in dB
        input_power - receiver_total_noise_power
    }
}

#[cfg(test)]
mod tests {
    use crate::receiver::Receiver;

    #[test]
    fn calculate_noise_floor() {
        let receiver = Receiver {
            gain: 10.0, // not used
            temperature: 290.0,
            noise_figure: 3.0, // not used
            bandwidth: 100.0e6,
        };

        let noise_floor: f64 = receiver.calculate_noise_floor();

        assert_eq!(-93.97722915699808, noise_floor);
    }

    #[test]
    fn calculate_noise_power() {
        let receiver = Receiver {
            gain: 10.0, // not used
            temperature: 290.0,
            noise_figure: 3.0,
            bandwidth: 100.0e6,
        };

        let noise_power: f64 = receiver.calculate_noise_power();

        // noise floor + noise figure
        assert_eq!(-90.97722915699808, noise_power);
    }

    #[test]
    fn g_over_t_db() {
        let receiver = Receiver {
            gain: 40.0,
            temperature: 290.0,
            noise_figure: 3.0,
            bandwidth: 100.0e6,
        };

        let g_over_t = receiver.g_over_t_db();

        // 40 - 10*log10(290) ≈ 40 - 24.6237 ≈ 15.3763
        let expected = 40.0 - 10.0 * 290.0_f64.log10();
        assert!((g_over_t - expected).abs() < 1e-10);
    }

    #[test]
    fn calculate_snr() {
        let receiver = Receiver {
            gain: 10.0, // not used
            temperature: 290.0,
            noise_figure: 3.0,
            bandwidth: 100.0e6,
        };

        let input_power: f64 = -70.0; // dBm

        // Assumes receiver input power is spread across the bandwidth
        // returns value in dB
        let snr: f64 = receiver.calculate_snr(input_power);

        assert_eq!(20.977229156998078, snr);
    }
}
