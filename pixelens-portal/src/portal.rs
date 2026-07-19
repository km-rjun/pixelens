//! Portal-native capture backend (UM5).
//!
//! [`PortalBackend`] implements [`pixelens_core::traits::CaptureProvider`] by
//! talking to `org.freedesktop.portal.ScreenCast` when the `portal` feature
//! is enabled. The portal I/O is abstracted behind the [`PortalSession`]
//! trait so the capture logic is unit-testable without a live DBus/pipewire
//! session.
//!
//! When the portal is unavailable, [`PortalBackend::capture`] transparently
//! falls back to the existing `slurp` + `grim` workflow
//! (`pixelens_capture::slurp_grim`), preserving the v1 behavior exactly. The
//! PNG produced by `grim` is decoded to an RGBA [`CaptureImage`] through the
//! [`PngDecoder`] trait — a real `image`-crate-backed decoder ships behind
//! the `portal` feature, and a mock decoder is used in tests so no external
//! binaries or crate features are required to exercise the wiring.

#[cfg(feature = "portal")]
use pixelens_capture::{GrimCapturer, SlurpSelector};
use pixelens_capture::{RegionSelector, ScreenCapturer};
use pixelens_core::{
    CaptureError, CaptureImage, CaptureProvider, CaptureRequest, PixelensError, RawCapture,
};

/// Outcome of a single portal session attempt.
#[derive(Debug, Clone)]
pub enum PortalOutcome {
    /// The selected region was captured successfully.
    Captured(RawCapture),
    /// The user cancelled the interactive selection.
    Cancelled,
    /// The portal is unavailable (no service, DBus error, …); the caller
    /// should fall back to the legacy `slurp`/`grim` path.
    Unavailable,
}

/// Abstraction over a single portal screen-cast session.
///
/// Keeping this behind a trait lets [`PortalBackend`] run all of its logic
/// against [`MockPortalSession`] in unit tests without a real
/// `org.freedesktop.portal.ScreenCast` service.
pub trait PortalSession: Send + Sync {
    /// Attempt to capture the user-selected region through the portal.
    ///
    /// Returns [`PortalOutcome::Unavailable`] (rather than an error) when the
    /// portal cannot be reached, so the caller falls back transparently.
    fn run(&self, request: &CaptureRequest) -> Result<PortalOutcome, PixelensError>;
}

/// Decodes a PNG byte buffer into an RGBA [`CaptureImage`].
///
/// Abstracted behind a trait so the `image`-crate-backed decoder can stay
/// feature-gated while tests inject a decoder that needs no external crate.
pub trait PngDecoder: Send + Sync {
    fn decode(&self, data: &[u8]) -> Result<CaptureImage, PixelensError>;
}

/// Portal-native capture backend with transparent `slurp`/`grim` fallback.
pub struct PortalBackend {
    session: Box<dyn PortalSession>,
    selector: Box<dyn RegionSelector>,
    capturer: Box<dyn ScreenCapturer>,
    decoder: Box<dyn PngDecoder>,
}

impl PortalBackend {
    /// Build a backend from explicit components (used by tests and callers
    /// that want to inject mock session/selector/capturer/decoder).
    pub fn with_components(
        session: Box<dyn PortalSession>,
        selector: Box<dyn RegionSelector>,
        capturer: Box<dyn ScreenCapturer>,
        decoder: Box<dyn PngDecoder>,
    ) -> Self {
        Self {
            session,
            selector,
            capturer,
            decoder,
        }
    }

    /// Build the production backend: a real portal session with the legacy
    /// `slurp`/`grim` pair as the fallback.
    #[cfg(feature = "portal")]
    pub fn new() -> Self {
        Self::with_components(
            Box::new(RealPortalSession::new()),
            Box::new(SlurpSelector::new()),
            Box::new(GrimCapturer::new()),
            Box::new(RealPngDecoder),
        )
    }

