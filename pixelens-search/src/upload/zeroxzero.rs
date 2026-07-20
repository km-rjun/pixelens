//! Uploader for the <https://0x0.st> pastebin-style file host.

use std::fs;

use crate::error::SearchError;
use crate::upload::ImageUploader;

/// Uploads images to <https://0x0.st>, which returns the public URL as the
/// plain-text response body.
#[derive(Default)]
pub struct ZeroXZeroUploader;

impl ZeroXZeroUploader {
    /// Create a new uploader.
    pub fn new() -> Self {
        Self
    }
}

impl ImageUploader for ZeroXZeroUploader {
    fn upload(&self, image_path: &str) -> Result<String, SearchError> {
        let data = fs::read(image_path)?;

        let boundary = format!("----PixelensBoundary{}", fastrand::u64(..));
        let filename = std::path::Path::new(image_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image.png");

        let mut body = Vec::new();
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
                filename
            )
            .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
        body.extend_from_slice(&data);
        body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

        let response = ureq::post("https://0x0.st")
            .set(
                "Content-Type",
                &format!("multipart/form-data; boundary={}", boundary),
            )
            .send_bytes(&body)
            .map_err(|e| SearchError::Network(e.to_string()))?;

        let result = response.into_string().map_err(|e| {
            SearchError::InvalidResponse(format!("Failed to read upload response: {}", e))
        })?;

        let url = result.trim().to_string();
        if url.is_empty() || url.contains("error") {
            return Err(SearchError::Upload(format!("Upload failed: {}", url)));
        }

        Ok(url)
    }

    fn name(&self) -> &str {
        "0x0"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uploader_name() {
        let uploader = ZeroXZeroUploader::new();
        assert_eq!(uploader.name(), "0x0");
    }
}
