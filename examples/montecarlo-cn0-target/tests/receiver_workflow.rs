use linkbudget::{ber, energy, Modulation, Receiver, ReceiverNoise};

#[test]
fn fixed_receiver_margin_against_independent_thermal_noise_control() {
    // SI k=1.380649e-23, Tsys=50+290*(2-1)=340 K, C=-90 dBm.
    // C/(k*Tsys) gives 83.28437800279511 dB-Hz, independent of Bn.
    // Independent erfc inversion gives QPSK BER=1e-6 at 10.529831699571446 dB.
    let required = ber::required_eb_no_db(1e-6, &Modulation::Qpsk).unwrap();
    assert!((required - 10.529831699571446).abs() < 1e-10);
    for noise_bandwidth_hz in [5e6, 10e6, 25e6] {
        let receiver = Receiver {
            gain: 42.0,
            noise: ReceiverNoise::SourceAndNoiseFigure {
                source_temperature_k: 50.0,
                noise_figure_db: 10.0 * 2.0_f64.log10(),
            },
            bandwidth: noise_bandwidth_hz,
        };
        let c_no = energy::snr_to_c_over_no(receiver.calculate_snr(-90.0), noise_bandwidth_hz);
        assert!((c_no - 83.28437800279511).abs() < 1e-10);
        let eb_no = energy::c_over_no_to_eb_over_no(c_no, 10e6);
        assert!((eb_no - 13.284378002795108).abs() < 1e-10);
        assert!((eb_no - required - 2.7545463032236626).abs() < 1e-10);
        let doubled_rate_eb_no = energy::c_over_no_to_eb_over_no(c_no, 20e6);
        assert!((doubled_rate_eb_no - 10.274078046155296).abs() < 1e-10);
    }
}
