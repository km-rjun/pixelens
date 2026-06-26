use clap::{Parser, Subcommand};

use pixelens_actions::get_action_names;
use pixelens_capture::{check_tools as check_capture_tools, detect_backend};
use pixelens_config::Config;
use pixelens_ocr::{check_tools as check_ocr_tools, create_engine};

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
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Capture => cmd_capture(),
        Commands::Ocr { image, language } => cmd_ocr(&image, &language),
        Commands::Ai { prompt, image } => cmd_ai(&prompt, image.as_deref()),
        Commands::Action { name, text, image } => cmd_action(&name, &text, image.as_deref()),
        Commands::Actions => cmd_actions(),
        Commands::Check => cmd_check(),
        Commands::Config => cmd_config(),
        Commands::SaveConfig {
            endpoint,
            key,
            model,
            language,
        } => {
            cmd_save_config(endpoint, key, model, language);
        }
    }
}

fn cmd_capture() {
    match detect_backend() {
        Ok(backend) => match backend.select_region() {
            Ok(region) => match backend.capture(&region) {
                Ok(result) => println!("{}", result.image_path),
                Err(e) => eprintln!("Capture failed: {}", e),
            },
            Err(e) => eprintln!("Selection failed: {}", e),
        },
        Err(e) => eprintln!("No capture backend: {}", e),
    }
}

fn cmd_ocr(image: &str, language: &str) {
    match create_engine() {
        Ok(engine) => match engine.perform_ocr(image, language) {
            Ok(result) => print!("{}", result.text),
            Err(e) => eprintln!("OCR failed: {}", e),
        },
        Err(e) => eprintln!("OCR engine not available: {}", e),
    }
}

fn cmd_ai(prompt: &str, image: Option<&str>) {
    let config = Config::load();
    let client = pixelens_actions::ai::OpenAiClient::new(
        config.api_endpoint.clone(),
        config.api_key.clone(),
        config.model.clone(),
    );

    let request = pixelens_common::AiRequest {
        prompt: prompt.to_string(),
        image_path: image.map(|s| s.to_string()),
    };

    match client.chat(&request) {
        Ok(response) => println!("{}", response.content),
        Err(e) => eprintln!("AI request failed: {}", e),
    }
}

fn cmd_action(name: &str, text: &str, image: Option<&str>) {
    let action_type = match name.parse::<pixelens_common::ActionType>() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Invalid action: {}", e);
            return;
        }
    };

    let handler = match pixelens_actions::get_handler(&action_type) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to get handler: {}", e);
            return;
        }
    };

    let payload = pixelens_common::ActionPayload {
        text: text.to_string(),
        image_path: image.map(|s| s.to_string()),
    };

    match handler.execute(&payload) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Action failed: {}", e),
    }
}

fn cmd_actions() {
    for action in get_action_names() {
        println!("{}", action);
    }
}

fn cmd_check() {
    let capture_missing = check_capture_tools();
    let ocr_missing = check_ocr_tools();

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
