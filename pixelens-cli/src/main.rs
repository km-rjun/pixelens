use clap:: Parser;

#[derive(Parser, Debug)]
#[clap(name = "pixelens", author = "Your Name", version = "1.0")]
mod cli;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    match args.first().map(String::as_str) {
        None | Some("help") | Some("-h") | Some("--help") => {
            print!("{}", HELP);
            ExitCode::SUCCESS
        },
        Some("version") | Some("-V") | Some("--version") => {
            println!("{} {}", BINARY, VERSION);
            ExitCode::SUCCESS
        },
        Some("grab") | Some("copy") => {
            if !wayland::is_wayland() {
                eprintln!("Error: This feature requires Wayland");
                return ExitCode::from(1);
            }

            // Start slurp for area selection
            let mut child = std::process::Command::new("slurp")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("Failed to start slurp");

            // Wait for user to select area
            let _ = child.stdout
                .as_ref()
                .unwrap()
                .read_to_string(&mut String);

            // Get selected area coordinates
            let coords_str = child.stdout
                .as_ref()
                .unwrap()
                .to_string();
            let coords: Vec<u64> = coords_str
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();

            // Handle cancellation
            if std::env::args().any(|arg| arg == "--cancel") {
                eprintln!("Capture cancelled by user");
                return ExitCode::from(1);
            }

            // Execute grim capture
            let mut grim = std::process::Command::new("grim")
                .arg("--save-to")
                .arg("/tmp/screenshot.png")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("Failed to start grim");

            // Wait for capture to complete
            let _ = grim.wait_with_output()
                .expect("Failed to capture screenshot");

            // Output image path
            println!("Saved screenshot to /tmp/screenshot.png");
            ExitCode::SUCCESS
        },
        // ... rest of match arms
    }
}