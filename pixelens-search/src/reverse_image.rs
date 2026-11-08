//! Reverse-image-search orchestration (cache + upload + search).
//!
//! This module carries over the pure logic from `origin/main`
//! `crates/pixelens-core/src/actions/reverse_image.rs`. It is intentionally
//! **headless-safe**: it never spawns `xdg-open`, `wl-copy`, or any external
//! process. It returns URLs / status strings, and the `pixelens-daemon`
//! decides how to open them.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use pixelens_config::Config;

use crate::error::SearchError;
use crate::google_lens::GoogleLensProvider;
use crate::upload::{create_uploader, ImageUploader};

/// Return the directory used to stage screenshots before upload/search.
///
/// Falls back to the current directory if no user cache dir is available.
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pixelens")
}

/// Ensure the cache directory exists.
pub fn ensure_cache_dir() -> Result<PathBuf, SearchError> {
    let dir = cache_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Copy `image_path` into the cache dir under a unique name and return the
/// path of the staged copy.
pub fn save_to_cache(image_path: &str) -> Result<PathBuf, SearchError> {
    let source = PathBuf::from(image_path);
    if !source.exists() {
        return Err(SearchError::Config(format!(
            "image file does not exist: {image_path}"
        )));
    }

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let dir = ensure_cache_dir()?;
    let dest = dir.join(format!("reverse_search_{millis}.png"));
    fs::copy(&source, &dest)?;
    Ok(dest)
}

/// Orchestrates reverse-image search: cache the image, optionally upload it,
/// and build the Google Lens search URL. Headless-safe.
pub struct ReverseImageSearcher {
    config: Config,
}

impl ReverseImageSearcher {
    /// Build a searcher from the loaded configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the full reverse-image-search flow for `image_path`.
    ///
    /// Returns a human-readable status string. When automatic upload is not
    /// configured, the message explains that manual upload is required and
    /// points at the Google Lens upload page. No external process is spawned.
    pub fn run(&self, image_path: &str) -> Result<String, SearchError> {
        if image_path.trim().is_empty() {
            return Err(SearchError::Config(
                "no image path provided for reverse image search".to_string(),
            ));
        }

        let saved = save_to_cache(image_path)?;
        let saved_str = saved.to_string_lossy().to_string();

        if self.config.upload.provider.trim().is_empty() {
            // Automatic upload is disabled. The daemon may open this page for
            // the user; we never spawn a browser ourselves.
            return Ok(format!(
                "Image saved: {saved_str}\n\
                 Opened Google Lens upload page (https://lens.google.com/uploadbyurl).\n\
                 Automatic upload is not enabled; please drop the image manually."
            ));
        }

        let endpoint = if self.config.upload.endpoint.trim().is_empty() {
            None
        } else {
            Some(self.config.upload.endpoint.as_str())
        };

        let uploader = create_uploader(&self.config.upload.provider, endpoint)?;
        let public_url = uploader.upload(&saved_str)?;

        let search_url = GoogleLensProvider::new().reverse_search_url(&public_url)?;

        Ok(format!(
            "Image uploaded to: {public_url}\n\
             Search URL: {search_url}\n\
             Opened in browser."
        ))
    }
}

/// Standalone helper that uploads `image_path` (already cached caller-side or
/// directly) via `uploader`, builds the Google Lens search URL via
/// `search_provider`, and returns the search URL string.
///
/// `browser_note` is an optional message appended to the returned URL string;
/// it documents how the URL will be opened without this function doing so.
pub fn execute_reverse_image_search(
    image_path: &str,
    uploader: &dyn ImageUploader,
    search_provider: &GoogleLensProvider,
    browser_note: &str,
) -> Result<String, SearchError> {
    if image_path.trim().is_empty() {
        return Err(SearchError::Config(
            "no image path provided for reverse image search".to_string(),
        ));
    }

    let public_url = uploader.upload(image_path)?;
    let search_url = search_provider.reverse_search_url(&public_url)?;

    if browser_note.is_empty() {
        Ok(search_url)
    } else {
        Ok(format!("{search_url}\n{browser_note}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Minimal in-memory uploader that returns a fixed public URL.
    struct MockUploader;

    impl ImageUploader for MockUploader {
        fn upload(&self, _image_path: &str) -> Result<String, SearchError> {
            Ok("https://example.com/i.png".to_string())
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[test]
    fn save_to_cache_writes_file() {
        let dir = std::env::temp_dir();
        let src = dir.join(format!("pixelens_test_src_{}.png", std::process::id()));
        {
            let mut f = fs::File::create(&src).unwrap();
            f.write_all(b"fake-png-bytes").unwrap();
        } // ensure handle is dropped

        let cached = save_to_cache(src.to_str().unwrap()).expect("save_to_cache ok");
        assert!(cached.exists(), "cached file should exist");
        assert!(
            cached
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("reverse_search_"),
            "filename should be prefixed"
        );

        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&cached);
    }

    #[test]
    fn run_with_empty_provider_returns_saved_message() {
        let dir = std::env::temp_dir();
        let src = dir.join("pixelens_test_run.png");
        {
            let mut f = fs::File::create(&src).unwrap();
            f.write_all(b"fake-png-bytes").unwrap();
        }

        let mut config = Config::default();
        config.upload.provider = String::new(); // empty -> manual upload path

        let searcher = ReverseImageSearcher::new(config);
        let out = searcher
            .run(src.to_str().unwrap())
            .expect("run should succeed with empty provider");

        assert!(
            out.contains("Image saved"),
            "message should report the image was saved: {out}"
        );
        assert!(
            !out.contains("Search URL"),
            "manual path must not produce a search URL"
        );

        let _ = fs::remove_file(&src);
    }

    #[test]
    fn execute_builds_lens_url() {
        let src = std::env::temp_dir().join("pixelens_test_exec.png");
        {
            let mut f = fs::File::create(&src).unwrap();
            f.write_all(b"fake-png-bytes").unwrap();
        }

        let uploader = MockUploader;
        let provider = GoogleLensProvider::new();
        let result = execute_reverse_image_search(src.to_str().unwrap(), &uploader, &provider, "")
            .expect("execute ok");

        assert!(
            result.contains("lens.google.com"),
            "search URL should point at Google Lens: {result}"
        );

        let _ = fs::remove_file(&src);
    }

    #[test]
    fn run_missing_path_errors() {
        let config = Config::default();
        let searcher = ReverseImageSearcher::new(config);
        let err = searcher.run("").unwrap_err();
        matches!(err, SearchError::Config(_));
    }
}
