use crate::fspl::SlantRange;
use crate::phy::PhyRate;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;
use crate::utils::print::{print_header, print_row, print_separator, print_title};

// elevation_angle and altitude could be moved to a struct
// also could come from the position of the transmitter and receiver
// and the radius of the body (lat/long/alt of the transmitter and receiver)

pub struct LinkBudget {
    pub name: &'static str,
    pub frequency: f64,
    pub bandwidth: f64,
    pub transmitter: Transmitter,
    pub receiver: Receiver,
    pub elevation_angle_degrees: f64,
    pub altitude: f64,
    pub rain_fade: f64,
}

impl LinkBudget {
    pub fn fspl(&self) -> f64 {
        let slant_range: f64 = SlantRange {
            elevation_angle_degrees: self.elevation_angle_degrees,
            altitude: self.altitude,
        }
        .calculate();

        crate::fspl::calculate_free_space_path_loss(self.frequency, slant_range)
    }

    pub fn pin_at_reciever(&self) -> f64 {
        let free_space_path_loss = self.fspl();

        // Assumes receiver input power is spread across the bandwidth

        let pin_at_receiver = self.transmitter.output_power + self.transmitter.gain
            - free_space_path_loss
            - self.rain_fade
            + self.receiver.gain;

        pin_at_receiver
    }
    pub fn snr(&self) -> f64 {
        // returns value in dB
        self.receiver.calculate_snr(self.pin_at_reciever())
    }

    pub fn snr_linear(&self) -> f64 {
        // returns linear value (not dB)
        10.0_f64.powf(self.snr() / 10.0)
    }

    pub fn phy_rate(&self) -> PhyRate {
        PhyRate {
            bandwidth: self.bandwidth,
            snr: self.snr_linear(),
        }
    }

    pub fn print(&self) {
        print_title(&self.name);
        print_row("Summary", "Value", "Unit");
        print_header();
        print_row("Frequency", &self.frequency.to_string(), "Hz");
        print_row("Bandwidth", &self.bandwidth.to_string(), "Hz");
        print_row(
            "Transmitter Pout",
            &self.transmitter.output_power.to_string(),
            "dBm",
        );
        print_row("Transmitter Gain", &self.transmitter.gain.to_string(), "dB");
        print_row("Receiver Gain", &self.receiver.gain.to_string(), "dB");
        print_row("Receiver NF", &self.receiver.noise_figure.to_string(), "dB");
        print_row(
            "Receiver Temperature",
            &self.receiver.temperature.to_string(),
            "K",
        );
        print_row(
            "Elevation Angle",
            &self.elevation_angle_degrees.to_string(),
            "deg",
        );
        print_row("Altitude", &self.altitude.to_string(), "km");
        print_row("Rain Fade", &self.rain_fade.to_string(), "dB");
        print_separator();
        print_row("Calculations", "Value", "Unit");
        print_header();
        print_row("Free Space Path Loss", &self.fspl().to_string(), "dB");
        print_row(
            "Receiver Input Power",
            &self.pin_at_reciever().to_string(),
            "dBm",
        );
        print_row("SNR", &self.snr().to_string(), "dB");
        print_separator();

        self.phy_rate().print();

        print_separator();
    }
}
