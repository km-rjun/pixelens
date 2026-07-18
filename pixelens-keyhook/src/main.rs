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

    // Combo resolution (M8): env PIXELENS_HOTKEY wins, then the on-disk
    // config's general.hotkey, then the model default. This makes the
    // config key actually drive the listener rather than only being
    // parsed.
    let env_combo = std::env::var("PIXELENS_HOTKEY").ok();
    let combo_str = match &env_combo {
        Some(c) => c.clone(),
        None => match pixelens_config::load_config() {
            Ok(cfg) => {
                tracing::info!(hotkey = %cfg.general.hotkey, "using config hotkey as keyhook combo");
                cfg.general.hotkey
            }
            Err(e) => {
                tracing::warn!(error = %e, "config load failed; using default hotkey");
                "Super+Shift+T".to_string()
            }
        },
    };
    let combo = KeyCombo::parse(&combo_str)
        .map_err(|e| anyhow::anyhow!("invalid hotkey combo '{combo_str}': {e}"))?;

    let display = detect_display_server()
        .map_err(|e| anyhow::anyhow!("display server detection failed: {e}"))?;

    let listener = backend::build(display, combo)?;
    listener.run()?;
    Ok(())
}
