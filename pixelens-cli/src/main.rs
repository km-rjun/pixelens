//! `pixelens` — CLI client that talks to the daemon over IPC.
//!
//! Each subcommand connects to the daemon socket, sends a single
//! request, reads the response, and prints a human-readable summary.
//! Subcommands that hit a cancelled / errored response exit non-zero
//! so shell pipelines can detect failure.

use std::path::PathBuf;
use std::process::ExitCode;

use pixelens_config::{get_value, load_config, save_config, set_value, KNOWN_KEYS};
use pixelens_ipc::{
    connect as ipc_connect, read_response, write_frame, Command, FrameError, GrabResponsePayload,
    IpcError, IpcRequest, IpcResponse, IpcStream, ResponseStatus,
};
use thiserror::Error;
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
  hotkey       Manage global hotkey (enable|disable|status)
  autostart    Manage XDG autostart .desktop (enable|disable|status)
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
        Some("hotkey") => match run_hotkey(args.get(1).map(String::as_str)).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        },
        Some("autostart") => match run_autostart(args.get(1).map(String::as_str)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        },
        Some("config") => {
            match run_config(args.get(1).map(String::as_str), args.get(2), args.get(3)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::from(1)
                }
            }
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

/// Resolve the user systemd unit path for the keyhook service.
fn keyhook_unit_path() -> std::path::PathBuf {
    let mut dir = dirs_user_systemd();
    dir.push("pixelens-keyhook.service");
    dir
}

fn dirs_user_systemd() -> std::path::PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return std::path::PathBuf::from(xdg).join("systemd/user");
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(home).join(".config/systemd/user");
    }
    std::path::PathBuf::from(".config/systemd/user")
}

/// Locate the `pixelens-keyhook` binary on PATH.
fn keyhook_binary() -> Option<std::path::PathBuf> {
    if let Ok(out) = std::process::Command::new("which")
        .arg("pixelens-keyhook")
        .output()
    {
        if out.status.success() {
            let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !p.is_empty() {
                return Some(std::path::PathBuf::from(p));
            }
        }
    }
    // Fallbacks
    [
        std::path::PathBuf::from("/usr/bin/pixelens-keyhook"),
        std::path::PathBuf::from(".cargo/bin/pixelens-keyhook"),
    ]
    .into_iter()
    .find(|cand| cand.exists())
}

fn keyhook_combo() -> String {
    if let Ok(env) = std::env::var("PIXELENS_HOTKEY") {
        if !env.is_empty() {
            return env;
        }
    }
    // M8: fall back to the on-disk config's general.hotkey so the CLI
    // reports the same combo the keyhook will actually use.
    pixelens_config::load_config()
        .map(|c| c.general.hotkey)
        .unwrap_or_else(|_| "Super+Shift+T".to_string())
}

fn unit_content(bin: &std::path::Path) -> String {
    format!(
        "[Unit]\n\
         Description=Pixelens global hotkey listener\n\
         After=pixelens.service\n\
         PartOf=pixelens.service\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={bin}\n\
         Restart=on-failure\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy=default.target\n",
        bin = bin.display()
    )
}

fn systemctl(args: &[&str]) -> std::process::Command {
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("--user");
    cmd.args(args);
    cmd
}

