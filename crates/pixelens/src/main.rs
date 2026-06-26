use clap::{Parser, Subcommand};
use std::process;

use pixelens_core::config::Config;
use pixelens_core::ipc::client::IpcClient;
use pixelens_core::ipc::protocol::Request;
use pixelens_core::types::ActionType;

#[derive(Parser)]
#[command(name = "pixelens")]
#[command(about = "Linux-native visual search and OCR utility")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Select a region, OCR it, and print the extracted text
    Grab,

    /// Select a region, OCR it, copy text to clipboard
    Copy,

    /// Select a region, OCR it, search the web for the text
    Search,

    /// Select a region, OCR it, send to AI
    Ai {
        /// Optional prompt or question about the selection
        #[arg(long)]
        prompt: Option<String>,
    },

    /// Select a region, OCR it, translate the text
    Translate {
        /// Target language (default: English)
        #[arg(long, default_value = "English")]
        to: String,
    },

    /// Select a region, perform reverse image search
    Image,

    /// Manage the pixelensd daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Show or set configuration
    Config {
        /// Set API endpoint
        #[arg(long)]
        endpoint: Option<String>,

        /// Set model
        #[arg(long)]
        model: Option<String>,

        /// Set OCR language
        #[arg(long)]
        language: Option<String>,

        /// Set hotkey
        #[arg(long)]
        hotkey: Option<String>,
    },

    /// Show version information
    Version,
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon process
    Start,
    /// Check daemon status
    Status,
    /// Stop the daemon gracefully
    Stop,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Grab => cmd_grab().await,
        Commands::Copy => cmd_copy().await,
        Commands::Search => cmd_search().await,
        Commands::Ai { prompt } => cmd_ai(prompt.as_deref()).await,
        Commands::Translate { to } => cmd_translate(&to).await,
        Commands::Image => cmd_image().await,
        Commands::Daemon { action } => match action {
            DaemonAction::Start => cmd_daemon_start(),
            DaemonAction::Status => cmd_daemon_status().await,
            DaemonAction::Stop => cmd_daemon_stop().await,
        },
        Commands::Config {
            endpoint,
            model,
            language,
            hotkey,
        } => cmd_config(endpoint, model, language, hotkey),
        Commands::Version => {
            println!("pixelens {}", env!("CARGO_PKG_VERSION"));
            0
        }
    };

    process::exit(exit_code);
}

async fn do_grab() -> Result<(String, Option<String>), i32> {
    let client = IpcClient::new();
    match client.grab(false, None).await {
        Ok((_image_path, text, _ai_response)) => {
            if let Some(t) = &text {
                Ok((t.clone(), Some(t.clone())))
            } else {
                eprintln!("No text extracted from capture");
                Err(1)
            }
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            eprintln!("Daemon not running. Start with: pixelens daemon start");
            Err(1)
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("cancelled") {
                eprintln!("Capture cancelled");
            } else {
                eprintln!("Capture failed: {}", msg);
            }
            Err(1)
        }
    }
}

async fn cmd_grab() -> i32 {
    match do_grab().await {
        Ok((text, _)) => {
            println!("{}", text);
            0
        }
        Err(code) => code,
    }
}

async fn cmd_copy() -> i32 {
    let (text, _) = match do_grab().await {
        Ok(v) => v,
        Err(code) => return code,
    };

    let client = IpcClient::new();
    match client
        .action(ActionType::CopyToClipboard, &text, None)
        .await
    {
        Ok(_) => {
            eprintln!("Copied to clipboard");
            0
        }
        Err(e) => {
            eprintln!("Copy failed: {}", e);
            1
        }
    }
}

async fn cmd_search() -> i32 {
    let (text, _) = match do_grab().await {
        Ok(v) => v,
        Err(code) => return code,
    };

    let client = IpcClient::new();
    match client.action(ActionType::SearchWeb, &text, None).await {
        Ok(url) => {
            println!("{}", url);
            0
        }
        Err(e) => {
            eprintln!("Search failed: {}", e);
            1
        }
    }
}

async fn cmd_ai(prompt: Option<&str>) -> i32 {
    let (ocr_text, _) = match do_grab().await {
        Ok(v) => v,
        Err(code) => return code,
    };

    let final_prompt = match prompt {
        Some(p) => format!("{}\n\nText from screen:\n{}", p, ocr_text),
        None => format!("Describe or explain the following text:\n\n{}", ocr_text),
    };

    let client = IpcClient::new();
    match client
        .send(Request::Ai {
            prompt: final_prompt,
            image_path: None,
        })
        .await
    {
        Ok(pixelens_core::ipc::protocol::Response::AiResult { content, .. }) => {
            println!("{}", content);
            0
        }
        Ok(pixelens_core::ipc::protocol::Response::Error(e)) => {
            eprintln!("AI error: {}", e);
            1
        }
        Ok(_) => {
            eprintln!("Unexpected response from daemon");
            1
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            eprintln!("Daemon not running. Start with: pixelens daemon start");
            1
        }
        Err(e) => {
            eprintln!("AI request failed: {}", e);
            1
        }
    }
}

