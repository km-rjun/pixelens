use pixelens_core::config::Config;
use pixelens_core::ipc::server::IpcServer;
use std::sync::Arc;
use tokio::sync::Notify;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::load();
    log::info!("Pixelens daemon starting");
    log::info!("API endpoint: {}", config.api_endpoint);
    log::info!("OCR language: {}", config.ocr_language);

    let server = IpcServer::new();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let server_task_notify = notify.clone();

    // Handle shutdown signal
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        log::info!("Shutdown signal received");
        notify_clone.notify_one();
    });

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_task_notify.notified().await;
        server_clone.stop();
        std::process::exit(0);
    });

    if let Err(e) = server.start(notify).await {
        log::error!("Daemon error: {}", e);
        std::process::exit(1);
    }
}
