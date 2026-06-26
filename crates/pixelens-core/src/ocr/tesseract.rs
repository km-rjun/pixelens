use std::process::Command;

use crate::error::OcrError;
use crate::ocr::OcrEngine;
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
    #[ignore = "requires tesseract and creates temp image"]
    fn test_perform_ocr() {
        let tmp_dir = std::env::temp_dir();
        let img_path = tmp_dir.join("pixelens_test_ocr.png");

        // Create a minimal valid PNG (1x1 white pixel)
        let png_header: Vec<u8> = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // PNG signature
            0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // 8-bit RGB
            0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00, // compressed data
            0x00, 0x00, 0x02, 0x00, 0x01, 0xe2, 0x21, 0xbc, 0x33, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4e, // IEND chunk
            0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        std::fs::write(&img_path, &png_header).unwrap();

        let engine = TesseractEngine;
        let result = engine.perform_ocr(img_path.to_str().unwrap(), "eng");
        assert!(result.is_ok(), "OCR should succeed: {:?}", result.err());

        // Clean up
        let _ = std::fs::remove_file(&img_path);
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
