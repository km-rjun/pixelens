use crate::error::PixelensError;
use crate::search::ReverseSearchProvider;

#[derive(Default)]
pub struct GoogleLensProvider;

impl GoogleLensProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ReverseSearchProvider for GoogleLensProvider {
    fn search_url(&self, public_image_url: &str) -> Result<String, PixelensError> {
        if public_image_url.is_empty() {
            return Err(PixelensError::Config(
                "Empty image URL provided".to_string(),
            ));
        }

        if public_image_url.starts_with("file://") {
            return Err(PixelensError::Config(
                "Local file URLs are not supported. Upload the image first.".to_string(),
            ));
        }

        let encoded = urlencoding::encode(public_image_url);
        Ok(format!(
            "https://lens.google.com/uploadbyurl?url={}",
            encoded
        ))
    }

    fn name(&self) -> &str {
        "google_lens"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_url() {
        let provider = GoogleLensProvider::new();
        let url = provider
            .search_url("https://example.com/image.png")
            .unwrap();
        assert!(url.contains("lens.google.com"));
        assert!(url.contains("https%3A%2F%2Fexample.com%2Fimage.png"));
    }

    #[test]
    fn test_empty_url() {
        let provider = GoogleLensProvider::new();
        let result = provider.search_url("");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_url_rejected() {
        let provider = GoogleLensProvider::new();
        let result = provider.search_url("file:///tmp/image.png");
        assert!(result.is_err());
    }
}
