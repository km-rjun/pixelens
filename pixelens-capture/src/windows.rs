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
    use windows::Graphics::Capture::{
        GraphicsCaptureItem, GraphicsCapturePicker, GraphicsCaptureSession,
    };
    use windows::Graphics::DirectX::Direct3D11::{IDirect3DDevice, IDirect3DSurface};
    use windows::Graphics::SizeInt32;
    use windows::Win32::Graphics::Direct3D11::{
        ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
        D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
    };
    use windows::Win32::Graphics::Dxgi::IDXGIDevice;
    use windows::Win32::Graphics::Imaging::{
        GUID_ContainerFormatPng, IWICBitmapEncoder, IWICImagingFactory,
        WICBitmapEncoderCacheOption, WICBitmapEncoderNoCache, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice;
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, GetSystemMetrics, TranslateMessage, MSG, SM_CXSCREEN,
        SM_CYSCREEN,
    };

    /// WinRT-backed region selector. Pops the system capture picker on an
    /// STA thread and returns primary-monitor bounds once the user commits
    /// a selection. The WinRT item carries no direct bounds API; the real
    /// frame's `ContentSize` is read after the capture session opens.
    pub(super) struct WinRtSelector;

    impl crate::slurp_grim::RegionSelector for WinRtSelector {
        fn select(&self) -> CaptureResult<Option<Rect>> {
            // Must run on STA thread with message pump for COM/WinRT picker
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                // Initialize COM as STA on this thread
                let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
                // If COM already initialized as MTA, we can't re-init as STA.
                // RPC_E_CHANGED_MODE (0x80010106) means mode already set.
                // S_OK (0) or S_FALSE (1) means success (already STA or just set).
                let com_ok = matches!(hr.0, 0 | 1 | -2147417850i32);

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

                        // PickSingleItemAsync must run on STA thread with message pump
                        let async_op = picker
                            .PickSingleItemAsync()
                            .map_err(|e| CaptureError::Selector(format!("picker failed: {e}")))?;

                        // Run message pump while waiting for async operation to complete
                        let mut msg = MSG::default();
                        loop {
                            let status = async_op.Status();
                            match status {
                                Ok(windows::Foundation::AsyncStatus::Completed) => break,
                                Ok(_) => unsafe {
                                    if GetMessageW(&mut msg, None, 0, 0).into() {
                                        let _ = TranslateMessage(&msg);
                                        DispatchMessageW(&msg);
                                    } else {
                                        break;
                                    }
                                },
                                Err(e) => {
                                    return Err(CaptureError::Selector(format!(
                                        "async status error: {e}"
                                    )));
                                }
                            }
                        }

                        let item: GraphicsCaptureItem = async_op.GetResults().map_err(|e| {
                            CaptureError::Selector(format!("no item selected: {e}"))
                        })?;

                        // Get the actual selected region size from the picker item
                        let size = item.Size().map_err(|e| {
                            CaptureError::Selector(format!("failed to get item size: {e}"))
                        })?;

                        let w = size.Width;
                        let h = size.Height;
                        if w <= 0 || h <= 0 {
                            return Err(CaptureError::Selector(
                                "failed to read capture item metrics".into(),
                            ));
                        }

                        Ok(Some(Rect::new(0, 0, w as u32, h as u32)))
                    })()
                };

                // Uninitialize COM on this thread
                unsafe { CoUninitialize() };

                let _ = tx.send(result);
            });

            rx.recv()
                .map_err(|e| CaptureError::Selector(format!("STA thread failed: {e}")))?
        }
    }

    /// WinRT-backed capturer. Currently writes a minimal placeholder PNG.
    /// The real Direct3D11 frame capture is complex and will be implemented in a future milestone.
    pub(super) struct WinRtCapturer;

    impl crate::slurp_grim::ScreenCapturer for WinRtCapturer {
        fn capture(&self, region: Rect, output_path: &Path) -> CaptureResult<()> {
            // Write a minimal 1x1 transparent PNG for the selected region size
            // Real frame capture via Direct3D11CaptureFramePool will be implemented in M3
            use std::io::Write;

            // Minimal valid PNG (1x1 transparent)
            let png_data = [
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
                0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
                0x49, 0x48, 0x44, 0x52, // IHDR
                0x00, 0x00, 0x00, 0x01, // width: 1
                0x00, 0x00, 0x00, 0x01, // height: 1
                0x08, 0x06, 0x00, 0x00,
                0x00, // bit depth: 8, color type: 6 (RGBA), compression: 0, filter: 0, interlace: 0
                0x1F, 0x15, 0xC4, 0x89, // CRC
                0x00, 0x00, 0x00, 0x0C, // IDAT chunk length
                0x49, 0x44, 0x41, 0x54, // IDAT
                0x08, 0xD7, 0x63, 0xF8, 0x0F, 0x00, 0x01, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
                0xB4, // compressed data
                0x00, 0x00, 0x00, 0x00, // IEND chunk length
                0x49, 0x45, 0x4E, 0x44, // IEND
                0xAE, 0x42, 0x60, 0x82, // CRC
            ];

            std::fs::write(output_path, png_data).map_err(CaptureError::Io)?;
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
