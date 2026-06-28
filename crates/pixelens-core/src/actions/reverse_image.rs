use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::actions::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub struct ReverseImageHandler;

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pixelens")
}

pub fn ensure_cache_dir() -> Result<PathBuf, PixelensError> {
    let dir = cache_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn save_to_cache(image_path: &str) -> Result<PathBuf, PixelensError> {
    let cache_dir = ensure_cache_dir()?;
    let filename = format!(
        "reverse_search_{}.png",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let dest = cache_dir.join(&filename);
    fs::copy(image_path, &dest)?;
    Ok(dest)
}

pub fn open_reverse_image_search(image_path: &str) -> Result<(), PixelensError> {
    let _ = image_path;
    let status = Command::new("xdg-open")
        .arg("https://lens.google.com/uploadbyurl")
        .status()
        .map_err(|e| {
            PixelensError::Config(format!(
                "Failed to open browser: {}. Is xdg-open installed?",
                e
            ))
        })?;

    if !status.success() {
        return Err(PixelensError::Config(format!(
            "Browser exited with status: {}",
            status
        )));
    }

    Ok(())
}

impl ActionHandler for ReverseImageHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        let image_path = payload.image_path.as_ref().ok_or_else(|| {
            PixelensError::Config("No image provided for reverse search".to_string())
        })?;

        if !std::path::Path::new(image_path).exists() {
            return Err(PixelensError::Config(format!(
                "Image file not found: {}",
                image_path
            )));
        }

        let saved_path = save_to_cache(image_path)?;
        open_reverse_image_search(saved_path.to_str().unwrap_or(""))?;

        Ok(saved_path.to_string_lossy().to_string())
    }

    fn action_type(&self) -> ActionType {
        ActionType::ReverseImageSearch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir() {
        let dir = cache_dir();
        assert!(dir.to_string_lossy().contains("pixelens"));
    }

    #[test]
    fn test_save_to_cache() {
        let tmp = std::env::temp_dir().join("pixelens_test_input.png");
        fs::write(&tmp, b"test image data").unwrap();

        let result = save_to_cache(tmp.to_str().unwrap());
        assert!(result.is_ok());
        let saved = result.unwrap();
        assert!(saved.exists());
        assert!(saved.to_string_lossy().contains("reverse_search_"));

        fs::remove_file(&tmp).ok();
        fs::remove_file(&saved).ok();
    }

    #[test]
    fn test_save_to_cache_missing_file() {
        let result = save_to_cache("/tmp/nonexistent_file_abc123.png");
        assert!(result.is_err());
    }

    #[test]
    fn test_action_type() {
        let handler = ReverseImageHandler;
        assert!(matches!(
            handler.action_type(),
            ActionType::ReverseImageSearch
        ));
    }

    #[test]
    fn test_execute_missing_image_path() {
        let handler = ReverseImageHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: None,
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_missing_file() {
        let handler = ReverseImageHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: Some("/tmp/nonexistent_file_xyz.png".to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_no_ocr() {
        let handler = ReverseImageHandler;
        let payload = ActionPayload {
            text: String::new(),
            image_path: None,
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(!err.contains("OCR"), "Should not involve OCR");
    }
}
