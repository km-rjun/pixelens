use pixelens_config::Config;
use pixelens_ipc::server::IpcServer;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::load();
    log::info!("Pixelens daemon starting");
    log::info!("API endpoint: {}", config.api_endpoint);
    log::info!("OCR language: {}", config.ocr_language);

    let server = IpcServer::new();

    // Handle shutdown signal
    let server_clone = IpcServer::new();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        log::info!("Shutdown signal received");
        server_clone.stop();
        std::process::exit(0);
    });

    if let Err(e) = server.start().await {
        log::error!("Daemon error: {}", e);
        std::process::exit(1);
    }
}
