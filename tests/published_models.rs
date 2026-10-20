//! Public workflows anchored to published data or independent analytic values.

use linkbudget::atmosphere::sky_noise_temperature_k;
use linkbudget::dvbs2::{self, Mode};
use linkbudget::rain::{specific_attenuation_db_per_km, RainPath};
use linkbudget::{ber, Modulation};
use linkbudget::{LinkBudget, ModelError, PathLoss, Receiver, ReceiverNoise, Transmitter};

#[test]
fn gaussian_tails_match_independent_high_precision_references() {
    // 1200-digit stdlib Decimal erf Taylor series, with Machin pi. This is
    // independent of the gamma continued fraction used for positive tails.
    for (x, expected) in [
        (0.0, 1.0),
        (0.125, 0.859_683_795_198_666_2),
        (0.5, 0.479_500_122_186_953_5),
        (1.0, 0.157_299_207_050_285_13),
        (1.499_999_999_999, 0.033_894_853_524_808_21),
        (1.5, 0.033_894_853_524_689_274),
        (1.500_000_000_001, 0.033_894_853_524_570_334),
        (2.0, 0.004_677_734_981_047_266),
        (4.0, 1.541_725_790_028_002e-8),
        (8.0, 1.122_429_717_298_292_6e-29),
        (10.0, 2.088_487_583_762_545e-45),
        (20.0, 5.395_865_611_607_901e-176),
        (26.0, 5.663_192_408_856_143e-296),
    ] {
        let actual = ber::erfc(x);
        assert!(
            (actual / expected - 1.0).abs() < 1e-12,
            "erfc({x})={actual}"
        );
        assert!((ber::erfc(-x) - (2.0 - expected)).abs() < 2e-15);
    }
    assert_eq!(ber::erfc(27.0), 5.237_05e-319);
    assert_eq!(ber::erfc(27.2), f64::from_bits(2));
    assert_eq!(ber::erfc(27.25), 0.0);
    assert_eq!(ber::erfc(f64::INFINITY), 0.0);
    assert_eq!(ber::erfc(f64::NEG_INFINITY), 2.0);
    assert!(ber::erfc(f64::NAN).is_nan());
    assert_eq!(ber::q_function(f64::INFINITY), 0.0);
    assert_eq!(ber::q_function(f64::NEG_INFINITY), 1.0);
    assert!(ber::q_function(f64::NAN).is_nan());
    assert_eq!(ber::q_function(0.0), 0.5);
    assert_eq!(ber::erfc(-0.0), 1.0);
    assert_eq!(ber::erfc(f64::from_bits(1)), 1.0);
}

