use crate::utils::print::{print_row, print_separator};

pub struct Receiver {
    pub gain: f64,         // dB
    pub temperature: f64,  // K
    pub noise_figure: f64, // dB
    pub bandwidth: f64,    // Hz
}

impl Receiver {
    pub fn calculate_noise_floor(&self) -> f64 {
        let receiver_noise_floor_power =
            crate::conversions::noise::noise_power_from_bandwidth(self.temperature, self.bandwidth);

        crate::conversions::power::watts_to_dbm(receiver_noise_floor_power)
    }

    pub fn calculate_noise_power(&self) -> f64 {
        self.calculate_noise_floor() + self.noise_figure
    }

    pub fn calculate_snr(&self, input_power: f64) -> f64 {
        let receiver_noise_floor_dbm = self.calculate_noise_floor();

        let receiver_total_noise_power = receiver_noise_floor_dbm + self.noise_figure;

        // Assumes receiver input power is spread across the bandwidth
        // returns value in dB
        input_power - receiver_total_noise_power
    }

    pub fn print(&self, input_power: f64) {
        print_row("Gain", &self.gain.to_string(), "dB");
        print_row("Temperature", &self.temperature.to_string(), "K");
        print_row("Noise Figure", &self.noise_figure.to_string(), "dB");
        print_row("Bandwidth", &self.bandwidth.to_string(), "Hz");
        print_separator();
        print_row(
            "Noise Floor",
            &self.calculate_noise_floor().to_string(),
            "dBm",
        );
        print_row(
            "Noise Power",
            &self.calculate_noise_power().to_string(),
            "dBm",
        );
        print_row("SNR", &self.calculate_snr(input_power).to_string(), "dB");
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
