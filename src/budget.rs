//! End-to-end link budget calculation.
//!
//! `docs/physics-rf-link.md` derives power spreading, kT noise, G/T, and
//! channel capacity. `docs/physics-communications.md` derives bit energy and
//! BER. Keep the common receiver input plane and the distinct bandwidths in
//! those guides when composing the scalar calculations here.

use crate::ber;
use crate::coding::CodedModulation;
use crate::energy;
use crate::modulation::Modulation;
use crate::path_loss::PathLoss;
use crate::phy::PhyRate;
use crate::receiver::Receiver;
use crate::transmitter::Transmitter;

/// A complete link budget from transmitter through path to receiver.
#[doc(alias = "link margin")]
pub struct LinkBudget {
    /// Human-readable name for this link.
    pub name: &'static str,
    /// Channel bandwidth in Hz (treated as symbol rate for throughput and Eb/No).
    pub bandwidth: f64,
    /// Transmitter (include any pointing loss in the gain).
    pub transmitter: Transmitter,
    /// Receiver (include any pointing loss in the gain).
    pub receiver: Receiver,
    /// Free-space path loss model (you may calculate distance yourself).
    pub path_loss: PathLoss,
    /// Optional frequency-dependent loss (rain fade, atmospheric absorption, etc.) in dB.
    #[doc(alias = "rain fade")]
    pub frequency_dependent_loss: Option<f64>,
}

impl LinkBudget {
    /// Total path loss in dB, including any frequency-dependent loss.
    #[doc(alias = "FSPL")]
    #[must_use]
    pub fn path_loss(&self) -> f64 {
        let path_loss_in_db: f64 = self.path_loss.calculate();

        let mut total_path_loss_in_db: f64 = path_loss_in_db;
        if let Some(frequency_dependent_loss) = self.frequency_dependent_loss {
            total_path_loss_in_db += frequency_dependent_loss;
        }

        total_path_loss_in_db
    }

    /// Received power at the receiver input in dBm.
    ///
    /// Friis gives `C = P_tx*G_tx*G_rx/(L_fs*L_extra)` in linear units.
    /// Its logarithm adds antenna gains and subtracts path losses.
    #[must_use]
    pub fn pin_at_receiver(&self) -> f64 {
        let path_loss_in_db = self.path_loss();

        self.transmitter.eirp_dbm() - path_loss_in_db + self.receiver.gain
    }

    /// SNR at the receiver in dB.
    ///
    /// This is `C/(k*T_sys*B_noise)`, integrated over receiver noise bandwidth.
    /// It is not automatically the SNR over the channel bandwidth.
    #[doc(alias = "SNR")]
    #[must_use]
    pub fn snr(&self) -> f64 {
        self.receiver.calculate_snr(self.pin_at_receiver())
    }

    /// SNR at the receiver (linear, not dB).
    #[doc(alias = "SNR")]
    #[must_use]
    pub fn snr_linear(&self) -> f64 {
        10.0_f64.powf(self.snr() / 10.0)
    }

    /// Ideal AWGN channel capacity over this link's channel bandwidth.
    ///
    /// Uses `B * log2(1 + (C/No)/B)`, where `B` is [`Self::bandwidth`].
    /// Receiver noise bandwidth sets the integrated noise used to infer C/No;
    /// channel bandwidth sets the noise power in the capacity calculation.
    /// Assumes white noise, a flat channel, and all received signal power within
    /// the channel and receiver passband. Filter clipping and distortion are
    /// not modeled. This is a capacity bound, not a modulation's throughput.
    #[must_use]
    pub fn phy_rate(&self) -> PhyRate {
        let channel_snr_db = energy::c_over_no_to_snr(self.c_over_no(), self.bandwidth);
        PhyRate {
            bandwidth: self.bandwidth,
            snr: 10.0_f64.powf(channel_snr_db / 10.0),
        }
    }

    // ----- Modulation-aware methods -----

    /// C/No in dB·Hz from the link budget SNR and noise bandwidth.
    ///
    /// `C/No = SNR(dB) + 10·log10(noise_bandwidth)`
    /// This follows by multiplying `C/(N0*B_noise)` by `B_noise`; the linear
    /// result has units of Hz. It is invariant to noise bandwidth when the
    /// signal power and flat noise density are fixed.
    #[doc(alias = "C/N")]
    #[must_use]
    pub fn c_over_no(&self) -> f64 {
        energy::snr_to_c_over_no(self.snr(), self.receiver.bandwidth)
    }

