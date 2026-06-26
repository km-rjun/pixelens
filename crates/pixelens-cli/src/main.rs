use clap::{Parser, Subcommand};
use std::str::FromStr;

use pixelens_actions::get_action_names;
use pixelens_common::ActionType;
use pixelens_config::Config;
use pixelens_ipc::client::IpcClient;
use pixelens_ipc::protocol::{Request, Response};

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
    /// Capture a region of the screen
    Capture,

    /// Perform OCR on an image
    Ocr {
        /// Path to the image file
        #[arg(short, long)]
        image: String,

        /// Language for OCR (default: eng)
        #[arg(short, long, default_value = "eng")]
        language: String,
    },

    /// Ask AI about an image or text
    Ai {
        /// Prompt to send to AI
        #[arg(short, long)]
        prompt: String,

        /// Optional image path
        #[arg(short, long)]
        image: Option<String>,
    },

    /// Execute an action (copy, search, reverse_image, translate)
    Action {
        /// Action to execute
        #[arg(short, long)]
        name: String,

        /// Text for the action
        #[arg(short, long)]
        text: String,

        /// Optional image path
        #[arg(short, long)]
        image: Option<String>,
    },

    /// List available actions
    Actions,

    /// Check if required tools are installed
    Check,

    /// Show current configuration
    Config,

    /// Save configuration
    SaveConfig {
        /// API endpoint
        #[arg(short, long)]
        endpoint: Option<String>,

        /// API key
        #[arg(short, long)]
        key: Option<String>,

        /// Model name
        #[arg(short, long)]
        model: Option<String>,

        /// OCR language
        #[arg(short, long)]
        language: Option<String>,
    },

    /// Start the daemon
    Daemon,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Capture => cmd_capture().await,
        Commands::Ocr { image, language } => cmd_ocr(&image, &language).await,
        Commands::Ai { prompt, image } => cmd_ai(&prompt, image.as_deref()).await,
        Commands::Action { name, text, image } => cmd_action(&name, &text, image.as_deref()).await,
        Commands::Actions => cmd_actions(),
        Commands::Check => cmd_check().await,
        Commands::Config => cmd_config(),
        Commands::SaveConfig {
            endpoint,
            key,
            model,
            language,
        } => {
            cmd_save_config(endpoint, key, model, language);
        }
        Commands::Daemon => cmd_daemon().await,
    }
}

async fn cmd_capture() {
    let client = IpcClient::new();
    match client.send(Request::Capture).await {
        Ok(Response::CaptureResult { image_path, .. }) => println!("{}", image_path),
        Ok(Response::Error(e)) => eprintln!("Capture failed: {}", e),
        Ok(_) => eprintln!("Unexpected response"),
        Err(e) => eprintln!("IPC error: {}", e),
    }
}

async fn cmd_ocr(image: &str, language: &str) {
    let client = IpcClient::new();
    match client
        .send(Request::Ocr {
            image_path: image.to_string(),
            language: language.to_string(),
        })
        .await
    {
        Ok(Response::OcrResult(result)) => print!("{}", result.text),
        Ok(Response::Error(e)) => eprintln!("OCR failed: {}", e),
        Ok(_) => eprintln!("Unexpected response"),
        Err(e) => eprintln!("IPC error: {}", e),
    }
}

async fn cmd_ai(prompt: &str, image: Option<&str>) {
    let client = IpcClient::new();
    match client
        .send(Request::Ai {
            prompt: prompt.to_string(),
            image_path: image.map(|s| s.to_string()),
        })
        .await
    {
        Ok(Response::AiResult { content, .. }) => println!("{}", content),
        Ok(Response::Error(e)) => eprintln!("AI request failed: {}", e),
        Ok(_) => eprintln!("Unexpected response"),
        Err(e) => eprintln!("IPC error: {}", e),
    }
}

async fn cmd_action(name: &str, text: &str, image: Option<&str>) {
    let action_type = match ActionType::from_str(name) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Invalid action: {}", e);
            return;
        }
    };

    let client = IpcClient::new();
    match client
        .send(Request::Action {
            action: action_type,
            text: text.to_string(),
            image_path: image.map(|s| s.to_string()),
        })
        .await
    {
        Ok(Response::ActionResult(result)) => println!("{}", result),
        Ok(Response::Error(e)) => eprintln!("Action failed: {}", e),
        Ok(_) => eprintln!("Unexpected response"),
        Err(e) => eprintln!("IPC error: {}", e),
    }
}

fn cmd_actions() {
    for action in get_action_names() {
        println!("{}", action);
    }
}

async fn cmd_check() {
    let client = IpcClient::new();
    match client.send(Request::CheckTools).await {
        Ok(Response::ToolsStatus {
            capture_missing,
            ocr_missing,
        }) => {
            if capture_missing.is_empty() && ocr_missing.is_empty() {
                println!("All tools are installed");
            } else {
                if !capture_missing.is_empty() {
                    eprintln!("Missing capture tools: {}", capture_missing.join(", "));
                }
                if !ocr_missing.is_empty() {
                    eprintln!("Missing OCR tools: {}", ocr_missing.join(", "));
                }
            }
        }
        Ok(Response::Error(e)) => eprintln!("Check failed: {}", e),
        Ok(_) => eprintln!("Unexpected response"),
        Err(e) => {
            if let pixelens_ipc::IpcError::ConnectionFailed(_) = e {
                eprintln!("Daemon not running. Start with: pixelens daemon");
            } else {
                eprintln!("IPC error: {}", e);
            }
        }
    }
}

fn cmd_config() {
    let config = Config::load();
    println!("Endpoint: {}", config.api_endpoint);
    println!("Model: {}", config.model);
    println!("OCR Language: {}", config.ocr_language);
    println!("Hotkey: {}", config.hotkey);
    if !config.api_key.is_empty() {
        println!("API Key: ****");
    }
}

fn cmd_save_config(
    endpoint: Option<String>,
    key: Option<String>,
    model: Option<String>,
    language: Option<String>,
) {
    let mut config = Config::load();

    if let Some(e) = endpoint {
        config.api_endpoint = e;
    }
    if let Some(k) = key {
        config.api_key = k;
    }
    if let Some(m) = model {
        config.model = m;
    }
    if let Some(l) = language {
        config.ocr_language = l;
    }

    match config.save() {
        Ok(()) => println!("Configuration saved"),
        Err(e) => eprintln!("Failed to save config: {}", e),
    }
}

async fn cmd_daemon() {
    let server = pixelens_ipc::server::IpcServer::new();
    log::info!("Starting Pixelens daemon...");

    // Start signal handler
    let server_clone = pixelens_ipc::server::IpcServer::new();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        log::info!("Shutdown signal received");
        server_clone.stop();
        std::process::exit(0);
    });

    if let Err(e) = server.start().await {
        eprintln!("Daemon error: {}", e);
    }
}
