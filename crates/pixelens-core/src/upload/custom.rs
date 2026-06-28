use crate::error::PixelensError;
use crate::upload::ImageUploader;

pub struct CustomUploader {
    endpoint: String,
}

impl CustomUploader {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl ImageUploader for CustomUploader {
    fn upload(&self, image_path: &str) -> Result<String, PixelensError> {
        let data = std::fs::read(image_path)
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

        let response = ureq::post(&self.endpoint)
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

        let parsed: serde_json::Value = serde_json::from_str(&result)
            .map_err(|e| PixelensError::Config(format!("Invalid JSON response: {}", e)))?;

        let url = parsed
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                PixelensError::Config(format!("No 'url' field in response: {}", result))
            })?;

        if url.starts_with("file://") {
            return Err(PixelensError::Config(
                "Provider returned a local file URL. This is not supported.".to_string(),
            ));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(PixelensError::Config(format!(
                "Provider returned an invalid URL: {}",
                url
            )));
        }

        Ok(url)
    }

    fn name(&self) -> &str {
        "custom"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uploader_name() {
        let uploader = CustomUploader::new("https://example.com/upload".to_string());
        assert_eq!(uploader.name(), "custom");
    }

    #[test]
    fn test_uploader_stores_endpoint() {
        let endpoint = "https://my-upload.example.com/api".to_string();
        let uploader = CustomUploader::new(endpoint.clone());
        assert_eq!(uploader.endpoint, endpoint);
    }
}
