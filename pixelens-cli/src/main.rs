//! `pixelens` — CLI client that talks to the daemon over IPC.
//!
//! Each subcommand connects to the daemon socket, sends a single
//! request, reads the response, and prints a human-readable summary.
//! Subcommands that hit a cancelled / errored response exit non-zero
//! so shell pipelines can detect failure.

use std::path::PathBuf;
use std::process::ExitCode;

use pixelens_ipc::{
    read_response, write_frame, Command, FrameError, GrabResponsePayload, IpcRequest,
    IpcResponse, IpcError, ResponseStatus,
};
use thiserror::Error;
use tokio::net::UnixStream;
use uuid::Uuid;

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

#[derive(Debug, Error)]
enum CliError {
    #[error("daemon is not running. Start it with: pixelensd (checked socket: {0})")]
    DaemonNotRunning(PathBuf),

    #[error("ipc error: {0}")]
    Ipc(#[from] IpcError),

    #[error("frame error: {0}")]
    Frame(#[from] FrameError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[tokio::main(flavor = "current_thread")]
async fn real_main() -> ExitCode {
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
        Some("grab") | Some("copy") => match run_grab().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(CliError::DaemonNotRunning(p)) => {
                eprintln!("error: daemon is not running. Start it with: pixelensd");
                eprintln!("  (expected socket: {})", p.display());
                ExitCode::from(1)
            }
            Err(CliError::Ipc(IpcError::Io(_))) | Err(CliError::Io(_)) => {
                let p = socket_path().unwrap_or_else(|_| PathBuf::from("(unknown)"));
                eprintln!("error: daemon is not running. Start it with: pixelensd");
                eprintln!("  (expected socket: {})", p.display());
                ExitCode::from(1)
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        },
        Some("daemon") => {
            println!("daemon mode is started by running `pixelensd` directly");
            ExitCode::SUCCESS
        }
        Some("status") => match run_status().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        },
        Some("stop") => match run_stop().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        },
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

fn main() -> ExitCode {
    real_main()
}

/// Resolve the daemon socket path. Mirrors the daemon's `socket_path()`
/// exactly — if the daemon can find the socket, so can the CLI.
fn socket_path() -> Result<PathBuf, CliError> {
    if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir).join("pixelens.sock"));
        }
    }

    #[cfg(unix)]
    {
        extern "C" {
            fn getuid() -> u32;
        }
        // SAFETY: getuid is async-signal-safe and has no preconditions.
        let uid = unsafe { getuid() };
        return Ok(PathBuf::from(format!("/tmp/pixelens-{uid}.sock")));
    }

    #[cfg(not(unix))]
    Err(CliError::DaemonNotRunning(PathBuf::from("(no socket path on non-unix)")))
}

async fn connect() -> Result<UnixStream, CliError> {
    let path = socket_path()?;
    match UnixStream::connect(&path).await {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound
                 || e.kind() == std::io::ErrorKind::ConnectionRefused =>
        {
            Err(CliError::DaemonNotRunning(path))
        }
        Err(e) => Err(CliError::Io(e)),
    }
}

async fn send_request(command: Command) -> Result<IpcResponse, CliError> {
    let mut stream = connect().await?;
    let request = IpcRequest {
        request_id: Uuid::new_v4().to_string(),
        command,
        payload: serde_json::json!({}),
    };
    write_frame(&request, &mut stream).await?;
    let response = read_response(&mut stream).await?;
    Ok(response)
}

async fn run_grab() -> Result<(), CliError> {
    eprintln!("select an area with the cursor; press Escape to cancel");
    let response = send_request(Command::Grab).await?;
    match response.status {
        ResponseStatus::Ok => {
            let payload: GrabResponsePayload =
                serde_json::from_value(response.payload).map_err(IpcError::Json)?;
            println!("{}", payload.path);
            eprintln!(
                "captured {}x{} region at ({}, {}), {} bytes -> {}",
                payload.region.width,
                payload.region.height,
                payload.region.x,
                payload.region.y,
                payload.bytes,
                payload.path
            );
            Ok(())
        }
        ResponseStatus::Cancelled => {
            eprintln!("cancelled");
            // Exit 0 for a user-initiated cancel — the workflow was
            // completed, just not by capturing anything.
            Ok(())
        }
        ResponseStatus::Error => {
            let msg = response
                .payload
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("(no error message)");
            eprintln!("error: {msg}");
            // Distinct exit code for daemon-side errors so scripts
            // can tell "user cancelled" (0) from "something failed" (1).
            std::process::exit(1);
        }
    }
}

async fn run_status() -> Result<(), CliError> {
    let response = send_request(Command::Status).await?;
    if response.status != ResponseStatus::Ok {
        let msg = response
            .payload
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("(no error message)");
        eprintln!("error: {msg}");
        std::process::exit(1);
    }
    let pretty = serde_json::to_string_pretty(&response.payload)
        .map_err(IpcError::Json)?;
    println!("{pretty}");
    Ok(())
}

async fn run_stop() -> Result<(), CliError> {
    let response = send_request(Command::Stop).await?;
    if response.status != ResponseStatus::Ok {
        let msg = response
            .payload
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("(no error message)");
        eprintln!("error: {msg}");
        std::process::exit(1);
    }
    println!("stop signal sent");
    Ok(())
}
