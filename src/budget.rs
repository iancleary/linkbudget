use crate::fspl::SlantRange;
use crate::phy::PhyRate;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;

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

    pub fn pin_at_receiver(&self) -> f64 {
        let free_space_path_loss = self.fspl();

        // Assumes receiver input power is spread across the bandwidth

        // pin_at_receiver =
        self.transmitter.output_power + self.transmitter.gain - free_space_path_loss - self.rain_fade + self.receiver.gain
    }
    pub fn snr(&self) -> f64 {
        // returns value in dB
        self.receiver.calculate_snr(self.pin_at_receiver())
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
}

