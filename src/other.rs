use std::env;
use std::io::{self, Write};
use std::process::Command;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};

fn main() {
    #[cfg(target_os = "linux")]
    {
        if env::var("TERM").is_err() {
            // No terminal detected, spawn a new terminal and re-run the program
            let current_exe = env::current_exe().expect("Failed to get current executable path");

            Command::new("gnome-terminal")
                .arg("--")
                .arg(current_exe)
                .spawn()
                .expect("Failed to spawn terminal");

            // Exit the original GUI instance
            std::process::exit(0);
        }
    }

    // Main logic that runs in the terminal
    run_program();
}

fn run_program() {
    let mut stdout = io::stdout();

    stdout.execute(SetForegroundColor(Color::Green)).unwrap();
    stdout.execute(Print("Hello from the cross-platform Rust program!\n")).unwrap();
    stdout.execute(ResetColor).unwrap();

    println!("Running some tasks...");
    println!("Port 3000 opened.");
    println!("External IP: 192.168.1.100");

    println!("\nPress Enter to exit...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}