#[test]
fn uncoded_ber_and_log_threshold_workflows_match_independent_references() {
    for (db, modulation, expected, log_expected) in [
        (
            0.0,
            Modulation::Bpsk,
            0.078_649_603_525_142_57,
            -2.542_752_690_493_193_4,
        ),
        (
            10.0,
            Modulation::Qpsk,
            3.872_108_215_522_042e-6,
            -12.461_711_440_716_82,
        ),
        (
            20.0,
            Modulation::Msk,
            1.044_243_791_881_272_4e-45,
            -103.573_036_205_404_83,
        ),
        (
            10.0,
            Modulation::Mpsk(8),
            0.001_011_395_320_712_890_7,
            -6.896_424_395_873_596,
        ),
        (
            15.0,
            Modulation::Mqam(16),
            1.841_855_511_094_483_2e-7,
            -15.507_322_157_518_67,
        ),
        (
            20.0,
            Modulation::Mqam(64),
            2.633_892_531_773_040_7e-8,
            -17.452_217_941_853_52,
        ),
    ] {
        assert!((ber::ber_from_db(db, &modulation) / expected - 1.0).abs() < 1e-12);
        assert!((ber::log_ber_from_db(db, &modulation).unwrap() - log_expected).abs() < 1e-12);
        // Invert independent literal probabilities, not crate-generated BER.
        assert!((ber::required_eb_no_db(expected, &modulation).unwrap() - db).abs() < 1e-11);
        assert!(
            (ber::required_eb_no_from_log_ber(log_expected, &modulation).unwrap() - db).abs()
                < 1e-11
        );
    }
    for modulation in [
        Modulation::Bpsk,
        Modulation::Qpsk,
        Modulation::Mpsk(2),
        Modulation::Mpsk(4),
        Modulation::Mqam(4),
        Modulation::Msk,
    ] {
        for (target, required) in [
            (1e-6, 10.529_831_699_571_448),
            (1e-100, 23.546_460_034_850_334),
            (1e-300, 28.364_783_528_218_62),
        ] {
            let result = ber::required_eb_no_db(target, &modulation).unwrap();
            assert!((result - required).abs() < 1e-11);
            assert!(
                (ber::required_eb_no_from_log_ber(target.ln(), &modulation).unwrap() - required)
                    .abs()
                    < 1e-11
            );
        }
    }
    let modulation = Modulation::Qpsk;
    assert_eq!(ber::ber_from_db(30.0, &modulation), 0.0);
    assert!(
        (ber::log_ber_from_db(30.0, &modulation).unwrap() + 1_004.719_889_139_511_9).abs() < 2e-12
    );
    let threshold = ber::required_eb_no_from_log_ber(-1000.0, &modulation).unwrap();
    assert!((threshold - 29.979_463_561_086_86).abs() < 1e-11);
    assert!(
        (ber::link_margin_db(12.0, 1e-6, &modulation).unwrap() - 1.470_168_300_428_552).abs()
            < 1e-11
    );
    // Compatibility is intentional: the legacy heuristic 32-APSK/QAM preset
    // still works, while checked APIs refuse its unsupported square geometry.
    assert!(linkbudget::coding::dvbs2_32apsk_r56()
        .required_eb_no_db(1e-6)
        .is_some());
    for invalid in [
        Modulation::Mqam(0),
        Modulation::Mqam(2),
        Modulation::Mqam(32),
        Modulation::Mpsk(1),
        Modulation::Mpsk(3),
    ] {
        assert!(matches!(
            ber::log_ber_from_db(10.0, &invalid),
            Err(ModelError::UnsupportedRegime(_))
        ));
        assert!(matches!(
            ber::required_eb_no_from_log_ber(-10.0, &invalid),
            Err(ModelError::UnsupportedRegime(_))
        ));
    }
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            ber::log_ber_from_db(invalid, &modulation),
            Err(ModelError::InvalidInput(_))
        ));
    }
    assert!(matches!(
        ber::log_ber_from_db(f64::MAX, &modulation),
        Err(ModelError::NumericalFailure(_))
    ));
    assert_eq!(
        ber::log_ber_from_db(-f64::MAX, &modulation).unwrap(),
        -std::f64::consts::LN_2
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, 1.0] {
        assert!(matches!(
            ber::required_eb_no_from_log_ber(invalid, &modulation),
            Err(ModelError::InvalidInput(_))
        ));
    }
    for outside in [-0.1, -1e6] {
        assert!(matches!(
            ber::required_eb_no_from_log_ber(outside, &modulation),
            Err(ModelError::UnsupportedRegime(_))
        ));
    }
    for db in [-5.0, 50.0] {
        let log_target = ber::log_ber_from_db(db, &modulation).unwrap();
        assert_eq!(
            ber::required_eb_no_from_log_ber(log_target, &modulation),
            Ok(db)
        );
    }
    let endpoint = ber::ber_from_db(-5.0, &modulation);
    assert_eq!(ber::required_eb_no_db(endpoint, &modulation), Some(-5.0));
    assert_eq!(ber::required_eb_no_db(0.5, &modulation), None);
    assert!(ber::required_eb_no_db(f64::from_bits(1), &modulation)
        .unwrap()
        .is_finite());
}

