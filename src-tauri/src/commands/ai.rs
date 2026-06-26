use crate::ai::create_provider;
use crate::types::{AiRequest, AiResponse};
use crate::utils::config::Config;

#[tauri::command]
pub fn ask_ai(
    prompt: String,
    image_path: Option<String>,
    api_endpoint: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
) -> Result<AiResponse, String> {
    let config = Config::load();
    let endpoint = api_endpoint.unwrap_or(config.api_endpoint);
    let key = api_key.unwrap_or(config.api_key);
    let mdl = model.unwrap_or(config.model);

    let provider = create_provider(&endpoint, &key, &mdl);
    let request = AiRequest {
        prompt,
        image_path,
    };
    provider.chat(&request).map_err(|e| e.to_string())
}
