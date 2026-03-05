//! Utilities for opening files in the default browser.

use crate::file_operations;
use std::process;

fn browser_command(url: &str) -> (&'static str, Vec<String>) {
    if cfg!(target_os = "windows") {
        // "start" is a shell built-in, so we must invoke "cmd /C start"
        ("cmd", vec!["/C", "start", "", url].into_iter().map(String::from).collect())
    } else if cfg!(target_os = "macos") {
        ("open", vec![url.to_string()])
    } else {
        ("xdg-open", vec![url.to_string()])
    }
}

/// Open a URL in the platform's default browser.
pub fn browser(url: &str) {
    let (cmd, args) = browser_command(url);

    match process::Command::new(cmd).args(&args).spawn() {
        Ok(_) => tracing::info!("Opening in browser: {}", url),
        Err(e) => tracing::error!("Failed to open {} in default browser: {}", url, e),
    }
}

/// Open a generated plot HTML file in the browser.
pub fn plot(file_path: String) {
    let html_file_url = file_operations::get_file_url(&file_path);

    tracing::info!("Plot available at: {}", html_file_url);

    // During tests, skip launching the browser process.
    if !cfg!(test) {
        tracing::debug!("Attempting to open plot in default browser...");
        browser(&html_file_url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_command_uses_expected_program_for_platform() {
        let (cmd, args) = browser_command("https://example.com");

        if cfg!(target_os = "windows") {
            assert_eq!(cmd, "cmd");
            assert_eq!(args, vec!["/C", "start", "", "https://example.com"]);
        } else if cfg!(target_os = "macos") {
            assert_eq!(cmd, "open");
            assert_eq!(args, vec!["https://example.com"]);
        } else {
            assert_eq!(cmd, "xdg-open");
            assert_eq!(args, vec!["https://example.com"]);
        }
    }

    #[test]
    fn browser_command_preserves_spaces_in_urls() {
        let url = "file:///tmp/plot with spaces.html";
        let (_cmd, args) = browser_command(url);
        assert_eq!(args.last().map(String::as_str), Some(url));
    }

    #[test]
    fn plot_handles_relative_path_without_panicking() {
        plot("files/example.toml.html".to_string());
    }

    #[test]
    fn plot_handles_absolute_path_with_spaces_without_panicking() {
        let mut temp_path = std::env::temp_dir();
        temp_path.push("linkbudget open plot test.html");
        std::fs::write(&temp_path, "<html></html>").unwrap();

        plot(temp_path.to_string_lossy().to_string());

        let _ = std::fs::remove_file(&temp_path);
    }
}
