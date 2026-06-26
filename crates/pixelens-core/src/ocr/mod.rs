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
    let lines: Vec<&str> = text.lines().collect();
    let mut result: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let is_blank = lines[i].trim().is_empty();

        if is_blank {
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
        } else {
            result.push(lines[i]);
            i += 1;
        }
    }

    result.join("\n")
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
    fn test_clean_single_blank_noise() {
        let input = "Join us now to add more\n\nknowledge and share it with the world!";
        assert_eq!(
            clean_ocr_output(input),
            "Join us now to add more\nknowledge and share it with the world!"
        );
    }

    #[test]
    fn test_clean_preserves_paragraph_break() {
        let input = "Paragraph one.\n\n\n\nParagraph two.";
        assert_eq!(clean_ocr_output(input), "Paragraph one.\nParagraph two.");
    }

    #[test]
    fn test_clean_heading_then_paragraph() {
        let input = "Title\n\nBody text here.";
        assert_eq!(clean_ocr_output(input), "Title\nBody text here.");
    }

    #[test]
    fn test_clean_empty_input() {
        assert_eq!(clean_ocr_output(""), "");
    }

    #[test]
    fn test_clean_multiple_lines_no_blanks() {
        let input = "Line 1\nLine 2\nLine 3";
        assert_eq!(clean_ocr_output(input), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_clean_many_blanks_collapses() {
        let input = "A\n\n\n\n\n\n\n\nB";
        assert_eq!(clean_ocr_output(input), "A\nB");
    }

    #[test]
    fn test_clean_trailing_blanks() {
        let input = "Hello\n\n\n";
        assert_eq!(clean_ocr_output(input), "Hello");
    }

    #[test]
    fn test_clean_leading_blanks() {
        let input = "\n\nHello";
        assert_eq!(clean_ocr_output(input), "Hello");
    }

    #[test]
    fn test_clean_ocr_real_example() {
        let input = "Join us now to add more\n\n\nknowledge and share it with the world!";
        let cleaned = clean_ocr_output(input);
        assert_eq!(
            cleaned,
            "Join us now to add more\nknowledge and share it with the world!"
        );
        assert!(
            !cleaned.contains("\n\n"),
            "Should not have double blank lines"
        );
    }
}