#[test]
fn inverse_gaussian_tail_supports_full_probability_domain() {
    for (probability, expected) in [
        (0.1, 1.281_551_565_544_600_4),
        (1e-100, 21.273_453_560_965_326),
        (1e-300, 37.047_096_299_361_2),
        (f64::from_bits(1), 38.467_405_617_144_344),
    ] {
        let actual = ber::inverse_q_function(probability).unwrap();
        assert!((actual - expected).abs() < 3e-14, "{probability}: {actual}");
    }
    assert_eq!(ber::inverse_q_function(0.5), Ok(0.0));
    for probability in [
        0.125,
        0.25,
        0.499_999_999_999_999_94,
        f64::from_bits(0x3fef_ffff_ffff_ffff),
    ] {
        let actual = ber::inverse_q_function(probability).unwrap();
        assert!((ber::q_function(actual) - probability).abs() < 3e-16);
        let complement = 1.0 - probability;
        // Only exact representable complements promise exact sign symmetry.
        if 1.0 - complement == probability {
            assert_eq!(ber::inverse_q_function(complement).unwrap(), -actual);
        }
    }
    for invalid in [
        0.0,
        1.0,
        -0.1,
        1.1,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert_eq!(
            ber::inverse_q_function(invalid),
            Err(ModelError::InvalidInput("probability"))
        );
    }
}

#[test]
fn rain_specific_attenuation_matches_published_and_independent_values() {
    // ITU-Rpy pinned CSVs and offline numerical evaluation; provenance and
    // precision distinctions are recorded in docs/production-models.md.
    for (frequency_ghz, rain_rate, elevation, tilt, expected) in [
        (14.25, 26.48052, 31.07699124, 0.0, 1.58130839),
        (29.0, 26.48052, 31.07699124, 0.0, 5.02180189),
        (14.25, 63.62668149, 48.24117054, 90.0, 3.72901264),
        (29.0, 42.91007183, 20.14335809, 90.0, 6.83364556),
    ] {
        let actual =
            specific_attenuation_db_per_km(frequency_ghz * 1e9, rain_rate, elevation, tilt)
                .unwrap();
        assert!((actual - expected).abs() < 1e-8, "{actual} != {expected}");
    }
    for (frequency_ghz, rain_rate, elevation, tilt, expected) in [
        (1.0, 25.0, 0.0, 0.0, 0.000_585_983_457_747_974_3),
        (10.0, 25.0, 0.0, 0.0, 0.695_871_528_099_342),
        (10.0, 25.0, 0.0, 90.0, 0.565_151_233_006_795_8),
        (20.0, 50.0, 30.0, 45.0, 5.073_415_344_222_843),
        (55.0, 100.0, 5.0, 90.0, 25.643_906_239_423_803),
        (100.0, 25.0, 60.0, 0.0, 12.188_947_321_409_204),
        (1000.0, 25.0, 90.0, 45.0, 10.767_074_998_092_73),
    ] {
        let actual =
            specific_attenuation_db_per_km(frequency_ghz * 1e9, rain_rate, elevation, tilt)
                .unwrap();
        assert!((actual - expected).abs() < 1e-11, "{actual} != {expected}");
    }
}

