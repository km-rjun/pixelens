pub mod tesseract;

use crate::error::OcrError;
use crate::types::OcrResult;

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

pub fn clean_ocr_output(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed: Vec<&str> = normalized.lines().map(|l| l.trim_end()).collect();
    let mut start = 0;
    let mut end = trimmed.len();

    while start < end && trimmed[start].is_empty() {
        start += 1;
    }
    while end > start && trimmed[end - 1].is_empty() {
        end -= 1;
    }

    trimmed[start..end].join("\n")
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
    fn test_empty_input() {
        assert_eq!(clean_ocr_output(""), "");
    }

    #[test]
    fn test_preserves_internal_blank_lines() {
        let input = "Line 1\n\nLine 2";
        assert_eq!(clean_ocr_output(input), "Line 1\n\nLine 2");
    }

    #[test]
    fn test_preserves_multiple_paragraphs() {
        let input = "Title\n\nBody.\n\nAnother paragraph.";
        assert_eq!(
            clean_ocr_output(input),
            "Title\n\nBody.\n\nAnother paragraph."
        );
    }

    #[test]
    fn test_normalizes_crlf() {
        let input = "Line 1\r\nLine 2\r\n";
        assert_eq!(clean_ocr_output(input), "Line 1\nLine 2");
    }

    #[test]
    fn test_strips_trailing_whitespace() {
        let input = "Hello   \nWorld  \n";
        assert_eq!(clean_ocr_output(input), "Hello\nWorld");
    }

    #[test]
    fn test_trims_leading_blank_lines() {
        let input = "\n\nHello";
        assert_eq!(clean_ocr_output(input), "Hello");
    }

    #[test]
    fn test_trims_trailing_blank_lines() {
        let input = "Hello\n\n\n";
        assert_eq!(clean_ocr_output(input), "Hello");
    }

    #[test]
    fn test_preserves_heading_paragraph_gap() {
        let input = "Heading\n\nFirst line of paragraph.";
        assert_eq!(
            clean_ocr_output(input),
            "Heading\n\nFirst line of paragraph."
        );
    }

    #[test]
    fn test_whitespace_only_input() {
        assert_eq!(clean_ocr_output("   \n  \n  "), "");
    }
}
