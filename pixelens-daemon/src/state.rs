//! Daemon-wide state shared between the IPC server and the dispatcher.
//!
//! In v1 the state is small: a snapshot of the detected display server,
//! an optional capture pipeline, and an optional warm OCR engine. The
//! pipeline may be absent if `slurp` / `grim` aren't installed; in that
//! case `pixelens grab` returns a clear `MissingTool` error and the
//! daemon keeps running for the other commands. Similarly the OCR engine
//! is `None` when `tesseract` is unavailable — grabs still succeed, they
//! just return no extracted text (M5).

use pixelens_capture::{DisplayServer, GrabPipeline};
use pixelens_ocr::TesseractOcrEngine;

pub struct DaemonState {
    pub display: DisplayServer,
    /// `None` when the pipeline failed to construct (e.g. slurp/grim
    /// missing). All other commands are unaffected; only `grab`
    /// surfaces this.
    pub pipeline: Option<GrabPipeline>,
    /// Warm OCR engine. `None` when `tesseract` is unavailable; grabs
    /// still work, they simply return empty `text` (M5 degrade path).
    pub ocr: Option<TesseractOcrEngine>,
}

impl DaemonState {
    pub fn new(
        display: DisplayServer,
        pipeline: Option<GrabPipeline>,
        ocr: Option<TesseractOcrEngine>,
    ) -> Self {
        Self {
            display,
            pipeline,
            ocr,
        }
    }
}
