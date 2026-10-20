//! Public workflows anchored to published data or independent analytic values.

use linkbudget::atmosphere::sky_noise_temperature_k;
use linkbudget::dvbs2::{self, Mode};
use linkbudget::{LinkBudget, ModelError, PathLoss, Receiver, ReceiverNoise, Transmitter};

#[test]
fn atmospheric_emission_and_attenuation_reduce_downlink_snr_together() {
    let downlink = |atmospheric_attenuation_db| LinkBudget {
        name: "Atmospheric emission: Ka-band LEO downlink",
        bandwidth: 20e6,
        transmitter: Transmitter {
            output_power: 10.0,
            gain: 35.0,
            bandwidth: 20e6,
        },
        receiver: Receiver {
            gain: 40.0,
            // Ideal sky-filled beam; no spillover or feed loss in this scenario.
            noise: ReceiverNoise::SourceAndNoiseFigure {
                source_temperature_k: sky_noise_temperature_k(atmospheric_attenuation_db, 275.0)
                    .unwrap(),
                noise_figure_db: 1.0,
            },
            bandwidth: 20e6,
        },
        path_loss: PathLoss {
            frequency: 20e9,
            distance: 550e3,
        },
        frequency_dependent_loss: Some(atmospheric_attenuation_db),
    };

    // Independent 50-digit decimal evaluation of P.618-14 eq. 69, Friis
    // (c = 299792458 m/s), and k*T*B (k = 1.380649e-23 J/K).
    let clear = downlink(0.0);
    let attenuated = downlink(3.0);
    for (budget, expected_temperature_k, expected_power_dbm, expected_noise_dbm, expected_snr_db) in [
        (
            &clear,
            77.788_369_420_308_49,
            -88.275_636_925_047_87,
            -106.679_720_536_420_24,
            18.404_083_611_372_366,
        ),
        (
            &attenuated,
            213.615_085_703_602_25,
            -91.275_636_925_047_87,
            -102.292_548_019_226,
            11.016_911_094_178_127,
        ),
    ] {
        assert!(
            (budget.receiver.system_noise_temperature_k() - expected_temperature_k).abs() < 1e-10
        );
        assert!((budget.pin_at_receiver() - expected_power_dbm).abs() < 1e-10);
        assert!((budget.receiver.calculate_noise_power() - expected_noise_dbm).abs() < 1e-10);
        assert!((budget.snr() - expected_snr_db).abs() < 1e-10);
    }
    assert!((sky_noise_temperature_k(3.0, 275.0).unwrap() - 138.526_716_283_293_76).abs() < 1e-10);
    assert!((clear.snr() - attenuated.snr() - 7.387_172_517_194_239).abs() < 1e-10);
    assert!(clear.snr() - attenuated.snr() > 3.0);
}

#[test]
fn atmospheric_sky_brightness_boundaries_and_invalid_inputs() {
    assert_eq!(sky_noise_temperature_k(0.0, 275.0).unwrap(), 2.7);
    assert_eq!(sky_noise_temperature_k(1e6, 275.0).unwrap(), 275.0);
    assert_eq!(sky_noise_temperature_k(f64::MAX, 275.0).unwrap(), 275.0);
    assert!(
        (sky_noise_temperature_k(1e-12, 275.0).unwrap() - 2.700_000_000_062_699_4).abs() < 1e-15
    );
    for attenuation_db in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            sky_noise_temperature_k(attenuation_db, 275.0),
            Err(ModelError::InvalidInput("atmospheric_attenuation_db"))
        );
    }
    for temperature_k in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            sky_noise_temperature_k(3.0, temperature_k),
            Err(ModelError::InvalidInput("mean_radiating_temperature_k"))
        );
    }
}

#[test]
fn dvbs2_normal_frame_selection_and_rate_from_explicit_allocation() {
    // ETSI EN 302 307-1 V1.4.1 Table 13: independent source values.
    let rows = [
        (Mode::QpskR12, 1.00, 0.988858),
        (Mode::QpskR34, 4.03, 1.487473),
        (Mode::Psk8R23, 6.62, 1.980636),
        (Mode::Apsk16R34, 10.21, 2.966728),
        (Mode::Apsk32R56, 14.28, 4.119540),
    ];
    for (mode, threshold_db, efficiency) in rows {
        assert_eq!(mode.required_es_no_db(), threshold_db);
        assert_eq!(mode.information_bits_per_symbol(), efficiency);
    }
    assert_eq!(dvbs2::REFERENCE.fecframe_bits, 64_800);
    assert_eq!(dvbs2::REFERENCE.packet_error_ratio, 1e-7);

    // 12 MHz occupied at alpha=.2 gives 10 Msymbol/s, independently of B_noise.
    let rs = dvbs2::symbol_rate_from_occupied_bandwidth(12e6, 0.2).unwrap();
    assert_eq!(rs, 10e6);
    let c_no_db_hz = 74.5; // Es/N0 = 4.5 dB at this symbol rate.
    let candidates = [Mode::QpskR34, Mode::QpskR12];
    let selected = dvbs2::select_mode(&candidates, c_no_db_hz, rs, 0.5)
        .unwrap()
        .unwrap();
    assert_eq!(selected, Mode::QpskR12);
    assert!((selected.margin_db(c_no_db_hz, rs, 0.5).unwrap() - 3.0).abs() < 1e-12);
    assert!((Mode::QpskR34.margin_db(c_no_db_hz, rs, 0.5).unwrap() + 0.03).abs() < 1e-12);
    assert_eq!(selected.information_rate_bps(rs).unwrap(), 9_888_580.0);
    assert_eq!(
        dvbs2::select_mode(&candidates, 60.0, rs, 0.5).unwrap(),
        None
    );
    assert!(dvbs2::select_mode(&[], f64::NAN, rs, 0.5).is_err());
    assert!(Mode::QpskR12.margin_db(c_no_db_hz, 0.0, 0.5).is_err());
    assert!(dvbs2::symbol_rate_from_occupied_bandwidth(12e6, 1.1).is_err());
}
