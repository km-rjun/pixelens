use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::actions::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub trait ClipboardCopier {
    fn copy_image(&self, path: &str) -> Result<(), String>;
}

pub struct WlCopyClipboard;

impl ClipboardCopier for WlCopyClipboard {
    fn copy_image(&self, path: &str) -> Result<(), String> {
        let status = Command::new("wl-copy")
            .args(["--type", "image/png"])
            .arg(path)
            .status()
            .map_err(|e| format!("Failed to run wl-copy: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("wl-copy exited with status: {}", status))
        }
    }
}

pub trait BrowserOpener {
    fn open(&self, url: &str) -> Result<(), String>;
}

pub struct XdgBrowserOpener;

impl BrowserOpener for XdgBrowserOpener {
    fn open(&self, url: &str) -> Result<(), String> {
        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("xdg-open exited with status: {}", status))
        }
    }
}

pub struct ReverseImageHandler<
    C: ClipboardCopier = WlCopyClipboard,
    B: BrowserOpener = XdgBrowserOpener,
> {
    clipboard: C,
    browser: B,
}

impl<C: ClipboardCopier, B: BrowserOpener> ReverseImageHandler<C, B> {
    pub fn new(clipboard: C, browser: B) -> Self {
        Self { clipboard, browser }
    }
}

impl Default for ReverseImageHandler<WlCopyClipboard, XdgBrowserOpener> {
    fn default() -> Self {
        Self {
            clipboard: WlCopyClipboard,
            browser: XdgBrowserOpener,
        }
    }
}

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

impl<C: ClipboardCopier, B: BrowserOpener> ActionHandler for ReverseImageHandler<C, B> {
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
        let saved_str = saved_path.to_string_lossy().to_string();

        let _ = self.clipboard.copy_image(&saved_str);

        self.browser
            .open("https://lens.google.com/uploadbyurl")
            .map_err(|e| {
                log::warn!("Browser open failed: {}", e);
                PixelensError::Config(format!("Failed to open browser: {}", e))
            })?;

        Ok(format!(
            "Image saved: {}\nOpened Google Lens upload page.\nAutomatic upload is not enabled. Upload the saved image manually or configure an upload provider later.",
            saved_str
        ))
    }

    fn action_type(&self) -> ActionType {
        ActionType::ReverseImageSearch
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    struct MockBrowser {
        last_url: Rc<RefCell<Option<String>>>,
        should_fail: bool,
    }

    impl MockBrowser {
        fn new(should_fail: bool) -> Self {
            Self {
                last_url: Rc::new(RefCell::new(None)),
                should_fail,
            }
        }

        fn shared_url(&self) -> Rc<RefCell<Option<String>>> {
            self.last_url.clone()
        }
    }

    impl BrowserOpener for MockBrowser {
        fn open(&self, url: &str) -> Result<(), String> {
            *self.last_url.borrow_mut() = Some(url.to_string());
            if self.should_fail {
                Err("mock browser failed".to_string())
            } else {
                Ok(())
            }
        }
    }

    struct MockClipboard {
        should_fail: bool,
    }

    impl MockClipboard {
        fn new(should_fail: bool) -> Self {
            Self { should_fail }
        }
    }

    impl ClipboardCopier for MockClipboard {
        fn copy_image(&self, _path: &str) -> Result<(), String> {
            if self.should_fail {
                Err("mock clipboard failed".to_string())
            } else {
                Ok(())
            }
        }
    }

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
        let browser = MockBrowser::new(false);
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);
        assert!(matches!(
            handler.action_type(),
            ActionType::ReverseImageSearch
        ));
    }

    #[test]
    fn test_execute_missing_image_path() {
        let browser = MockBrowser::new(false);
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);
        let payload = ActionPayload {
            text: String::new(),
            image_path: None,
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_missing_file() {
        let browser = MockBrowser::new(false);
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);
        let payload = ActionPayload {
            text: String::new(),
            image_path: Some("/tmp/nonexistent_file_xyz.png".to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_no_ocr() {
        let browser = MockBrowser::new(false);
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);
        let payload = ActionPayload {
            text: String::new(),
            image_path: None,
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(!err.contains("OCR"), "Should not involve OCR");
    }

    #[test]
    fn test_saves_local_png() {
        let tmp = std::env::temp_dir().join("pixelens_test_save.png");
        fs::write(&tmp, b"test image data").unwrap();

        let browser = MockBrowser::new(false);
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);

        let payload = ActionPayload {
            text: String::new(),
            image_path: Some(tmp.to_str().unwrap().to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("Image saved:"));
        assert!(msg.contains("reverse_search_"));

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_opens_google_lens_upload_page() {
        let tmp = std::env::temp_dir().join("pixelens_test_lens.png");
        fs::write(&tmp, b"test image data").unwrap();

        let browser = MockBrowser::new(false);
        let url_tracker = browser.shared_url();
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);

        let payload = ActionPayload {
            text: String::new(),
            image_path: Some(tmp.to_str().unwrap().to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_ok());
        let url = url_tracker.borrow();
        assert!(url.as_ref().unwrap().contains("lens.google.com"));
        assert!(url.as_ref().unwrap().contains("uploadbyurl"));

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_no_upload_made() {
        let tmp = std::env::temp_dir().join("pixelens_test_no_upload.png");
        fs::write(&tmp, b"test image data").unwrap();

        let browser = MockBrowser::new(false);
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);

        let payload = ActionPayload {
            text: String::new(),
            image_path: Some(tmp.to_str().unwrap().to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("Automatic upload is not enabled"));

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_browser_failure_returns_error() {
        let tmp = std::env::temp_dir().join("pixelens_test_browser_err.png");
        fs::write(&tmp, b"test image data").unwrap();

        let browser = MockBrowser::new(true);
        let clipboard = MockClipboard::new(false);
        let handler = ReverseImageHandler::new(clipboard, browser);

        let payload = ActionPayload {
            text: String::new(),
            image_path: Some(tmp.to_str().unwrap().to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_err());

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_clipboard_failure_still_saves() {
        let tmp = std::env::temp_dir().join("pixelens_test_clip_fail.png");
        fs::write(&tmp, b"test image data").unwrap();

        let browser = MockBrowser::new(false);
        let clipboard = MockClipboard::new(true);
        let handler = ReverseImageHandler::new(clipboard, browser);

        let payload = ActionPayload {
            text: String::new(),
            image_path: Some(tmp.to_str().unwrap().to_string()),
        };
        let result = handler.execute(&payload);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("Image saved:"));

        fs::remove_file(&tmp).ok();
    }
}
