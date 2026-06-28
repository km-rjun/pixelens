pub mod custom;
pub mod zeroxzero;

use crate::error::PixelensError;

pub trait ImageUploader {
    fn upload(&self, image_path: &str) -> Result<String, PixelensError>;
    fn name(&self) -> &str;
}

pub fn create_uploader(
    name: &str,
    url: Option<&str>,
) -> Result<Box<dyn ImageUploader>, PixelensError> {
    match name {
        "0x0" => Ok(Box::new(zeroxzero::ZeroXZeroUploader::new())),
        "custom" => {
            let endpoint = url.ok_or_else(|| {
                PixelensError::Config(
                    "Custom upload provider requires image_upload_url in config".to_string(),
                )
            })?;
            Ok(Box::new(custom::CustomUploader::new(endpoint.to_string())))
        }
        "" => Err(PixelensError::Config(
            "No upload provider configured. Set image_upload_provider in config.".to_string(),
        )),
        _ => Err(PixelensError::Config(format!(
            "Unknown upload provider: {}. Supported: 0x0, custom",
            name
        ))),
    }
}
