use std::env;
use std::process;

use linkbudget::cli;

fn main() {
    // Initialize tracing subscriber (controlled via RUST_LOG env var)
    // e.g. RUST_LOG=linkbudget=debug linkbudget config.toml
    #[cfg(feature = "cli")]
    {
        use tracing_subscriber::EnvFilter;
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    let args: Vec<String> = env::args().collect();

    let _ = cli::Command::run(&args).unwrap_or_else(|err| {
        println!();
        cli::print_error(&err.to_string()); //print at the top, but might be lost or hard to read
        println!();
        cli::print_help();
        println!();
        cli::print_error(&err.to_string()); // print error again, for human factors
        process::exit(1);
    });
}