#[test]
fn annual_rain_quantiles_match_validation_examples_with_explicit_climate_inputs() {
    // P.618-14 A_rain CSV values, with rain heights from the P.839-4 CSV.
    // Each row carries all RainPath inputs; percentages are .001, .01, .1, 1.
    let cases = [
        (
            14.25e9,
            31.07699124,
            51.5,
            31.382984,
            2452.73333,
            26.48052,
            0.0,
            [14.89982248, 6.798072267, 2.185847422, 0.495317069],
        ),
        (
            29e9,
            40.232036,
            41.9,
            46.122988,
            3047.49333,
            33.936232,
            0.0,
            [54.25561152, 28.74272193, 10.73100773, 2.823465634],
        ),
        (
            14.25e9,
            22.27833468,
            22.9,
            0.0,
            4158.77867,
            50.639304,
            0.0,
            [29.91171296, 18.94410356, 8.271647438, 1.706901281],
        ),
        (
            29e9,
            52.67898486,
            25.78,
            8.61728,
            4569.46133,
            78.2994993,
            0.0,
            [93.39562548, 58.47438336, 25.56295492, 6.654855651],
        ),
    ];
    for (
        frequency_hz,
        elevation_deg,
        latitude_deg,
        station_height_m,
        rain_height_m,
        rain_rate_001_mm_h,
        polarization_tilt_deg,
        expected,
    ) in cases
    {
        let path = RainPath {
            frequency_hz,
            elevation_deg,
            latitude_deg,
            station_height_m,
            rain_height_m,
            rain_rate_001_mm_h,
            polarization_tilt_deg,
        };
        for (percent, expected_db) in [0.001, 0.01, 0.1, 1.0].into_iter().zip(expected) {
            let actual = path.attenuation_exceeded_db(percent).unwrap();
            assert!(
                (actual - expected_db).abs() < 1e-6,
                "{path:?}, p={percent}: {actual} != {expected_db}"
            );
        }
    }
}

#[test]
fn rain_low_elevation_tropical_and_zenith_paths_match_independent_numerics() {
    // Additional ITU-Rpy numeric-routine cases, not official validation rows.
    let cases = [
        (
            20e9,
            2.0,
            10.0,
            100.0,
            5000.0,
            80.0,
            45.0,
            [
                233.35950039772555,
                171.65028526647654,
                87.72778671197015,
                24.419448751180017,
                8.879893615682954,
            ],
        ),
        (
            20e9,
            5.0,
            10.0,
            100.0,
            5000.0,
            80.0,
            45.0,
            [
                139.25163215841013,
                116.59775871644676,
                66.70412867060108,
                15.31002525141487,
                5.413570179020045,
            ],
        ),
        (
            20e9,
            30.0,
            -10.0,
            100.0,
            5000.0,
            80.0,
            90.0,
            [
                63.089342779084845,
                40.183985868474714,
                17.772331503917453,
                4.231215864473713,
                1.385053930335913,
            ],
        ),
        (
            55e9,
            90.0,
            36.0,
            0.0,
            4000.0,
            30.0,
            0.0,
            [
                118.63478933301931,
                68.79712252303197,
                28.11628057257508,
                8.097945448564392,
                2.7560606314539116,
            ],
        ),
        (
            1e9,
            10.0,
            45.0,
            -400.0,
            2500.0,
            10.0,
            0.0,
            [
                0.012273613634145288,
                0.0024641205973225722,
                0.0003486429229419937,
                0.00003476395129432661,
                0.0000056370183762336745,
            ],
        ),
    ];
    for (
        frequency_hz,
        elevation_deg,
        latitude_deg,
        station_height_m,
        rain_height_m,
        rain_rate_001_mm_h,
        polarization_tilt_deg,
        expected,
    ) in cases
    {
        let path = RainPath {
            frequency_hz,
            elevation_deg,
            latitude_deg,
            station_height_m,
            rain_height_m,
            rain_rate_001_mm_h,
            polarization_tilt_deg,
        };
        for (percent, expected_db) in [0.001, 0.01, 0.1, 1.0, 5.0].into_iter().zip(expected) {
            let actual = path.attenuation_exceeded_db(percent).unwrap();
            assert!(
                (actual - expected_db).abs() < 1e-10,
                "{path:?}, p={percent}: {actual} != {expected_db}"
            );
            let mirrored = RainPath {
                latitude_deg: -latitude_deg,
                ..path
            };
            assert_eq!(mirrored.attenuation_exceeded_db(percent).unwrap(), actual);
        }
    }
}

fn tropical_rain_path() -> RainPath {
    RainPath {
        frequency_hz: 20e9,
        elevation_deg: 30.0,
        latitude_deg: -10.0,
        station_height_m: 100.0,
        rain_height_m: 5000.0,
        rain_rate_001_mm_h: 80.0,
        polarization_tilt_deg: 90.0,
    }
}

