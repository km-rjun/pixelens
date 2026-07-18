//! `pixelens-keyhook` binary entry point.

use pixelens_capture::detect_display_server;
use pixelens_keyhook::backend;
use pixelens_keyhook::KeyCombo;

fn main() -> anyhow::Result<()> {
    // Minimal tracing init (no daemon dependency).
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();

    tracing::info!("pixelens-keyhook starting");

    // Combo comes from config (general.hotkey). Fall back to a sane default.
    let combo_str =
        std::env::var("PIXELENS_HOTKEY").unwrap_or_else(|_| "Super+Shift+T".to_string());
    let combo = KeyCombo::parse(&combo_str)
        .map_err(|e| anyhow::anyhow!("invalid hotkey combo '{combo_str}': {e}"))?;

    let display = detect_display_server()
        .map_err(|e| anyhow::anyhow!("display server detection failed: {e}"))?;

    let listener = backend::build(display, combo)?;
    listener.run()?;
    Ok(())
}
