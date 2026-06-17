//! OCR engine layer.
//!
//! M1: skeleton. The `TesseractOcrEngine` type is declared so the
//! dependency graph compiles, but the real Tesseract binding lands in
//! M5 alongside dependency validation and warm init.

use pixelens_core::{CaptureImage, OcrEngine, OcrError};

/// Tesseract-backed OCR engine. Constructed once at daemon startup and
/// held warm — per-request initialisation would add ~300–500 ms and
/// break the <1 s OCR target (PRD §"OCR").
pub struct TesseractOcrEngine {
    // Real fields (handle, language, page-seg-mode, etc.) arrive in M5.
    _private: (),
}

impl TesseractOcrEngine {
    /// Initialise a warm Tesseract engine. Returns an error if the
    /// `tesseract` binary is not on `$PATH` (PRD §"Dependency Management").
    pub fn new() -> Result<Self, OcrError> {
        // M5: invoke `tesseract --version` to confirm presence, then load
        // the API and prime the engine. M1 only checks the trait wiring.
        Ok(Self { _private: () })
    }
}

impl OcrEngine for TesseractOcrEngine {
    fn extract_text(&self, _image: &CaptureImage) -> Result<String, OcrError> {
        Err(OcrError::Engine(
            "TesseractOcrEngine not yet implemented (M5)".to_string(),
        ))
    }
}