async fn run_hotkey(sub: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match sub {
        Some("enable") => {
            let bin = keyhook_binary().ok_or("pixelens-keyhook binary not found on PATH")?;
            let unit = keyhook_unit_path();
            if let Some(parent) = unit.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&unit, unit_content(&bin))?;
            println!("installed unit: {}", unit.display());

            let status = systemctl(&["daemon-reload"]).status()?;
            if !status.success() {
                return Err("systemctl daemon-reload failed".into());
            }
            let status = systemctl(&["enable", "--now", "pixelens-keyhook"]).status()?;
            if !status.success() {
                return Err("systemctl enable --now pixelens-keyhook failed".into());
            }
            println!(
                "hotkey enabled (combo: {}). Press it to grab.",
                keyhook_combo()
            );
            Ok(())
        }
        Some("disable") => {
            let _ = systemctl(&["disable", "--now", "pixelens-keyhook"]).status();
            let unit = keyhook_unit_path();
            if unit.exists() {
                std::fs::remove_file(&unit).ok();
            }
            println!("hotkey disabled");
            Ok(())
        }
        Some("status") => {
            let out = systemctl(&["is-active", "pixelens-keyhook"]).output()?;
            let active = out.status.success();
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!(
                "hotkey service: {}",
                if active { &state } else { "inactive" }
            );
            println!("combo: {}", keyhook_combo());
            println!(
                "daemon: {}",
                if socket_path().map(|p| p.exists()).unwrap_or(false) {
                    "up"
                } else {
                    "down"
                }
            );
            Ok(())
        }
        other => {
            eprintln!("usage: pixelens hotkey <enable|disable|status>");
            if let Some(cmd) = other {
                return Err(format!("unknown hotkey subcommand '{cmd}'").into());
            }
            Ok(())
        }
    }
}

/// XDG autostart directory (`~/.config/autostart`), honouring
/// `XDG_CONFIG_HOME`. Falls back to `$HOME/.config/autostart`.
fn autostart_dir() -> std::path::PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return std::path::PathBuf::from(xdg).join("autostart");
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(home).join(".config/autostart");
    }
    std::path::PathBuf::from(".config/autostart")
}

/// Path of the generated autostart `.desktop` file.
fn autostart_desktop_path() -> std::path::PathBuf {
    autostart_dir().join("pixelens.desktop")
}

/// Build the XDG autostart `.desktop` file content for `pixelens-keyhook`.
fn autostart_desktop_content(bin: &std::path::Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Pixelens\nExec={bin}\nComment=Start Pixelens global hotkey listener on login\nX-GNOME-Autostart-enabled=true\n",
        bin = bin.display()
    )
}

/// Pure helper: write the autostart `.desktop` into `dir` using `bin`.
/// Returns the written file path. Creates the parent directory.
fn write_autostart_desktop(dir: &std::path::Path, bin: &std::path::Path) -> std::path::PathBuf {
    std::fs::create_dir_all(dir).ok();
    let path = dir.join("pixelens.desktop");
    std::fs::write(&path, autostart_desktop_content(bin)).ok();
    path
}

/// Pure helper: remove the autostart `.desktop` from `dir` if present.
/// Ignores a missing file (idempotent).
fn remove_autostart_desktop(dir: &std::path::Path) {
    let path = dir.join("pixelens.desktop");
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }
}

/// `pixelens autostart <enable|disable|status>` — XDG autostart
/// complement to the UM1 systemd `--user` hotkey service. The `.desktop`
/// file (not systemd) is the mechanism that survives login on DEs that
/// honour XDG autostart; no `systemctl` is invoked here.
fn run_autostart(sub: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match sub {
        Some("enable") => {
            let bin = keyhook_binary().ok_or("pixelens-keyhook binary not found on PATH")?;
            let dir = autostart_dir();
            let path = write_autostart_desktop(&dir, &bin);
            println!("autostart enabled: {}", path.display());
            Ok(())
        }
        Some("disable") => {
            remove_autostart_desktop(&autostart_dir());
            println!("autostart disabled");
            Ok(())
        }
        Some("status") => {
            let path = autostart_desktop_path();
            if path.exists() {
                println!("autostart: enabled ({})", path.display());
            } else {
                println!("autostart: disabled");
            }
            Ok(())
        }
        other => {
            eprintln!("usage: pixelens autostart <enable|disable|status>");
            if let Some(cmd) = other {
                return Err(format!("unknown autostart subcommand '{cmd}'").into());
            }
            Ok(())
        }
    }
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
        Ok(PathBuf::from(format!("/tmp/pixelens-{uid}.sock")))
    }

    #[cfg(not(unix))]
    Err(CliError::DaemonNotRunning(PathBuf::from(
        "(no socket path on non-unix)",
    )))
}

