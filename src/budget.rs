use crate::ber;
use crate::coding::CodedModulation;
use crate::energy;
use crate::modulation::Modulation;
use crate::path_loss::PathLoss;
use crate::phy::PhyRate;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;

// elevation_angle and altitude could be moved to a struct
// also could come from the position of the transmitter and receiver
// and the radius of the body (lat/long/alt of the transmitter and receiver)

pub struct LinkBudget {
    pub name: &'static str,
    pub bandwidth: f64,           // in Hz
    pub transmitter: Transmitter, // you should include any pointing loss, etc. here
    pub receiver: Receiver,       // you should include any pointing loss, etc. here
    pub path_loss: PathLoss,      // you may calculate this yourself for various situations
    // the frequency dependence is really an antenna effect (for free space)
    pub frequency_dependent_loss: Option<f64>, // optional fade margin, such as rain fade, obstacles, etc.
                                               // this is the part that generally has a frequency dependence
                                               // rain fade, obstacle penetration, atmospheric absorption, etc.
}

impl LinkBudget {
    pub fn path_loss(&self) -> f64 {
        let path_loss_in_db: f64 = self.path_loss.calculate();

        let mut total_path_loss_in_db: f64 = path_loss_in_db;
        if let Some(frequency_dependent_loss) = self.frequency_dependent_loss {
            total_path_loss_in_db += frequency_dependent_loss;
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

    // ----- Modulation-aware methods (Phase 5 of issue #12) -----

    /// C/No in dB·Hz from the link budget SNR and noise bandwidth.
    ///
    /// C/No = SNR(dB) + 10·log10(noise_bandwidth)
    ///
    /// Uses the receiver bandwidth as the noise bandwidth.
    pub fn c_over_no(&self) -> f64 {
        energy::snr_to_c_over_no(self.snr(), self.receiver.bandwidth)
    }

    /// Eb/No in dB for a given modulation scheme (uncoded).
    ///
    /// Converts the link SNR to Eb/No using the modulation's spectral efficiency:
    /// ```text
    /// Eb/No = SNR - 10·log10(spectral_efficiency)
    /// ```
    ///
    /// For coded systems, use [`Self::eb_no_coded_db`] instead.
    pub fn eb_no_db(&self, modulation: &Modulation) -> f64 {
        let eta = modulation.bits_per_symbol(); // uncoded spectral efficiency
        self.snr() - 10.0 * eta.log10()
    }

    /// Eb/No in dB for a coded modulation scheme.
    ///
    /// Accounts for the code rate's effect on spectral efficiency:
    /// ```text
    /// Eb/No = SNR - 10·log10(k * R)
    /// ```
    pub fn eb_no_coded_db(&self, coded_mod: &CodedModulation) -> f64 {
        let eta = coded_mod.spectral_efficiency();
        self.snr() - 10.0 * eta.log10()
    }

    /// BER for a given modulation at the link's Eb/No (uncoded).
    pub fn ber(&self, modulation: &Modulation) -> f64 {
        let eb_no_db = self.eb_no_db(modulation);
        ber::ber_from_db(eb_no_db, modulation)
    }

    /// BER for a coded modulation scheme at the link's Eb/No.
    pub fn ber_coded(&self, coded_mod: &CodedModulation) -> f64 {
        let eb_no_db = self.eb_no_coded_db(coded_mod);
        coded_mod.ber_from_db(eb_no_db)
    }

    /// Link margin in dB for a given modulation and target BER (uncoded).
    ///
    /// Positive margin = link closes with headroom.
    /// Negative margin = link does not close.
    pub fn link_margin_db(&self, modulation: &Modulation, target_ber: f64) -> Option<f64> {
        let actual = self.eb_no_db(modulation);
        let required = ber::required_eb_no_db(target_ber, modulation)?;
        Some(actual - required)
    }

    /// Link margin in dB for a coded modulation and target BER.
    pub fn link_margin_coded_db(
        &self,
        coded_mod: &CodedModulation,
        target_ber: f64,
    ) -> Option<f64> {
        let actual = self.eb_no_coded_db(coded_mod);
        coded_mod.link_margin_db(actual, target_ber)
    }

    /// Achievable throughput in bits/s for a coded modulation scheme.
    pub fn throughput_bps(&self, coded_mod: &CodedModulation) -> f64 {
        coded_mod.throughput_bps(self.bandwidth)
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    use crate::coding::{dvbs2_qpsk_r34, FecCode};

    fn sample_budget() -> LinkBudget {
        LinkBudget {
            name: "Test Ka-band LEO",
            bandwidth: 36e6,
            transmitter: Transmitter {
                output_power: 10.0,
                gain: 35.0,
                bandwidth: 36e6,
            },
            receiver: Receiver {
                gain: 40.0,
                temperature: 290.0,
                noise_figure: 2.0,
                bandwidth: 36e6,
            },
            path_loss: PathLoss {
                frequency: 20e9,
                distance: 550e3,
            },
            frequency_dependent_loss: Some(3.0),
        }
    }

    #[test]
    fn c_over_no_positive() {
        let b = sample_budget();
        let c_no = b.c_over_no();
        // Should be SNR + 10*log10(36e6) ≈ SNR + 75.56
        let expected = b.snr() + 10.0 * 36e6_f64.log10();
        assert!((c_no - expected).abs() < 0.01);
    }

    #[test]
    fn eb_no_less_than_snr_for_qpsk() {
        let b = sample_budget();
        let eb_no = b.eb_no_db(&Modulation::Qpsk);
        // QPSK: k=2, so Eb/No = SNR - 10*log10(2) ≈ SNR - 3.01
        let expected = b.snr() - 10.0 * 2.0_f64.log10();
        assert!((eb_no - expected).abs() < 0.01);
    }

    #[test]
    fn ber_returns_valid_value() {
        let b = sample_budget();
        let ber_val = b.ber(&Modulation::Qpsk);
        assert!(ber_val >= 0.0 && ber_val <= 0.5);
    }

    #[test]
    fn link_margin_qpsk() {
        let b = sample_budget();
        let margin = b.link_margin_db(&Modulation::Qpsk, 1e-5);
        assert!(margin.is_some());
        // Just check it returns a reasonable number
        let m = margin.unwrap();
        assert!(m > -50.0 && m < 100.0);
    }

    #[test]
    fn coded_margin_better_than_uncoded() {
        let b = sample_budget();
        let uncoded_margin = b.link_margin_db(&Modulation::Qpsk, 1e-5).unwrap();

        let coded = dvbs2_qpsk_r34();
        let coded_margin = b.link_margin_coded_db(&coded, 1e-5).unwrap();

        // Coded margin should be better (more headroom) due to coding gain
        assert!(coded_margin > uncoded_margin,
            "Coded margin ({:.1} dB) should exceed uncoded ({:.1} dB)",
            coded_margin, uncoded_margin);
    }

    #[test]
    fn throughput_matches_spectral_efficiency() {
        let b = sample_budget();
        let cm = dvbs2_qpsk_r34();
        let tp = b.throughput_bps(&cm);
        // QPSK R=3/4: η = 2 × 0.75 = 1.5, throughput = 36e6 × 1.5 = 54 Mbps
        assert!((tp - 54e6).abs() < 1.0);
    }

    #[test]
    fn coded_ber_lower_than_uncoded() {
        let b = sample_budget();
        let ber_uncoded = b.ber(&Modulation::Qpsk);

        let coded = dvbs2_qpsk_r34();
        let ber_coded = b.ber_coded(&coded);

        assert!(ber_coded < ber_uncoded,
            "Coded BER ({:.2e}) should be lower than uncoded ({:.2e})",
            ber_coded, ber_uncoded);
    }
}