    /// Eb/No in dB for a given modulation scheme (uncoded).
    ///
    /// Assumes the channel bandwidth is the symbol rate (zero roll-off).
    /// Receiver noise bandwidth may differ from this channel bandwidth.
    /// During time `t`, signal energy is `C*t` and information bits are
    /// `Rb*t`, so `Eb = C/Rb` with `Rb = Rs*log2(M)`.
    #[doc(alias = "Eb/N0")]
    #[must_use]
    pub fn eb_no_db(&self, modulation: &Modulation) -> f64 {
        let info_bit_rate_bps = self.bandwidth * modulation.bits_per_symbol();
        energy::c_over_no_to_eb_over_no(self.c_over_no(), info_bit_rate_bps)
    }

    /// Eb/No in dB for a coded modulation scheme.
    ///
    /// Assumes the channel bandwidth is the symbol rate (zero roll-off).
    /// Receiver noise bandwidth may differ from this channel bandwidth.
    /// Each symbol carries `r*log2(M)` information bits. Thus
    /// `Eb/N0 = (C/N0)/(Rs*r*log2(M))`; code rate changes energy per information
    /// bit at fixed symbol rate, separately from any assumed decoder gain.
    #[doc(alias = "Eb/N0")]
    #[must_use]
    pub fn eb_no_coded_db(&self, coded_mod: &CodedModulation) -> f64 {
        let info_bit_rate_bps = coded_mod.throughput_bps(self.bandwidth);
        energy::c_over_no_to_eb_over_no(self.c_over_no(), info_bit_rate_bps)
    }

    /// BER for a given modulation at the link's Eb/No (uncoded).
    #[doc(alias = "BER")]
    #[must_use]
    pub fn ber(&self, modulation: &Modulation) -> f64 {
        let eb_no_db = self.eb_no_db(modulation);
        ber::ber_from_db(eb_no_db, modulation)
    }

    /// BER for a coded modulation scheme at the link's Eb/No.
    ///
    /// Uses the coding module's approximate horizontal shift of an uncoded
    /// BER curve. This is not a decoder simulation or a guaranteed coded BER.
    #[doc(alias = "BER")]
    #[must_use]
    pub fn ber_coded(&self, coded_mod: &CodedModulation) -> f64 {
        let eb_no_db = self.eb_no_coded_db(coded_mod);
        coded_mod.ber_from_db(eb_no_db)
    }

    /// Link margin in dB for a given modulation and target BER (uncoded).
    ///
    /// Positive margin = link closes with headroom.
    /// Negative margin = link does not close.
    #[doc(alias = "link margin")]
    #[must_use]
    pub fn link_margin_db(&self, modulation: &Modulation, target_ber: f64) -> Option<f64> {
        let actual = self.eb_no_db(modulation);
        let required = ber::required_eb_no_db(target_ber, modulation)?;
        let margin = actual - required;
        tracing::debug!(
            actual_eb_no_db = actual,
            required_eb_no_db = required,
            margin_db = margin,
            target_ber,
            "Link margin"
        );
        Some(margin)
    }

    /// Link margin in dB for a coded modulation and target BER.
    #[doc(alias = "link margin")]
    #[must_use]
    pub fn link_margin_coded_db(
        &self,
        coded_mod: &CodedModulation,
        target_ber: f64,
    ) -> Option<f64> {
        let actual = self.eb_no_coded_db(coded_mod);
        coded_mod.link_margin_db(actual, target_ber)
    }

    /// Configured information rate in bits/s for a coded modulation scheme.
    ///
    /// Counts `Rs*r*log2(M)` with `Rs = self.bandwidth` (zero roll-off).
    /// This is a rate calculation; it does not establish that the link meets
    /// a target BER, channel-capacity bound, or application overhead budget.
    #[must_use]
    pub fn throughput_bps(&self, coded_mod: &CodedModulation) -> f64 {
        coded_mod.throughput_bps(self.bandwidth)
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    use crate::coding::dvbs2_qpsk_r34;
    use crate::coding::FecCode;
    use crate::receiver::ReceiverNoise;

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
                noise: ReceiverNoise::SourceAndNoiseFigure {
                    source_temperature_k: 290.0,
                    noise_figure_db: 2.0,
                },
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
        let expected = b.snr() + 10.0 * 36e6_f64.log10();
        assert!((c_no - expected).abs() < 0.01);
    }