async fn connect() -> Result<IpcStream, CliError> {
    match ipc_connect().await {
        Ok(s) => Ok(s),
        Err(e)
            if e.to_string().contains("NotFound")
                || e.to_string().contains("ConnectionRefused")
                || e.to_string().contains("BrokenPipe") =>
        {
            Err(CliError::DaemonNotRunning(socket_path().unwrap_or_else(|_| PathBuf::from("(unknown)"))))
        }
        Err(e) => Err(CliError::Frame(e)),
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
    let pretty = serde_json::to_string_pretty(&response.payload).map_err(IpcError::Json)?;
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

/// `pixelens config <list|get <key>|set <key> <value>>`.
///
/// Operates on the on-disk TOML config directly (M8). Machine-friendly
/// output: `list` prints `key = value` lines, `get` prints just the
/// value, `set` reports success on stderr-free stdout.
fn run_config(
    sub: Option<&str>,
    arg2: Option<&String>,
    arg3: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match sub {
        Some("list") => {
            let cfg = load_config()?;
            for key in KNOWN_KEYS {
                println!("{key} = {}", get_value(&cfg, key)?);
            }
            Ok(())
        }
        Some("get") => {
            let key = arg2.ok_or("usage: pixelens config get <key>")?;
            let cfg = load_config()?;
            println!("{}", get_value(&cfg, key)?);
            Ok(())
        }
        Some("set") => {
            let key = arg2.ok_or("usage: pixelens config set <key> <value>")?;
            let value = arg3.ok_or("usage: pixelens config set <key> <value>")?;
            let mut cfg = load_config()?;
            set_value(&mut cfg, key, value)?;
            save_config(&cfg)?;
            println!("set {key} = {value}");
            // UM3: `general.autostart` has a real side effect — keep the
            // XDG autostart `.desktop` in sync with the config key. This is
            // best-effort: a write failure warns but does not fail `set`.
            if key == "general.autostart" {
                match value.trim().to_ascii_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => {
                        if let Some(bin) = keyhook_binary() {
                            let dir = autostart_dir();
                            write_autostart_desktop(&dir, &bin);
                            println!(
                                "wrote autostart entry: {}",
                                autostart_desktop_path().display()
                            );
                        } else {
                            eprintln!(
                                "warning: pixelens-keyhook not found on PATH; autostart not written"
                            );
                        }
                    }
                    _ => {
                        remove_autostart_desktop(&autostart_dir());
                        println!("removed autostart entry (if present)");
                    }
                }
            }
            Ok(())
        }
        Some(other) => {
            Err(format!("unknown config subcommand '{other}' (expected list|get|set)").into())
        }
        None => {
            eprintln!("usage: pixelens config <list|get <key>|set <key> <value>>");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autostart_desktop_round_trip() {
        // Isolate from the real $HOME / XDG_CONFIG_HOME.
        let tmp = std::env::temp_dir().join(format!(
            "pixelens-autostart-test-{}-{}",
            std::process::id(),
            "roundtrip"
        ));
        let dir = tmp.join("config").join("autostart");
        let _ = std::fs::remove_dir_all(&tmp);

        let bin = PathBuf::from("/usr/bin/pixelens-keyhook");

        // Initially nothing is written.
        assert!(!dir.join("pixelens.desktop").exists());

        // enable -> desktop file appears with the right content.
        let path = write_autostart_desktop(&dir, &bin);
        assert!(path.exists(), "desktop file should exist after write");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[Desktop Entry]"));
        assert!(content.contains("Type=Application"));
        assert!(content.contains("Name=Pixelens"));
        assert!(content.contains("Exec=/usr/bin/pixelens-keyhook"));
        assert!(content.contains("X-GNOME-Autostart-enabled=true"));

        // disable -> desktop file removed (idempotent even if absent).
        remove_autostart_desktop(&dir);
        assert!(!dir.join("pixelens.desktop").exists());
        remove_autostart_desktop(&dir); // second call must not panic

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn autostart_dir_honors_xdg_config_home() {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-test");
        std::env::remove_var("HOME");
        let dir = autostart_dir();
        assert_eq!(dir, PathBuf::from("/tmp/xdg-test/autostart"));
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
