use linkbudget::Receiver;
use montycarlo::{MonteCarloEngine, Simulation};
use rand::Rng;
use std::fs::{create_dir_all, File};
use std::io::Write;

/// Monte Carlo analysis for receiver SNR margin against a design target.
///
/// Uses randomized `Receiver` parameters (temperature, noise figure, bandwidth)
/// plus randomized received power to emulate environmental + implementation spread.
struct ReceiverMarginSim {
    target_snr_db: f64,
}

impl Simulation for ReceiverMarginSim {
    // (temperature K, noise_figure dB, bandwidth Hz, input_power dBm)
    type Sample = (f64, f64, f64, f64);
    // SNR margin (dB): positive means target is met.
    type Output = f64;

    fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
        // Example realistic ranges for a small satcom receiver analysis case.
        let temperature_k = rng.gen_range(240.0..=380.0); // environment/system noise spread
        let noise_figure_db = rng.gen_range(1.2..=3.5); // RF front-end variation
        let bandwidth_hz = rng.gen_range(5.0e6..=25.0e6); // waveform/config variation
        let input_power_dbm = rng.gen_range(-107.0..=-97.0); // fading / pointing / rain spread

        (temperature_k, noise_figure_db, bandwidth_hz, input_power_dbm)
    }

    fn evaluate(&self, s: &Self::Sample) -> Self::Output {
        let rx = Receiver {
            gain: 42.0, // not used by SNR calc but representative of dish gain in link models
            temperature: s.0,
            noise_figure: s.1,
            bandwidth: s.2,
        };

        let snr_db = rx.calculate_snr(s.3);
        snr_db - self.target_snr_db
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let trials = 50_000;
    let target_snr_db = 10.0;

    let sim = ReceiverMarginSim { target_snr_db };
    let result = MonteCarloEngine::new(sim, trials).with_seed(99).run();

    create_dir_all("output")?;

    let mut csv = File::create("output/receiver_snr_margin_samples.csv")?;
    writeln!(csv, "snr_margin_db")?;
    for v in result.sorted_values() {
        writeln!(csv, "{v:.6}")?;
    }

    let mut txt = File::create("output/receiver_snr_margin_summary.txt")?;
    writeln!(txt, "trials={}", result.len())?;
    writeln!(txt, "target_snr_db={target_snr_db}")?;
    writeln!(txt, "mean_margin_db={:.4}", result.mean())?;
    writeln!(txt, "p05_margin_db={:.4}", result.percentile(5.0))?;
    writeln!(txt, "p50_margin_db={:.4}", result.percentile(50.0))?;
    writeln!(txt, "p95_margin_db={:.4}", result.percentile(95.0))?;
    writeln!(txt, "prob_meeting_target={:.4}", 1.0 - result.cdf(0.0))?;

    println!("Wrote output/receiver_snr_margin_samples.csv");
    println!("Wrote output/receiver_snr_margin_summary.txt");
    Ok(())
}
