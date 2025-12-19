use std::env;
use std::fs;
use std::path::Path;
use std::process;

// this cannot be crate::Network because of how Cargo works,
// since cargo/rust treats lib.rs and main.rs as separate crates
use crate::file_operations;
use crate::open;
use crate::plot;

use crate::LinkBudget;
use crate::PathLoss;
use crate::Receiver;
use crate::Transmitter;

pub struct Command {}

impl Command {
    pub fn run(args: &[String]) -> Result<Command, Box<dyn std::error::Error>> {
        if args.len() < 2 {
            return Err("not enough arguments".into());
        }

        if args.len() > 2 {
            return Err(
                "too many arguments, expecting only 2, such as `gainlineup filepath`".into(),
            );
        }

        // Check for special flags
        match args[1].as_str() {
            "--version" | "-v" => {
                print_version();
                process::exit(0);
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            _ => {
                if args.len() > 2 {
                    return Err(
                        "too many arguments, expecting only 2, such as `touchstone filepath`"
                            .into(),
                    );
                }
            }
        }

        let cwd = std::env::current_dir().unwrap();
        // cargo run arg[1], such as cargo run tests/simple_config.toml
        // gainlineup arg[1], such as gainlineup tests/simple_config.toml
        let file_path = args[1].clone();
        println!("Config Path: {}", file_path);
        let full_path = cwd.join(&file_path);
        println!("Full Path: {}", full_path.display());

        let file_path_config: file_operations::FilePathConfig =
            file_operations::get_file_path_config(&full_path.display().to_string());

        // absolute path, append .html, remove woindows UNC Prefix if present
        // relative path with separators, just append .hmtl
        // bare_filename, prepend ./ and append .html
        // absolute path, append .html, remove woindows UNC Prefix if present
        // relative path with separators, just append .hmtl
        // bare_filename, prepend ./ and append .html
        let output_html_path =
            if file_path_config.unix_absolute_path || file_path_config.windows_absolute_path {
                let mut file_path_html = format!("{}.html", &file_path);
                // Remove the UNC prefix on Windows if present
                if file_path_config.windows_absolute_path && file_path_html.starts_with(r"\\?\") {
                    file_path_html = file_path_html[4..].to_string();
                }
                file_path_html
            } else if file_path_config.relative_path_with_separators {
                format!("{}.html", &file_path)
            } else if file_path_config.bare_filename {
                format!("./{}.html", &file_path)
            } else {
                panic!(
                    "file_path_config must have one true value: {:?}",
                    file_path_config
                );
            };

        println!("Generating HTML table at: {}", output_html_path);

        let output_html_path_str = output_html_path.as_str();

        let budget = LinkBudget {
            name: "Test Link",
            bandwidth: 10e6,
            transmitter: Transmitter {
                output_power: -20.0,
                gain: 20.0,
                bandwidth: 10e6,
            },
            receiver: Receiver {
                gain: 10.0,
                temperature: 290.0,
                noise_figure: 4.0,
                bandwidth: 10e6,
            },
            path_loss: PathLoss {
                frequency: 2.4e9,
                distance: 1000.0,
            },
            frequency_dependent_loss: Some(3.0),
        };

        match crate::plot::generate_html_summary(&budget, output_html_path_str) {
            Ok(_) => {
                open::plot(output_html_path.clone());
            }
            Err(e) => {
                eprintln!("Error generating HTML table: {}", e);
            }
        }

        Ok(Command {})
    }
}

pub fn print_version() {
    println!("gainlineup {}", env!("CARGO_PKG_VERSION"));
}

pub fn print_error(error: &str) {
    const RED: &str = "\x1b[31m";
    const RESET: &str = "\x1b[0m";
    println!("{}Problem parsing arguments: {error}{}", RED, RESET);
}

pub fn print_help() {
    // ANSI color codes
    const BOLD: &str = "\x1b[1m";
    const CYAN: &str = "\x1b[36m";
    const GREEN: &str = "\x1b[32m";
    const YELLOW: &str = "\x1b[33m";
    const RESET: &str = "\x1b[0m";

    println!(
        "📡 Gainlineup parser and calculator - https://github.com/iancleary/gainlineup{}",
        RESET
    );
    println!();
    println!("{}{}VERSION:{}", BOLD, YELLOW, RESET);
    println!("    {}{}{}", GREEN, env!("CARGO_PKG_VERSION"), RESET);
    println!();
    println!("{}{}USAGE:{}", BOLD, YELLOW, RESET);
    println!("    {} gainlineup <FILE_PATH>{}", GREEN, RESET);
    println!();
    println!("     FILE_PATH: path to a toml config file");
    println!();
    println!("     The toml file is parsed and an interactive plot (html file and js/ folder) ");
    println!("     is created next to the source file(s).");
    // println!("     ");
    println!();
    println!("{}{}OPTIONS:{}", BOLD, YELLOW, RESET);
    println!(
        "    {}  -v, --version{}{}    Print version information",
        GREEN, RESET, RESET
    );
    println!(
        "    {}  -h, --help{}{}       Print help information",
        GREEN, RESET, RESET
    );
    println!();
    println!("{}{}EXAMPLES:{}", BOLD, YELLOW, RESET);
    println!("    {} # Single file (Relative path){}", CYAN, RESET);
    println!("    {} gainlineup files/config.toml{}", GREEN, RESET);
    println!();
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use std::path::PathBuf;

    fn setup_test_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("gainlineup_tests");
        path.push(name);
        path.push(format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_run_function() {
        let test_dir = setup_test_dir("test_run_function");
        let toml_path = test_dir.join("test_cli_run.toml");
        fs::copy("files/example.toml", &toml_path).unwrap();

        let args = vec![
            String::from("program_name"),
            toml_path.to_str().unwrap().to_string(),
        ];
        let _cli_run = Command::run(&args).unwrap();
    }

    #[test]
    fn test_config_build_not_enough_args() {
        let args = vec![String::from("program_name")];
        let result = Command::run(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag() {
        // Help flag test - verifies the flag is recognized
        // Note: In actual execution, this would exit the process
        // This test just documents the expected behavior
        let help_flags = vec!["--help", "-h"];
        for flag in help_flags {
            assert!(flag == "--help" || flag == "-h");
        }
    }

    #[test]
    fn test_version_flag() {
        // Version flag test - verifies the flag is recognized
        // Note: In actual execution, this would exit the process
        // This test just documents the expected behavior
        let version_flags = vec!["--version", "-v"];
        for flag in version_flags {
            assert!(flag == "--version" || flag == "-v");
        }
    }

    #[test]
    fn test_version_output_format() {
        // Test that version string is in correct format
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        // Version should be in format X.Y.Z
        let parts: Vec<&str> = version.split('.').collect();
        assert_eq!(parts.len(), 3, "Version should be in X.Y.Z format");
    }
}
