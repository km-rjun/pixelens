use std::process::Command;

use crate::OcrEngine;
use pixelens_common::{OcrError, OcrResult};

pub struct TesseractEngine;

impl OcrEngine for TesseractEngine {
    fn perform_ocr(&self, image_path: &str, language: &str) -> Result<OcrResult, OcrError> {
        if !std::path::Path::new(image_path).exists() {
            return Err(OcrError::InvalidImage(format!(
                "File not found: {}",
                image_path
            )));
        }

        let output = Command::new("tesseract")
            .args([image_path, "stdout", "-l", language])
            .output()
            .map_err(|e| OcrError::ToolNotFound(format!("tesseract: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OcrError::ToolFailed(stderr.to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(OcrResult {
            text,
            language: language.to_string(),
        })
    }

    fn is_available(&self) -> bool {
        Command::new("tesseract")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        let engine = TesseractEngine;
        let result = engine.is_available();
        assert!(result, "tesseract should be available");
    }

    #[test]
    fn test_file_not_found() {
        let engine = TesseractEngine;
        let result = engine.perform_ocr("/tmp/nonexistent.png", "eng");
        assert!(result.is_err());
        match result.unwrap_err() {
            OcrError::InvalidImage(msg) => assert!(msg.contains("File not found")),
            _ => panic!("Expected InvalidImage error"),
        }
    }

    #[test]
    fn test_perform_ocr() {
        let engine = TesseractEngine;
        let result = engine.perform_ocr("/tmp/test_ocr.png", "eng");
        assert!(result.is_ok(), "OCR should succeed: {:?}", result.err());
        let ocr_result = result.unwrap();
        assert!(
            ocr_result.text.contains("Hello"),
            "Should recognize 'Hello', got: {:?}",
            ocr_result.text
        );
        assert_eq!(ocr_result.language, "eng");
    }

    #[test]
    fn test_supported_languages() {
        let output = Command::new("tesseract")
            .arg("--list-langs")
            .output()
            .unwrap();
        let langs = String::from_utf8_lossy(&output.stdout);
        assert!(langs.contains("eng"), "English should be supported");
    }
}
