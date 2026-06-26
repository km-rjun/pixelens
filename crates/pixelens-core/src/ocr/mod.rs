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
    let mut result = String::with_capacity(text.len());
    let mut prev_was_blank = false;

    for line in text.lines() {
        let is_blank = line.trim().is_empty();

        if is_blank {
            if !prev_was_blank {
                result.push('\n');
            }
            prev_was_blank = true;
        } else {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(line);
            result.push('\n');
            prev_was_blank = false;
        }
    }

    result.trim_end().to_string()
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

    #[test]
    fn test_clean_excessive_blank_lines() {
        let input = "Line 1\n\n\n\nLine 2";
        assert_eq!(clean_ocr_output(input), "Line 1\n\nLine 2");
    }

    #[test]
    fn test_clean_preserves_paragraph_breaks() {
        let input = "Paragraph one.\n\nParagraph two.";
        assert_eq!(clean_ocr_output(input), "Paragraph one.\n\nParagraph two.");
    }

    #[test]
    fn test_clean_heading_then_paragraph() {
        let input = "Title\n\nBody text here.";
        assert_eq!(clean_ocr_output(input), "Title\n\nBody text here.");
    }

    #[test]
    fn test_clean_empty_input() {
        assert_eq!(clean_ocr_output(""), "");
    }

    #[test]
    fn test_clean_whitespace_only() {
        assert_eq!(clean_ocr_output("   \n  \n  "), "");
    }

    #[test]
    fn test_clean_single_line() {
        assert_eq!(clean_ocr_output("Hello world"), "Hello world");
    }

    #[test]
    fn test_clean_trailing_newlines() {
        let input = "Line 1\nLine 2\n\n";
        assert_eq!(clean_ocr_output(input), "Line 1\nLine 2");
    }

    #[test]
    fn test_clean_many_blank_lines() {
        let input = "A\n\n\n\n\n\n\n\nB";
        assert_eq!(clean_ocr_output(input), "A\n\nB");
    }
}