    /// Run the legacy `slurp` (select) + `grim` (capture) + PNG decode path.
    fn fallback_capture(&self, _request: &CaptureRequest) -> Result<RawCapture, PixelensError> {
        let Some(rect) = self.selector.select()? else {
            tracing::info!("region selection cancelled; aborting capture");
            return Err(PixelensError::Capture(CaptureError::Selector(
                "selection cancelled".to_string(),
            )));
        };

        let tmp_dir = std::env::temp_dir();
        let tmp_path = tmp_dir.join(format!("pixelens-grim-{}.png", uuid::Uuid::new_v4()));
        self.capturer.capture(rect, &tmp_path)?;
        let bytes = std::fs::read(&tmp_path).map_err(PixelensError::Io)?;
        let _ = std::fs::remove_file(&tmp_path);
        let image = self.decoder.decode(&bytes)?;
        Ok(RawCapture {
            region: rect,
            image,
        })
    }
}

#[cfg(feature = "portal")]
impl Default for PortalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CaptureProvider for PortalBackend {
    fn capture(&self, request: &CaptureRequest) -> Result<RawCapture, PixelensError> {
        match self.session.run(request) {
            Ok(PortalOutcome::Captured(rc)) => Ok(rc),
            Ok(PortalOutcome::Cancelled) => {
                tracing::info!("portal capture cancelled; aborting");
                Err(PixelensError::Capture(CaptureError::Selector(
                    "portal capture cancelled".to_string(),
                )))
            }
            Ok(PortalOutcome::Unavailable) => {
                tracing::info!("portal unavailable; falling back to slurp/grim");
                self.fallback_capture(request)
            }
            Err(e) => {
                tracing::warn!(error = %e, "portal session error; falling back to slurp/grim");
                self.fallback_capture(request)
            }
        }
    }

    fn cancel(&self, _session_id: &str) {
        // Portal sessions are short-lived and the slurp/grim fallback cannot
        // be cancelled mid-process; match the existing no-op behavior.
    }
}

// ----------------------------------------------------------------------------
// Real implementations (portal feature only)
// ----------------------------------------------------------------------------

#[cfg(feature = "portal")]
mod real {
    use super::{PngDecoder, PortalOutcome, PortalSession};
    use ashpd::desktop::screencast::{CursorMode, Screencast, SourceType};
    use ashpd::desktop::PersistMode;
    use pixelens_core::{CaptureImage, CaptureRequest, PixelensError};

    /// Real portal session backed by `ashpd` (xdg-desktop-portal).
    ///
    /// The DBus/pipewire plumbing is only reachable on a live desktop session.
    /// Headless (no session bus) it returns [`PortalOutcome::Unavailable`] so
    /// the backend falls back to `slurp`/`grim` — which is exactly the safety
    /// guarantee required by UM5.
    pub struct RealPortalSession;

    impl RealPortalSession {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for RealPortalSession {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PortalSession for RealPortalSession {
        fn run(&self, _request: &CaptureRequest) -> Result<PortalOutcome, PixelensError> {
            // ashpd's API is async; drive it on a single-threaded tokio runtime.
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    PixelensError::Capture(pixelens_core::CaptureError::Capture(format!(
                        "failed to build runtime: {e}"
                    )))
                })?;

            rt.block_on(async {
                let proxy = match Screencast::new().await {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::debug!(error = %e, "screencast portal unavailable");
                        return Ok(PortalOutcome::Unavailable);
                    }
                };

                let session = match proxy.create_session().await {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::debug!(error = %e, "failed to create screencast session");
                        return Ok(PortalOutcome::Unavailable);
                    }
                };

                if proxy
                    .select_sources(
                        &session,
                        CursorMode::Hidden,
                        SourceType::Monitor.into(),
                        false,
                        None,
                        PersistMode::DoNot,
                    )
                    .await
                    .is_err()
                {
                    return Ok(PortalOutcome::Unavailable);
                }

                // `start` returns a `Request<Streams>`; a cancelled dialog
                // surfaces as an error here, which we treat as a cancelled
                // capture below.
                let streams = match proxy.start(&session, None).await {
                    Ok(req) => req,
                    Err(e) => {
                        tracing::debug!(error = %e, "screencast start failed");
                        return Ok(PortalOutcome::Unavailable);
                    }
                };