#[test]
fn rain_boundary_regimes_and_invalid_inputs_are_explicit() {
    let base = tropical_rain_path();
    for percent in [0.001, 0.01, 0.1, 1.0, 5.0] {
        for path in [
            RainPath {
                rain_rate_001_mm_h: 0.0,
                ..base
            },
            RainPath {
                station_height_m: 5000.0,
                ..base
            },
            RainPath {
                station_height_m: 6000.0,
                ..base
            },
        ] {
            assert_eq!(path.attenuation_exceeded_db(percent), Ok(0.0));
        }
    }
    assert_eq!(specific_attenuation_db_per_km(1e9, 0.0, 0.0, 0.0), Ok(0.0));
    let circular = specific_attenuation_db_per_km(20e9, 50.0, 30.0, 45.0).unwrap();
    assert_eq!(
        specific_attenuation_db_per_km(20e9, 50.0, 30.0, -135.0).unwrap(),
        circular
    );
    assert!(specific_attenuation_db_per_km(20e9, 50.0, 30.0, f64::MAX)
        .unwrap()
        .is_finite());

    // P.618-14 step 10 specifies theta >= 25 (the pinned reference uses >).
    // A.001 from the reference routine; eq. 8 evaluated separately with beta=.13.
    let at_25 = RainPath {
        elevation_deg: 25.0,
        latitude_deg: 10.0,
        polarization_tilt_deg: 45.0,
        ..base
    };
    assert!((at_25.attenuation_exceeded_db(0.01).unwrap() - 46.863_003_643_322_216).abs() < 1e-10);
    assert!((at_25.attenuation_exceeded_db(0.1).unwrap() - 20.624_671_943_929_91).abs() < 1e-10);

    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for field in [
            "frequency_hz",
            "elevation_deg",
            "latitude_deg",
            "station_height_m",
            "rain_height_m",
            "rain_rate_001_mm_h",
            "polarization_tilt_deg",
        ] {
            let mut path = base;
            match field {
                "frequency_hz" => path.frequency_hz = value,
                "elevation_deg" => path.elevation_deg = value,
                "latitude_deg" => path.latitude_deg = value,
                "station_height_m" => path.station_height_m = value,
                "rain_height_m" => path.rain_height_m = value,
                "rain_rate_001_mm_h" => path.rain_rate_001_mm_h = value,
                "polarization_tilt_deg" => path.polarization_tilt_deg = value,
                _ => unreachable!(),
            }
            assert_eq!(
                path.attenuation_exceeded_db(0.1),
                Err(ModelError::InvalidInput(field))
            );
        }
        assert_eq!(
            base.attenuation_exceeded_db(value),
            Err(ModelError::InvalidInput("exceedance_percent"))
        );
        for result in [
            specific_attenuation_db_per_km(value, 25.0, 30.0, 0.0),
            specific_attenuation_db_per_km(20e9, value, 30.0, 0.0),
            specific_attenuation_db_per_km(20e9, 25.0, value, 0.0),
            specific_attenuation_db_per_km(20e9, 25.0, 30.0, value),
        ] {
            assert!(matches!(result, Err(ModelError::InvalidInput(_))));
        }
    }
    for path in [
        RainPath {
            frequency_hz: 0.0,
            ..base
        },
        RainPath {
            frequency_hz: -1.0,
            ..base
        },
        RainPath {
            elevation_deg: 0.0,
            ..base
        },
        RainPath {
            elevation_deg: -1.0,
            ..base
        },
        RainPath {
            elevation_deg: 90.1,
            ..base
        },
        RainPath {
            latitude_deg: -90.1,
            ..base
        },
        RainPath {
            latitude_deg: 90.1,
            ..base
        },
        RainPath {
            rain_height_m: -1.0,
            ..base
        },
        RainPath {
            rain_rate_001_mm_h: -1.0,
            ..base
        },
    ] {
        assert!(matches!(
            path.attenuation_exceeded_db(0.1),
            Err(ModelError::InvalidInput(_))
        ));
    }
    for frequency_hz in [0.999e9, 55.001e9] {
        assert!(matches!(
            RainPath {
                frequency_hz,
                ..base
            }
            .attenuation_exceeded_db(0.1),
            Err(ModelError::UnsupportedRegime(_))
        ));
    }
    for percent in [0.0, -0.1] {
        assert_eq!(
            base.attenuation_exceeded_db(percent),
            Err(ModelError::InvalidInput("exceedance_percent"))
        );
    }
    for percent in [0.000999, 5.001, 99.9] {
        assert!(matches!(
            base.attenuation_exceeded_db(percent),
            Err(ModelError::UnsupportedRegime(_))
        ));
    }
    // Early physical-zero cases must not hide invalid inputs.
    assert!(RainPath {
        rain_rate_001_mm_h: 0.0,
        latitude_deg: 91.0,
        ..base
    }
    .attenuation_exceeded_db(0.1)
    .is_err());
    assert!(RainPath {
        station_height_m: 6000.0,
        ..base
    }
    .attenuation_exceeded_db(99.9)
    .is_err());
    for result in [
        specific_attenuation_db_per_km(0.999e9, 25.0, 30.0, 0.0),
        specific_attenuation_db_per_km(1000.001e9, 25.0, 30.0, 0.0),
    ] {
        assert!(matches!(result, Err(ModelError::UnsupportedRegime(_))));
    }
    for result in [
        specific_attenuation_db_per_km(0.0, 25.0, 30.0, 0.0),
        specific_attenuation_db_per_km(20e9, -1.0, 30.0, 0.0),
        specific_attenuation_db_per_km(20e9, 25.0, -0.1, 0.0),
        specific_attenuation_db_per_km(20e9, 25.0, 90.1, 0.0),
    ] {
        assert!(matches!(result, Err(ModelError::InvalidInput(_))));
    }
    for result in [
        specific_attenuation_db_per_km(10e9, f64::MAX, 0.0, 0.0),
        RainPath {
            frequency_hz: 10e9,
            rain_rate_001_mm_h: f64::MAX,
            ..base
        }
        .attenuation_exceeded_db(0.1),
        RainPath {
            rain_height_m: f64::MAX,
            rain_rate_001_mm_h: 1e100,
            ..base
        }
        .attenuation_exceeded_db(0.1),
    ] {
        assert!(matches!(result, Err(ModelError::NumericalFailure(_))));
    }
}

