//! Public workflows anchored to published data or independent analytic values.

use linkbudget::dvbs2::{self, Mode};

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
