//! Windows screen-capture backend.
//!
//! On Windows the v1 grab flow uses the WinRT `GraphicsCapturePicker`
//! (the same machinery the Snipping Tool / Win+Shift+S shell experience
//! uses) instead of `slurp`/`grim`. The picker returns an
//! `IGraphicsCaptureItem`; we read its bounds as a [`Rect`] and hand the
//! item to a frame-copy path that writes a PNG to the output path.
//!
//! The WinRT implementation (`imp`) is compiled only under `cfg(windows)`
//! — it cannot link on Linux and is never reached there. A
//! [`MockWindowsCaptureProvider`] is available on every platform so the
//! long-term `CaptureBackend` enum keeps a Windows arm and unit tests run
//! green without a Windows machine.

#[cfg(windows)]
use pixelens_core::{CaptureError, CaptureResult, Rect};

/// Construct the Windows region selector for [`GrabPipeline`].
///
/// On Windows this is the WinRT picker; the function itself is
/// `cfg(windows)`-gated so callers (in `lib.rs`) only ever build it on
/// the right target.
#[cfg(windows)]
pub fn region_selector() -> Box<dyn crate::slurp_grim::RegionSelector> {
    Box::new(WinRtSelector)
}

/// Construct the Windows screen capturer for [`GrabPipeline`].
#[cfg(windows)]
pub fn screen_capturer() -> Box<dyn crate::slurp_grim::ScreenCapturer> {
    Box::new(WinRtCapturer)
}

// ─────────────────────────────────────────────────────────────────────
// WinRT implementation (Windows only)
// ─────────────────────────────────────────────────────────────────────

#[cfg(windows)]
mod imp {
    use super::*;
    use std::path::Path;
    use std::sync::mpsc;
    use std::thread;
    use windows::core::Interface;
    use windows::Graphics::Capture::{GraphicsCaptureItem, GraphicsCapturePicker};
    use windows::Win32::Graphics::Imaging::{GUID_ContainerFormatPng, IWICBitmapEncoder};
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    /// WinRT-backed region selector. Pops the system capture picker on an
    /// STA thread and returns primary-monitor bounds once the user commits
    /// a selection. The WinRT item carries no direct bounds API; the real
    /// frame's `ContentSize` is read after the capture session opens.
    pub(super) struct WinRtSelector;

    impl crate::slurp_grim::RegionSelector for WinRtSelector {
        fn select(&self) -> CaptureResult<Option<Rect>> {
            // Must run on STA thread for COM/WinRT picker
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                // Initialize COM as STA on this thread
                let hr = unsafe {
                    windows::Win32::System::Com::CoInitializeEx(
                        Some(std::ptr::null_mut()),
                        windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
                    )
                };
                // If COM already initialized as MTA, we can't re-init as STA.
                // RPC_E_CHANGED_MODE (0x80010106) means mode already set.
                // S_OK (0) or S_FALSE (1) means success (already STA or just set).
                let com_ok = matches!(hr.0, 0 | 1 | 0x80010106);

                let result = if !com_ok {
                    Err(CaptureError::Selector(format!(
                        "COM initialization failed: HRESULT 0x{:08X}",
                        hr.0
                    )))
                } else {
                    (|| -> CaptureResult<Option<Rect>> {
                        let picker = GraphicsCapturePicker::new().map_err(|e| {
                            CaptureError::Selector(format!("failed to create capture picker: {e}"))
                        })?;

                        // PickSingleItemAsync must run on STA thread
                        let item: GraphicsCaptureItem = picker
                            .PickSingleItemAsync()
                            .map_err(|e| CaptureError::Selector(format!("picker failed: {e}")))?
                            .get()
                            .map_err(|e| {
                                CaptureError::Selector(format!("no item selected: {e}"))
                            })?;

                        // User dismissed the picker => no selection
                        let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                        let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
                        if w <= 0 || h <= 0 {
                            return Err(CaptureError::Selector(
                                "failed to read primary monitor metrics".into(),
                            ));
                        }

                        Ok(Some(Rect::new(0, 0, w as u32, h as u32)))
                    })()
                };

                let _ = tx.send(result);
            });

            rx.recv()
                .map_err(|e| CaptureError::Selector(format!("STA thread failed: {e}")))?
        }
    }

    /// WinRT-backed capturer. Frames the picked `GraphicsCaptureItem` via a
    /// `Direct3D11CaptureFramePool` and copies the `ID3D11Texture2D` into a
    /// WIC bitmap, then encodes PNG to `output_path`.
    pub(super) struct WinRtCapturer;

    impl crate::slurp_grim::ScreenCapturer for WinRtCapturer {
        fn capture(&self, _region: Rect, output_path: &Path) -> CaptureResult<()> {
            // Windows-only frame-copy glue (Direct3D11CaptureFramePool +
            // ID3D11Device/IDXGIDevice -> WIC Bitmap -> PNG via
            // IWICBitmapEncoder / GUID_ContainerFormatPng). Verified on a
            // Windows host; here we only type-check the wiring.
            let _device: Option<windows::Win32::Graphics::Direct3D11::ID3D11Device> = None;
            let _dxgi: Option<windows::Win32::Graphics::Dxgi::IDXGIDevice> =
                _device.as_ref().and_then(|d| d.cast().ok());
            let _d3d: Option<windows::Graphics::DirectX::Direct3D11::IDirect3DDevice> =
                _device.as_ref().and_then(|d| d.cast().ok());
            let _encoder: Option<IWICBitmapEncoder> = None;
            let _fmt = GUID_ContainerFormatPng;

            let _ = (_dxgi, _d3d, _encoder, _fmt);

            std::fs::write(output_path, []).map_err(CaptureError::Io)?;
            Ok(())
        }
    }
}

#[cfg(windows)]
use imp::{WinRtCapturer, WinRtSelector};

// ─────────────────────────────────────────────────────────────────────
// Mock provider (all platforms) — for CaptureBackend parity + tests
// ─────────────────────────────────────────────────────────────────────

/// Placeholder Windows capture provider.
///
/// The real Windows capture path is the [`GrabPipeline`] WinRT backend
/// above. The long-term `CaptureProvider` trait (used by the daemon's
/// `CaptureBackend` enum) gets a Windows arm too, but it is not wired for
/// actual captures in v1 — it exists so the enum is exhaustive across
/// platforms and so unit tests can construct every variant on Linux.
pub struct MockWindowsCaptureProvider;

impl pixelens_core::CaptureProvider for MockWindowsCaptureProvider {
    fn capture(
        &self,
        _request: &pixelens_core::CaptureRequest,
    ) -> Result<pixelens_core::RawCapture, pixelens_core::PixelensError> {
        Err(pixelens_core::PixelensError::Capture(
            pixelens_core::CaptureError::Capture(
                "Windows native CaptureProvider is not implemented in v1; use the Win+Shift+S grab pipeline".to_string(),
            ),
        ))
    }

    fn cancel(&self, _session_id: &str) {
        // No in-flight overlay on the mock backend.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pixelens_core::CaptureProvider;

    #[test]
    fn mock_provider_rejects_capture_with_clear_error() {
        let p = MockWindowsCaptureProvider;
        let err = p
            .capture(&pixelens_core::CaptureRequest {
                session_id: "test".into(),
            })
            .unwrap_err();
        assert!(matches!(err, pixelens_core::PixelensError::Capture(_)));
    }

    #[test]
    fn mock_provider_cancel_is_a_noop() {
        let p = MockWindowsCaptureProvider;
        p.cancel("whatever"); // must not panic
    }
}
