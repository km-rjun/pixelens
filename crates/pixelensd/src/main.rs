use tokio::signal;

use pixelens_config::Config;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::load();
    log::info!("Pixelens daemon starting");
    log::info!("API endpoint: {}", config.api_endpoint);
    log::info!("OCR language: {}", config.ocr_language);

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            log::info!("Shutdown signal received");
        }
        Err(err) => {
            log::error!("Failed to listen for shutdown signal: {}", err);
        }
    }

    log::info!("Pixelens daemon stopped");
}
