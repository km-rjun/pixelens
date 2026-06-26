use clap::{Parser, Subcommand};
use std::process;

use pixelens_core::config::Config;
use pixelens_core::ipc::client::IpcClient;
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
    /// Capture a region and show extracted text
    Grab {
        /// Search the web for extracted text
        #[arg(long)]
        search: bool,

        /// Ask AI about the captured region
        #[arg(long)]
        ai: Option<String>,
    },

    /// Copy text to clipboard
    Copy {
        /// Text to copy
        text: String,
    },

    /// Search the web for text
    Search {
        /// Text to search
        text: String,
    },

    /// Ask AI about text or an image
    Ai {
        /// Prompt to send to AI
        prompt: String,

        /// Optional image path
        #[arg(long)]
        image: Option<String>,
    },

    /// Translate text
    Translate {
        /// Text to translate
        text: String,

        /// Target language (default: English)
        #[arg(long, default_value = "English")]
        to: String,
    },

    /// Reverse image search
    Image {
        /// Image path
        path: String,
    },

    /// Start the daemon
    Daemon,

    /// Show daemon status
    Status,

    /// Stop the daemon
    Stop,

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

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Grab { search, ai } => cmd_grab(search, ai.as_deref()).await,
        Commands::Copy { text } => cmd_copy(&text).await,
        Commands::Search { text } => cmd_search(&text).await,
        Commands::Ai { prompt, image } => cmd_ai(&prompt, image.as_deref()).await,
        Commands::Translate { text, to } => cmd_translate(&text, &to).await,
        Commands::Image { path } => cmd_image(&path).await,
        Commands::Daemon => cmd_daemon().await,
        Commands::Status => cmd_status().await,
        Commands::Stop => cmd_stop().await,
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

async fn cmd_grab(search: bool, ai: Option<&str>) -> i32 {
    let client = IpcClient::new();
    match client.grab(search, ai).await {
        Ok((image_path, text, ai_response)) => {
            println!("{}", image_path);
            if let Some(t) = text {
                println!("{}", t);
            }
            if let Some(a) = ai_response {
                println!("{}", a);
            }
            0
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            eprintln!("Daemon not running. Start with: pixelens daemon");
            1
        }
        Err(e) => {
            eprintln!("Grab failed: {}", e);
            1
        }
    }
}

async fn cmd_copy(text: &str) -> i32 {
    let client = IpcClient::new();
    match client.action(ActionType::CopyToClipboard, text, None).await {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Copy failed: {}", e);
            1
        }
    }
}

async fn cmd_search(text: &str) -> i32 {
    let client = IpcClient::new();
    match client.action(ActionType::SearchWeb, text, None).await {
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

async fn cmd_ai(prompt: &str, image: Option<&str>) -> i32 {
    let client = IpcClient::new();
    match client.ai(prompt, image).await {
        Ok((content, _)) => {
            println!("{}", content);
            0
        }
        Err(pixelens_core::ipc::IpcError::ConnectionFailed(_)) => {
            eprintln!("Daemon not running. Start with: pixelens daemon");
            1
        }
        Err(e) => {
            eprintln!("AI request failed: {}", e);
            1
        }
    }
}

async fn cmd_translate(text: &str, to: &str) -> i32 {
    let client = IpcClient::new();
    match client
        .action(ActionType::Translate(to.to_string()), text, None)
        .await
    {
        Ok(prompt) => match client.ai(&prompt, None).await {
            Ok((content, _)) => {
                println!("{}", content);
                0
            }
            Err(e) => {
                eprintln!("Translation failed: {}", e);
                1
            }
        },
        Err(e) => {
            eprintln!("Translate failed: {}", e);
            1
        }
    }
}

async fn cmd_image(path: &str) -> i32 {
    let client = IpcClient::new();
    match client
        .action(ActionType::ReverseImageSearch, "", Some(path))
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

async fn cmd_daemon() -> i32 {
    let server = pixelens_core::ipc::server::IpcServer::new();
    log::info!("Starting Pixelens daemon...");

    let notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let notify_clone = notify.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        log::info!("Shutdown signal received");
        notify_clone.notify_one();
    });

    let server_clone = server.clone();
    tokio::spawn(async move {
        notify.notified().await;
        server_clone.stop();
        process::exit(0);
    });

    if let Err(e) = server.start().await {
        eprintln!("Daemon error: {}", e);
        1
    } else {
        0
    }
}

async fn cmd_status() -> i32 {
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

async fn cmd_stop() -> i32 {
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
