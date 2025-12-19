use crate::file_operations;
use std::process;

pub fn browser(url: &str) {
    // 1. Determine the OS-specific command and arguments
    let (cmd, args) = if cfg!(target_os = "windows") {
        // Windows: specific syntax to handle spaces and detach process
        // "start" is a shell built-in, so we must invoke "cmd /C start"
        // The empty string "" is the window title (required by start if paths have quotes)
        ("cmd", vec!["/C", "start", "", url])
    } else if cfg!(target_os = "macos") {
        // macOS: The "open" command handles everything
        ("open", vec![url])
    } else {
        // Linux/BSD: "xdg-open" is the Freedesktop standard
        ("xdg-open", vec![url])
    };

    // 2. Spawn the process
    // .spawn() creates the child process and returns immediately.
    // We do NOT use .output() because that would wait for the browser to close.
    match process::Command::new(cmd).args(&args).spawn() {
        Ok(_) => println!("Success! Opening: {}", url),
        Err(e) => eprintln!("Failed to open {} in your default browser: {}", url, e),
    }
}

pub fn plot(file_path: String) {
    // opens plot in browser

    // Note: This does NOT handle space encoding (spaces remain spaces),
    // which most modern browsers can handle, but strictly speaking is invalid URI syntax.
    let html_file_url = file_operations::get_file_url(&file_path);

    println!(
        "You can open the plot in your browser at:\n{}",
        html_file_url
    );

    // if not part of cargo test, open the created file
    if cfg!(test) {
        // pass
    } else {
        println!("Attempting to open plot in your default browser...");
        // 2. Use the open crate to launch the file, if not testing
        browser(&html_file_url);
    }
}

// no automated testing planned
