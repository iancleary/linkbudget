use crate::fspl::FreeSpacePathLoss;
use crate::phy::PhyRate;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;

// elevation_angle and altitude could be moved to a struct
// also could come from the position of the transmitter and receiver
// and the radius of the body (lat/long/alt of the transmitter and receiver)

pub struct LinkBudget {
    pub name: &'static str,
    pub bandwidth: f64,              // in Hz
    pub transmitter: Transmitter,    // you should include any pointing loss, etc. here
    pub receiver: Receiver,          // you should include any pointing loss, etc. here
    pub fspl: FreeSpacePathLoss,     // you may calculate this yourself for various situations
    pub fade_margin_db: Option<f64>, // optional fade margin, such as rain fade, obstacles, etc.
}

impl LinkBudget {
    pub fn path_loss(&self) -> f64 {
        let fspl_in_db: f64 = self.fspl.calculate();

        let mut total_path_loss_in_db: f64 = fspl_in_db;
        if let Some(fade_margin_db) = self.fade_margin_db {
            total_path_loss_in_db += fade_margin_db;
        }

        total_path_loss_in_db
    }

    pub fn pin_at_receiver(&self) -> f64 {
        let path_loss_in_db = self.path_loss();

        // Assumes receiver input power is spread across the bandwidth
        // pin_at_receiver =
        self.transmitter.output_power + self.transmitter.gain - path_loss_in_db + self.receiver.gain
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