                // Resolve the user's selection. A dismissed/cancelled dialog is
                // reported by the portal as an error response.
                let _streams = match streams.response() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::debug!(error = %e, "screencast selection cancelled");
                        return Ok(PortalOutcome::Cancelled);
                    }
                };

                // The pipewire fd/stream is obtained here; decoding pipewire
                // frames to pixels requires the `pipewire` crate, which is out
                // of scope for this milestone. On a real desktop this is where
                // the selected region's pixels would be read. Until then we
                // signal availability via the portal but defer to the fallback
                // for actual pixel extraction so behavior stays correct.
                let _ = proxy.open_pipe_wire_remote(&session).await;
                Ok(PortalOutcome::Unavailable)
            })
        }
    }

    /// PNG decoder backed by the `image` crate.
    pub struct RealPngDecoder;

    impl PngDecoder for RealPngDecoder {
        fn decode(&self, data: &[u8]) -> Result<CaptureImage, PixelensError> {
            let img = image::load_from_memory_with_format(data, image::ImageFormat::Png)
                .map_err(|e| {
                    PixelensError::Capture(pixelens_core::CaptureError::Capture(format!(
                        "failed to decode PNG: {e}"
                    )))
                })?
                .to_rgba8();
            let (width, height) = img.dimensions();
            Ok(CaptureImage {
                width,
                height,
                stride: width * 4,
                pixels: img.into_raw(),
            })
        }
    }
}

#[cfg(feature = "portal")]
pub use real::{RealPngDecoder, RealPortalSession};

