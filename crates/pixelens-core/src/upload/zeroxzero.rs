use std::fs;

use crate::error::PixelensError;
use crate::upload::ImageUploader;

#[derive(Default)]
pub struct ZeroXZeroUploader;

impl ZeroXZeroUploader {
    pub fn new() -> Self {
        Self
    }
}

impl ImageUploader for ZeroXZeroUploader {
    fn upload(&self, image_path: &str) -> Result<String, PixelensError> {
        let data = fs::read(image_path)
            .map_err(|e| PixelensError::Config(format!("Failed to read image: {}", e)))?;

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
            .header(
                "Content-Type",
                &format!("multipart/form-data; boundary={}", boundary),
            )
            .send(&body)
            .map_err(|e| PixelensError::Config(format!("Upload request failed: {}", e)))?;

        let mut result = String::new();
        response
            .into_body()
            .read_to_string()
            .map_err(|e| PixelensError::Config(format!("Failed to read upload response: {}", e)))?
            .chars()
            .for_each(|c| result.push(c));

        let url = result.trim().to_string();
        if url.is_empty() || url.contains("error") {
            return Err(PixelensError::Config(format!("Upload failed: {}", url)));
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
