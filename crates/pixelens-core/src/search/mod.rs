pub mod google_lens;

use crate::error::PixelensError;

pub trait ReverseSearchProvider {
    fn search_url(&self, public_image_url: &str) -> Result<String, PixelensError>;
    fn name(&self) -> &str;
}

pub fn create_search_provider(name: &str) -> Result<Box<dyn ReverseSearchProvider>, PixelensError> {
    match name {
        "google_lens" => Ok(Box::new(google_lens::GoogleLensProvider::new())),
        _ => Err(PixelensError::Config(format!(
            "Unknown search provider: {}. Supported: google_lens",
            name
        ))),
    }
}