// ----------------------------------------------------------------------------
// In-memory mocks + unit tests (compile under both default and `portal`)
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pixelens_core::Rect;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    /// Portal session that returns a known 2x2 RGBA buffer + known region.
    struct MockPortalSession {
        outcome: PortalOutcome,
    }

    impl PortalSession for MockPortalSession {
        fn run(&self, _request: &CaptureRequest) -> Result<PortalOutcome, PixelensError> {
            Ok(self.outcome.clone())
        }
    }

    /// Decoder that ignores the bytes and returns a fixed known image.
    struct MockPngDecoder {
        image: CaptureImage,
    }

    impl PngDecoder for MockPngDecoder {
        fn decode(&self, _data: &[u8]) -> Result<CaptureImage, PixelensError> {
            Ok(self.image.clone())
        }
    }

    /// In-memory region selector that always returns a fixed rect.
    struct MockSelector {
        rect: Option<Rect>,
        called: Arc<AtomicBool>,
    }

    impl RegionSelector for MockSelector {
        fn select(&self) -> pixelens_core::CaptureResult<Option<Rect>> {
            self.called.store(true, Ordering::SeqCst);
            Ok(self.rect)
        }
    }

    /// In-memory screen capturer that writes a fixed byte blob (a "PNG") to
    /// the requested path, recording that it was invoked.
    struct MockCapturer {
        bytes: Vec<u8>,
        called: Arc<AtomicBool>,
        path: Arc<Mutex<Option<std::path::PathBuf>>>,
    }

    impl ScreenCapturer for MockCapturer {
        fn capture(&self, _region: Rect, output_path: &Path) -> pixelens_core::CaptureResult<()> {
            self.called.store(true, Ordering::SeqCst);
            *self.path.lock().unwrap() = Some(output_path.to_path_buf());
            std::fs::write(output_path, &self.bytes).map_err(pixelens_core::CaptureError::Io)?;
            Ok(())
        }
    }

    const KNOWN_RECT: Rect = Rect::new(10, 20, 2, 2);
    const KNOWN_PIXELS: [u8; 16] = [
        0xFF, 0x00, 0x00, 0xFF, // red
        0x00, 0xFF, 0x00, 0xFF, // green
        0x00, 0x00, 0xFF, 0xFF, // blue
        0xFF, 0xFF, 0xFF, 0xFF, // white
    ];

    fn known_image() -> CaptureImage {
        CaptureImage {
            width: 2,
            height: 2,
            stride: 8,
            pixels: KNOWN_PIXELS.to_vec(),
        }
    }

    #[test]
    fn mock_portal_session_returns_known_capture() {
        let session = MockPortalSession {
            outcome: PortalOutcome::Captured(RawCapture {
                region: KNOWN_RECT,
                image: known_image(),
            }),
        };
        let selector_called = Arc::new(AtomicBool::new(false));
        let capturer_called = Arc::new(AtomicBool::new(false));

        let backend = PortalBackend::with_components(
            Box::new(session),
            Box::new(MockSelector {
                rect: Some(KNOWN_RECT),
                called: selector_called.clone(),
            }),
            Box::new(MockCapturer {
                bytes: vec![],
                called: capturer_called.clone(),
                path: Arc::new(Mutex::new(None)),
            }),
            Box::new(MockPngDecoder {
                image: known_image(),
            }),
        );

        let req = CaptureRequest {
            session_id: "test-1".to_string(),
        };
        let rc = backend.capture(&req).expect("capture should succeed");

        assert_eq!(rc.region, KNOWN_RECT);
        assert_eq!(rc.image.width, 2);
        assert_eq!(rc.image.height, 2);
        assert_eq!(rc.image.stride, 8);
        assert_eq!(rc.image.pixels, KNOWN_PIXELS);

        // Portal succeeded, so the fallback must NOT have been invoked.
        assert!(!selector_called.load(Ordering::SeqCst));
        assert!(!capturer_called.load(Ordering::SeqCst));
    }

    #[test]
    fn unavailable_portal_falls_back_to_slurp_grim() {
        let session = MockPortalSession {
            outcome: PortalOutcome::Unavailable,
        };
        let selector_called = Arc::new(AtomicBool::new(false));
        let capturer_called = Arc::new(AtomicBool::new(false));
        let captured_path = Arc::new(Mutex::new(None::<std::path::PathBuf>));

        let backend = PortalBackend::with_components(
            Box::new(session),
            Box::new(MockSelector {
                rect: Some(KNOWN_RECT),
                called: selector_called.clone(),
            }),
            Box::new(MockCapturer {
                bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
                called: capturer_called.clone(),
                path: captured_path.clone(),
            }),
            Box::new(MockPngDecoder {
                image: known_image(),
            }),
        );

        let req = CaptureRequest {
            session_id: "test-2".to_string(),
        };
        let rc = backend
            .capture(&req)
            .expect("fallback capture should succeed");

        // Fallback wiring must have invoked both selector and capturer.
        assert!(
            selector_called.load(Ordering::SeqCst),
            "selector not invoked"
        );
        assert!(
            capturer_called.load(Ordering::SeqCst),
            "capturer not invoked"
        );
        assert!(captured_path.lock().unwrap().is_some());

        // And produced the decoder's known image for the selected region.
        assert_eq!(rc.region, KNOWN_RECT);
        assert_eq!(rc.image.pixels, KNOWN_PIXELS);
    }

    #[test]
    fn cancelled_portal_does_not_fall_back() {
        let session = MockPortalSession {
            outcome: PortalOutcome::Cancelled,
        };
        let selector_called = Arc::new(AtomicBool::new(false));
        let capturer_called = Arc::new(AtomicBool::new(false));

        let backend = PortalBackend::with_components(
            Box::new(session),
            Box::new(MockSelector {
                rect: Some(KNOWN_RECT),
                called: selector_called.clone(),
            }),
            Box::new(MockCapturer {
                bytes: vec![],
                called: capturer_called.clone(),
                path: Arc::new(Mutex::new(None)),
            }),
            Box::new(MockPngDecoder {
                image: known_image(),
            }),
        );

        let req = CaptureRequest {
            session_id: "test-3".to_string(),
        };
        let err = backend.capture(&req).expect_err("cancel should error");
        assert!(matches!(err, PixelensError::Capture(_)));
        assert!(!selector_called.load(Ordering::SeqCst));
        assert!(!capturer_called.load(Ordering::SeqCst));
    }
}
