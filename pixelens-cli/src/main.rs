//! `pixelens` — thin CLI client that talks to the daemon over a Unix
//! domain socket. M1 implements argument parsing and the help text from
//! PRD §"CLI Specification" verbatim; M6 will add the IPC transport and
//! dispatch; M8 will add `pixelens config` subcommands.

use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BINARY: &str = "pixelens";

const HELP: &str = "\
Pixelens - Linux-native visual text extraction

Usage:
  pixelens <command>

Commands:
  grab         Select an area and copy text to clipboard
  copy         Alias for grab

  daemon       Start background daemon
  status       Show daemon status
  stop         Stop daemon
  config       Manage configuration

  version      Show version
  help         Show help
";

/// Commands reserved for future versions; parsed but rejected with a
/// clear message (PRD §"CLI Specification").
const RESERVED_COMMANDS: &[&str] = &["search", "ai", "translate", "image"];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        None | Some("help") | Some("-h") | Some("--help") => {
            print!("{}", HELP);
            ExitCode::SUCCESS
        }
        Some("version") | Some("-V") | Some("--version") => {
            println!("{} {}", BINARY, VERSION);
            ExitCode::SUCCESS
        }
        Some("grab") | Some("copy") => {
            println!("'{}' dispatched to daemon (stub — wired in M6)", args[0]);
            ExitCode::SUCCESS
        }
        Some("daemon") => {
            println!("daemon mode is started by running `pixelensd` directly");
            ExitCode::SUCCESS
        }
        Some("status") | Some("stop") => {
            println!("'{}' command (stub — wired in M6)", args[0]);
            ExitCode::SUCCESS
        }
        Some("config") => {
            println!("'config' command (stub — wired in M8)");
            ExitCode::SUCCESS
        }
        Some(cmd) if RESERVED_COMMANDS.contains(&cmd) => {
            eprintln!(
                "error: '{}' is reserved for a future release and is not available in v1.0",
                cmd
            );
            ExitCode::from(2)
        }
        Some(other) => {
            eprintln!("error: unknown command '{}'\n", other);
            eprint!("{}", HELP);
            ExitCode::from(2)
        }
    }
}
