use montycarlo::{MonteCarloEngine, Simulation};
use rand::Rng;
use std::fs::{create_dir_all, File};
use std::io::Write;

struct Cn0MarginSim {
    cn0_clear_sky_dbhz: f64,
    target_cn0_dbhz: f64,
}

impl Simulation for Cn0MarginSim {
    type Sample = (f64, f64, f64);
    type Output = f64;

    fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
        let rain_loss = rng.gen_range(0.0..=8.0);
        let pointing_loss = rng.gen_range(0.0..=2.0);
        let implementation_penalty = rng.gen_range(0.0..=1.0);
        (rain_loss, pointing_loss, implementation_penalty)
    }

    fn evaluate(&self, sample: &Self::Sample) -> Self::Output {
        let degraded_cn0 = self.cn0_clear_sky_dbhz - sample.0 - sample.1 - sample.2;
        degraded_cn0 - self.target_cn0_dbhz
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let trials = 50_000;
    let cn0_clear_sky_dbhz = 74.0;
    let target_cn0_dbhz = 68.0;

    let sim = Cn0MarginSim {
        cn0_clear_sky_dbhz,
        target_cn0_dbhz,
    };

    let result = MonteCarloEngine::new(sim, trials).with_seed(99).run();

    create_dir_all("examples/output")?;
    let mut csv = File::create("examples/output/cn0_margin_samples.csv")?;
    writeln!(csv, "margin_db")?;
    for v in result.sorted_values() {
        writeln!(csv, "{v:.6}")?;
    }

    let mut txt = File::create("examples/output/cn0_margin_summary.txt")?;
    writeln!(txt, "trials={}", result.len())?;
    writeln!(txt, "cn0_clear_sky_dbhz={cn0_clear_sky_dbhz}")?;
    writeln!(txt, "target_cn0_dbhz={target_cn0_dbhz}")?;
    writeln!(txt, "mean_margin_db={:.4}", result.mean())?;
    writeln!(txt, "p05_margin_db={:.4}", result.percentile(5.0))?;
    writeln!(txt, "p50_margin_db={:.4}", result.percentile(50.0))?;
    writeln!(txt, "p95_margin_db={:.4}", result.percentile(95.0))?;
    writeln!(txt, "prob_meeting_target={:.4}", 1.0 - result.cdf(0.0))?;

    println!("Wrote examples/output/cn0_margin_samples.csv");
    println!("Wrote examples/output/cn0_margin_summary.txt");
    Ok(())
}
