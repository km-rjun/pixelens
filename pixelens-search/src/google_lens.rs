//! Google Lens reverse-image-search provider for Pixelens.
//!
//! Ported from `origin/main` `crates/pixelens-core/src/search/google_lens.rs`
//! (Strategy C). The `ReverseSearchProvider` trait harness is dropped — this
//! struct exposes a plain `reverse_search_url` method returning
//! `crate::error::SearchError` on failure.

use crate::error::SearchError;
use urlencoding;

/// Reverse-image-search provider backed by Google Lens.
#[derive(Default)]
pub struct GoogleLensProvider;

impl GoogleLensProvider {
    /// Create a new provider instance.
    pub fn new() -> Self {
        Self
    }

    /// Build a Google Lens "upload by URL" link for a publicly hosted image.
    ///
    /// Returns an error if the URL is empty or points at a local `file://`
    /// resource (which Lens cannot fetch — the image must be uploaded first).
    pub fn reverse_search_url(&self, public_image_url: &str) -> Result<String, SearchError> {
        if public_image_url.is_empty() {
            return Err(SearchError::Config("Empty image URL provided".to_string()));
        }
        if public_image_url.starts_with("file://") {
            return Err(SearchError::Config(
                "Local file URLs are not supported. Upload the image first.".to_string(),
            ));
        }
        let encoded = urlencoding::encode(public_image_url);
        Ok(format!(
            "https://lens.google.com/uploadbyurl?url={}",
            encoded
        ))
    }

    /// Provider identifier.
    pub fn name(&self) -> &str {
        "google_lens"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_lens_url_with_encoded_image() {
        let provider = GoogleLensProvider::new();
        let url = provider
            .reverse_search_url("https://example.com/image.png")
            .expect("valid public url should succeed");
        assert!(
            url.contains("lens.google.com"),
            "missing lens host: {}",
            url
        );
        assert!(
            url.starts_with("https://lens.google.com/uploadbyurl?url="),
            "unexpected prefix: {}",
            url
        );
        // The slash in the image path should be percent-encoded.
        assert!(url.contains("%2F"), "expected encoded path, got: {}", url);
    }

    #[test]
    fn rejects_empty_url() {
        let provider = GoogleLensProvider::new();
        let result = provider.reverse_search_url("");
        match result {
            Err(SearchError::Config(msg)) => {
                assert_eq!(msg, "Empty image URL provided");
            }
            other => panic!("expected Config error, got {:?}", other),
        }
    }

    #[test]
    fn rejects_file_url() {
        let provider = GoogleLensProvider::new();
        let result = provider.reverse_search_url("file:///home/user/pic.png");
        match result {
            Err(SearchError::Config(msg)) => {
                assert_eq!(
                    msg,
                    "Local file URLs are not supported. Upload the image first."
                );
            }
            other => panic!("expected Config error, got {:?}", other),
        }
    }

    #[test]
    fn name_is_google_lens() {
        let provider = GoogleLensProvider::new();
        assert_eq!(provider.name(), "google_lens");
    }
}
