//! Image upload providers used to obtain a public URL for a screenshot before
//! performing a reverse-image / lens search.

pub mod custom;
pub mod zeroxzero;

use crate::error::SearchError;

/// A provider capable of uploading a local image and returning a public URL.
pub trait ImageUploader {
    /// Upload the image at `image_path` and return the resulting public URL.
    fn upload(&self, image_path: &str) -> Result<String, SearchError>;

    /// The provider's short name (e.g. `"0x0"`, `"custom"`).
    fn name(&self) -> &str;
}

/// Construct an [`ImageUploader`] for the given provider `name`.
///
/// * `"0x0"` -> [`zeroxzero::ZeroXZeroUploader`]
/// * `"custom"` -> [`custom::CustomUploader`] (requires `url`)
///
/// An empty or unknown `name` yields a [`SearchError::Config`].
pub fn create_uploader(
    name: &str,
    url: Option<&str>,
) -> Result<Box<dyn ImageUploader>, SearchError> {
    match name {
        "0x0" => Ok(Box::new(zeroxzero::ZeroXZeroUploader::new())),
        "custom" => {
            let endpoint = url.ok_or_else(|| {
                SearchError::Config(
                    "Custom upload provider requires image_upload_url in config".to_string(),
                )
            })?;
            Ok(Box::new(custom::CustomUploader::new(endpoint.to_string())))
        }
        "" => Err(SearchError::Config(
            "No upload provider configured. Set image_upload_provider in config.".to_string(),
        )),
        _ => Err(SearchError::Config(format!(
            "Unknown upload provider: {}. Supported: 0x0, custom",
            name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_uploader_0x0() {
        let uploader = create_uploader("0x0", None).expect("0x0 uploader should be created");
        assert_eq!(uploader.name(), "0x0");
    }

    #[test]
    fn test_create_uploader_custom() {
        let uploader = create_uploader("custom", Some("https://example.com/upload"))
            .expect("custom uploader should be created");
        assert_eq!(uploader.name(), "custom");
    }

    #[test]
    fn test_create_uploader_custom_without_url_errors() {
        assert!(create_uploader("custom", None).is_err());
    }

    #[test]
    fn test_create_uploader_empty_errors() {
        assert!(create_uploader("", None).is_err());
    }

    #[test]
    fn test_create_uploader_unknown_errors() {
        assert!(create_uploader("bogus", None).is_err());
    }
}
