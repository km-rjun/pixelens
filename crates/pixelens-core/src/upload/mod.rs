pub mod zeroxzero;

use crate::error::PixelensError;

pub trait ImageUploader {
    fn upload(&self, image_path: &str) -> Result<String, PixelensError>;
    fn name(&self) -> &str;
}

pub fn create_uploader(name: &str) -> Result<Box<dyn ImageUploader>, PixelensError> {
    match name {
        "0x0" => Ok(Box::new(zeroxzero::ZeroXZeroUploader::new())),
        _ => Err(PixelensError::Config(format!(
            "Unknown upload provider: {}. Supported: 0x0",
            name
        ))),
    }
}
