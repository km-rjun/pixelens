//! `pixelensd` — the Pixelens background daemon binary entry point.
//!
//! All non-trivial logic lives in the [`pixelens_daemon`] library so
//! integration tests can exercise it without spawning a subprocess.
//! This file is a thin wrapper that calls [`pixelens_daemon::run`].

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(pixelens_daemon::run())
}
