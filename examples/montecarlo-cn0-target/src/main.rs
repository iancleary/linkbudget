use montycarlo::{MonteCarloEngine, Simulation};
use rand::Rng;
use std::fs::{create_dir_all, File};
use std::io::Write;

struct Cn0MarginSim { cn0_clear_sky_dbhz: f64, target_cn0_dbhz: f64 }

impl Simulation for Cn0MarginSim {
    type Sample = (f64, f64, f64);
    type Output = f64;

    fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
        (rng.gen_range(0.0..=8.0), rng.gen_range(0.0..=2.0), rng.gen_range(0.0..=1.0))
    }

    fn evaluate(&self, s: &Self::Sample) -> Self::Output {
        (self.cn0_clear_sky_dbhz - s.0 - s.1 - s.2) - self.target_cn0_dbhz
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sim = Cn0MarginSim { cn0_clear_sky_dbhz: 74.0, target_cn0_dbhz: 68.0 };
    let result = MonteCarloEngine::new(sim, 50_000).with_seed(99).run();

    create_dir_all("output")?;
    let mut csv = File::create("output/cn0_margin_samples.csv")?;
    writeln!(csv, "margin_db")?;
    for v in result.sorted_values() { writeln!(csv, "{v:.6}")?; }

    let mut txt = File::create("output/cn0_margin_summary.txt")?;
    writeln!(txt, "trials={}", result.len())?;
    writeln!(txt, "mean_margin_db={:.4}", result.mean())?;
    writeln!(txt, "p05_margin_db={:.4}", result.percentile(5.0))?;
    writeln!(txt, "p50_margin_db={:.4}", result.percentile(50.0))?;
    writeln!(txt, "p95_margin_db={:.4}", result.percentile(95.0))?;
    writeln!(txt, "prob_meeting_target={:.4}", 1.0 - result.cdf(0.0))?;
    Ok(())
}
