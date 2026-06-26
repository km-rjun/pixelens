pub mod tesseract;

use crate::error::OcrError;
use crate::types::OcrResult;

pub trait OcrEngine {
    fn perform_ocr(&self, image_path: &str, language: &str) -> Result<OcrResult, OcrError>;
    fn is_available(&self) -> bool;
}

pub fn create_engine() -> Box<dyn OcrEngine> {
    Box::new(tesseract::TesseractEngine)
}