    #[test]
    fn eb_no_less_than_snr_for_qpsk() {
        let b = sample_budget();
        let eb_no = b.eb_no_db(&Modulation::Qpsk);
        let expected = b.snr() - 10.0 * 2.0_f64.log10();
        assert!((eb_no - expected).abs() < 0.01);
    }

    #[test]
    fn eb_no_matches_energy_chain_when_noise_and_channel_bandwidths_differ() {
        let mut b = sample_budget();
        b.bandwidth = 20e6; // channel bandwidth and assumed symbol rate
        b.receiver.bandwidth = 30e6; // receiver noise bandwidth
        let uncoded = Modulation::Qpsk;
        let coded = CodedModulation::new(Modulation::Qpsk, FecCode::Ldpc { rate: 0.75 });

        let expected_uncoded =
            energy::snr_to_eb_over_no(b.snr(), b.receiver.bandwidth, &uncoded, b.bandwidth, 1.0);
        let expected_coded = energy::snr_to_eb_over_no(
            b.snr(),
            b.receiver.bandwidth,
            &coded.modulation,
            b.bandwidth,
            coded.code_rate(),
        );

        assert!((b.eb_no_db(&uncoded) - expected_uncoded).abs() < 1e-10);
        assert!((b.eb_no_coded_db(&coded) - expected_coded).abs() < 1e-10);
        let bandwidth_correction_db = 10.0 * (b.receiver.bandwidth / b.bandwidth).log10();
        let eb_no_from_snr = b.snr() - 10.0 * uncoded.bits_per_symbol().log10();
        assert!((b.eb_no_db(&uncoded) - eb_no_from_snr - bandwidth_correction_db).abs() < 1e-10);
        assert!(
            (b.eb_no_coded_db(&coded) - b.eb_no_db(&uncoded) + 10.0 * coded.code_rate().log10())
                .abs()
                < 1e-10
        );

        let uncoded_as_coded = CodedModulation::new(Modulation::Qpsk, FecCode::Uncoded);
        assert!((b.eb_no_db(&uncoded) - b.eb_no_coded_db(&uncoded_as_coded)).abs() < 1e-10);

        let original_snr = b.snr();
        let original_eb_no = b.eb_no_db(&uncoded);
        b.receiver.bandwidth *= 2.0;
        assert!((b.snr() - original_snr + 10.0 * 2.0_f64.log10()).abs() < 1e-10);
        assert!((b.eb_no_db(&uncoded) - original_eb_no).abs() < 1e-10);
    }

    #[test]
    fn ber_returns_valid_value() {
        let b = sample_budget();
        let ber_val = b.ber(&Modulation::Qpsk);
        assert!((0.0..=0.5).contains(&ber_val));
    }

    #[test]
    fn link_margin_qpsk() {
        let b = sample_budget();
        let margin = b.link_margin_db(&Modulation::Qpsk, 1e-5);
        assert!(margin.is_some());
        let m = margin.unwrap();
        assert!(m > -50.0 && m < 100.0);
    }

    #[test]
    fn coded_margin_better_than_uncoded() {
        let b = sample_budget();
        let uncoded_margin = b.link_margin_db(&Modulation::Qpsk, 1e-5).unwrap();

        let coded = dvbs2_qpsk_r34();
        let coded_margin = b.link_margin_coded_db(&coded, 1e-5).unwrap();

        assert!(
            coded_margin > uncoded_margin,
            "Coded margin ({:.1} dB) should exceed uncoded ({:.1} dB)",
            coded_margin,
            uncoded_margin
        );
    }

    #[test]
    fn throughput_matches_spectral_efficiency() {
        let b = sample_budget();
        let cm = dvbs2_qpsk_r34();
        let tp = b.throughput_bps(&cm);
        assert!((tp - 54e6).abs() < 1.0);
    }

    #[test]
    fn coded_ber_lower_than_uncoded() {
        let b = sample_budget();
        let ber_uncoded = b.ber(&Modulation::Qpsk);

        let coded = dvbs2_qpsk_r34();
        let ber_coded = b.ber_coded(&coded);

        assert!(
            ber_coded < ber_uncoded,
            "Coded BER ({:.2e}) should be lower than uncoded ({:.2e})",
            ber_coded,
            ber_uncoded
        );
    }
}