#[test]
fn rain_quantile_is_explicit_additional_link_loss_not_automatic_noise() {
    let loss_db = tropical_rain_path().attenuation_exceeded_db(0.1).unwrap();
    let downlink = |loss| LinkBudget {
        name: "Rain-only 99.9% annual quantile, externally supplied system noise",
        bandwidth: 20e6,
        transmitter: Transmitter {
            output_power: 10.0,
            gain: 35.0,
            bandwidth: 20e6,
        },
        receiver: Receiver {
            gain: 40.0,
            // Fixed externally characterized noise isolates the signal-loss
            // contribution; this is not a prediction of wet-sky emission.
            noise: ReceiverNoise::SystemTemperature {
                temperature_k: 200.0,
            },
            bandwidth: 20e6,
        },
        path_loss: PathLoss {
            frequency: 20e9,
            distance: 550e3,
        },
        frequency_dependent_loss: loss,
    };
    let clear = downlink(None);
    let rainy = downlink(Some(loss_db));
    assert!((loss_db - 17.772_331_503_917_453).abs() < 1e-10);
    assert!((clear.pin_at_receiver() - rainy.pin_at_receiver() - loss_db).abs() < 1e-12);
    assert!((clear.c_over_no() - rainy.c_over_no() - loss_db).abs() < 1e-12);
    assert!((clear.snr() - rainy.snr() - loss_db).abs() < 1e-12);
    assert_eq!(
        clear.receiver.calculate_noise_power(),
        rainy.receiver.calculate_noise_power()
    );
}

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
