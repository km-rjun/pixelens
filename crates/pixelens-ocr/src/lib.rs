pub mod tesseract;

use pixelens_common::{OcrError, OcrResult};

pub trait OcrEngine {
    fn perform_ocr(&self, image_path: &str, language: &str) -> Result<OcrResult, OcrError>;
    fn is_available(&self) -> bool;
}

pub fn create_engine() -> Result<Box<dyn OcrEngine>, OcrError> {
    let engine = tesseract::TesseractEngine;
    if engine.is_available() {
        Ok(Box::new(engine))
    } else {
        Err(OcrError::ToolNotFound(
            "Tesseract OCR not found".to_string(),
        ))
    }
}

pub fn check_tools() -> Vec<String> {
    let mut missing = Vec::new();
    if !tool_exists("tesseract") {
        missing.push("tesseract".to_string());
    }
    missing
}

fn tool_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tools() {
        let missing = check_tools();
        assert!(missing.is_empty(), "Missing tools: {:?}", missing);
    }

    #[test]
    fn test_create_engine() {
        let result = create_engine();
        assert!(result.is_ok(), "Should create engine: {:?}", result.err());
    }
}
