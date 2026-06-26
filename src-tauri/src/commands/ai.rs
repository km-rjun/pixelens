use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
struct Message {
    role: String,
    content: serde_json::Value,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[tauri::command]
pub fn ask_ai(
    prompt: String,
    image_path: Option<String>,
    api_endpoint: String,
    api_key: String,
    model: Option<String>,
) -> Result<String, String> {
    let model = model.unwrap_or_else(|| "gpt-4o".to_string());

    let mut content = serde_json::Value::Array(vec![]);

    if let Some(path) = image_path {
        let image_data = fs::read(&path).map_err(|e| format!("Failed to read image: {}", e))?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

        let image_content = serde_json::json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/png;base64,{}", base64_image)
            }
        });

        content.as_array_mut().unwrap().push(image_content);
    }

    let text_content = serde_json::json!({
        "type": "text",
        "text": prompt
    });
    content.as_array_mut().unwrap().push(text_content);

    let request = ChatRequest {
        model,
        messages: vec![Message {
            role: "user".to_string(),
            content,
        }],
    };

    let url = format!("{}/chat/completions", api_endpoint);
    let response = ureq::post(&url)
        .header("Authorization", &format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send_json(&request)
        .map_err(|e| format!("Failed to send request: {}", e))?;

    let body = response
        .into_body()
        .read_to_string()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let chat_response: ChatResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to deserialize response: {}", e))?;

    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from AI".to_string())
}
