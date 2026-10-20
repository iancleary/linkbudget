use linkbudget::{ber, energy, Modulation, Receiver, ReceiverNoise};
use montycarlo::{MonteCarloEngine, Simulation};
use rand::Rng;
use std::fs::{create_dir_all, File};
use std::io::{Error, Write};

const TRIALS: usize = 50_000;
const SEED: u64 = 99;
const INFORMATION_RATE_BPS: f64 = 10e6;
const TARGET_BER: f64 = 1e-6;
const TWO_SIDED_95_Z: f64 = 1.959963984540054;

struct ReceiverSample {
    source_temperature_k: f64,
    noise_figure_db: f64,
    noise_bandwidth_hz: f64,
    input_power_dbm: f64,
}

struct ReceiverMarginSim {
    required_eb_no_db: f64,
}

impl Simulation for ReceiverMarginSim {
    type Sample = ReceiverSample;
    type Output = f64;

    fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
        ReceiverSample {
            source_temperature_k: rng.gen_range(250.0..=300.0),
            noise_figure_db: rng.gen_range(1.2..=3.5),
            noise_bandwidth_hz: rng.gen_range(5.0e6..=25.0e6),
            input_power_dbm: rng.gen_range(-92.0..=-82.0),
        }
    }

    fn evaluate(&self, sample: &Self::Sample) -> Self::Output {
        let rx = Receiver {
            gain: 42.0,
            noise: ReceiverNoise::SourceAndNoiseFigure {
                source_temperature_k: sample.source_temperature_k,
                noise_figure_db: sample.noise_figure_db,
            },
            bandwidth: sample.noise_bandwidth_hz,
        };
        let snr_db = rx.calculate_snr(sample.input_power_dbm);
        let c_no_db_hz = energy::snr_to_c_over_no(snr_db, sample.noise_bandwidth_hz);
        let eb_no_db = energy::c_over_no_to_eb_over_no(c_no_db_hz, INFORMATION_RATE_BPS);
        eb_no_db - self.required_eb_no_db
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let required_eb_no_db = ber::required_eb_no_db(TARGET_BER, &Modulation::Qpsk)
        .ok_or_else(|| Error::other("QPSK BER threshold could not be calculated"))?;
    let sim = ReceiverMarginSim { required_eb_no_db };
    let result = MonteCarloEngine::new(sim, TRIALS).with_seed(SEED).run();
    let failures = result.sorted_values().iter().filter(|&&v| v < 0.0).count();
    let n = result.len() as f64;
    let failure_probability = failures as f64 / n;
    let z2 = TWO_SIDED_95_Z * TWO_SIDED_95_Z;
    let denominator = 1.0 + z2 / n;
    let center = (failure_probability + z2 / (2.0 * n)) / denominator;
    let half_width = TWO_SIDED_95_Z
        * (failure_probability * (1.0 - failure_probability) / n + z2 / (4.0 * n * n)).sqrt()
        / denominator;

    create_dir_all("output")?;
    let mut csv = File::create("output/receiver_ebno_margin_samples.csv")?;
    writeln!(csv, "eb_no_margin_db")?;
    for v in result.sorted_values() {
        writeln!(csv, "{v:.17}")?;
    }

    let mut txt = File::create("output/receiver_ebno_margin_summary.txt")?;
    writeln!(txt, "trials={}", result.len())?;
    writeln!(txt, "seed={SEED}")?;
    writeln!(txt, "metric=actual Eb/N0 minus required Eb/N0 (dB)")?;
    writeln!(txt, "modulation=uncoded coherent Gray QPSK in AWGN")?;
    writeln!(txt, "target_ber={TARGET_BER}")?;
    writeln!(txt, "information_rate_bps={INFORMATION_RATE_BPS}")?;
    writeln!(txt, "required_eb_no_db={required_eb_no_db:.17}")?;
    writeln!(txt, "source_temperature_k=Uniform[250,300]")?;
    writeln!(txt, "noise_figure_db=Uniform[1.2,3.5] at 290 K reference")?;
    writeln!(txt, "noise_bandwidth_hz=Uniform[5000000,25000000]")?;
    writeln!(txt, "input_power_dbm=Uniform[-92,-82]")?;
    writeln!(
        txt,
        "input_assumptions=independent illustrative uniforms; not calibrated climate"
    )?;
    writeln!(txt, "physical_assumptions=flat thermal noise; all signal captured; common receiver input plane; no implementation loss or FEC")?;
    writeln!(txt, "quantile_method=linear interpolation at rank p*(N-1)")?;
    writeln!(txt, "mean_margin_db={:.17}", result.mean())?;
    for p in [5, 50, 95] {
        writeln!(txt, "p{p:02}_margin_db={:.17}", result.percentile(p as f64))?;
    }
    writeln!(
        txt,
        "failure_definition=eb_no_margin_db < 0 (zero meets target)"
    )?;
    writeln!(txt, "failures={failures}")?;
    writeln!(txt, "failure_probability={failure_probability:.17}")?;
    writeln!(
        txt,
        "failure_wilson_two_sided_95_lower={:.17}",
        (center - half_width).max(0.0)
    )?;
    writeln!(
        txt,
        "failure_wilson_two_sided_95_upper={:.17}",
        (center + half_width).min(1.0)
    )?;
    writeln!(txt, "wilson_z={TWO_SIDED_95_Z}")?;
    if failures == 0 {
        let upper = -(0.05_f64.ln() / n).exp_m1();
        writeln!(txt, "zero_failure_exact_one_sided_95_upper={upper:.17}")?;
    }
    writeln!(txt, "interval_meaning=binomial Monte Carlo sampling uncertainty conditional on the input model; one-sided exact bound is distinct from two-sided Wilson interval")?;
    writeln!(txt, "limits=fixed bounded trial count; no adaptive convergence guarantee; no field or annual availability claim; input uncertainty and model discrepancy not covered by binomial interval")?;

    println!("Wrote output/receiver_ebno_margin_samples.csv");
    println!("Wrote output/receiver_ebno_margin_summary.txt");
    Ok(())
}