async fn cmd_translate(to: &str) -> i32 {
    let (ocr_text, _) = match do_grab().await {
        Ok(v) => v,
        Err(code) => return code,
    };

    let prompt = format!(
        "Translate the following text to {}. Return only the translation:\n\n{}",
        to, ocr_text
    );

    let client = IpcClient::new();
    match client
        .send(Request::Ai {
            prompt,
            image_path: None,
        })
        .await
    {
        Ok(pixelens_core::ipc::protocol::Response::AiResult { content, .. }) => {
            println!("{}", content);
            0
        }
        Ok(pixelens_core::ipc::protocol::Response::Error(e)) => {
            eprintln!("Translation error: {}", e);
            1
        }
        Ok(_) => {
            eprintln!("Unexpected response from daemon");
            1
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            eprintln!("Daemon not running. Start with: pixelens daemon start");
            1
        }
        Err(e) => {
            eprintln!("Translation failed: {}", e);
            1
        }
    }
}

async fn cmd_image() -> i32 {
    let client = IpcClient::new();
    match client.grab(false, None).await {
        Ok((image_path, _text, _ai)) => {
            match client
                .action(ActionType::ReverseImageSearch, "", Some(&image_path))
                .await
            {
                Ok(url) => {
                    println!("{}", url);
                    0
                }
                Err(e) => {
                    eprintln!("Image search failed: {}", e);
                    1
                }
            }
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            eprintln!("Daemon not running. Start with: pixelens daemon start");
            1
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("cancelled") {
                eprintln!("Capture cancelled");
            } else {
                eprintln!("Capture failed: {}", msg);
            }
            1
        }
    }
}

fn cmd_daemon_start() -> i32 {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("pixelensd")).or(Some(p)));

    let exe_path = match exe {
        Some(p) => p,
        None => {
            eprintln!("Could not determine pixelensd path");
            return 1;
        }
    };

    if !exe_path.exists() {
        eprintln!("pixelensd not found at: {}", exe_path.display());
        eprintln!("Build it with: cargo build --release -p pixelensd");
        return 1;
    }

    match process::Command::new(&exe_path).spawn() {
        Ok(child) => {
            eprintln!("pixelensd started (pid: {})", child.id());
            0
        }
        Err(e) => {
            eprintln!("Failed to start pixelensd: {}", e);
            1
        }
    }
}

async fn cmd_daemon_status() -> i32 {
    let client = IpcClient::new();
    match client.status().await {
        Ok((running, capture_missing, ocr_missing)) => {
            if running {
                println!("Daemon: running");
            } else {
                println!("Daemon: stopped");
            }
            if !capture_missing.is_empty() {
                eprintln!("Missing capture tools: {}", capture_missing.join(", "));
            }
            if !ocr_missing.is_empty() {
                eprintln!("Missing OCR tools: {}", ocr_missing.join(", "));
            }
            0
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            println!("Daemon: stopped");
            0
        }
        Err(e) => {
            eprintln!("Status check failed: {}", e);
            1
        }
    }
}

async fn cmd_daemon_stop() -> i32 {
    let client = IpcClient::new();
    match client.stop().await {
        Ok(()) => {
            println!("Daemon stopped");
            0
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            println!("Daemon was not running");
            0
        }
        Err(e) => {
            eprintln!("Stop failed: {}", e);
            1
        }
    }
}

fn cmd_config(
    endpoint: Option<String>,
    model: Option<String>,
    language: Option<String>,
    hotkey: Option<String>,
) -> i32 {
    let mut config = Config::load();
    let mut changed = false;

    if let Some(e) = endpoint {
        config.api_endpoint = e;
        changed = true;
    }
    if let Some(m) = model {
        config.model = m;
        changed = true;
    }
    if let Some(l) = language {
        config.ocr_language = l;
        changed = true;
    }
    if let Some(h) = hotkey {
        config.hotkey = h;
        changed = true;
    }

    if changed {
        match config.save() {
            Ok(()) => {
                println!("Configuration saved");
                0
            }
            Err(e) => {
                eprintln!("Failed to save config: {}", e);
                1
            }
        }
    } else {
        println!("Endpoint: {}", config.api_endpoint);
        println!("Model: {}", config.model);
        println!("OCR Language: {}", config.ocr_language);
        println!("Hotkey: {}", config.hotkey);
        if !config.api_key.is_empty() {
            println!("API Key: ****");
        }
        0
    }
}
