use std::process::Command;

use super::OcrEngine;
use crate::error::OcrError;
use crate::types::OcrResult;

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
}
