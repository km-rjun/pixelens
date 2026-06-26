use std::str::FromStr;

use crate::actions::get_handler;
use crate::types::{ActionPayload, ActionType};

#[tauri::command]
pub fn execute_action(
    action: String,
    text: String,
    image_path: Option<String>,
) -> Result<String, String> {
    let action_type = ActionType::from_str(&action)?;

    let handler = get_handler(&action_type).map_err(|e| e.to_string())?;

    let payload = ActionPayload { text, image_path };
    handler.execute(&payload).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_actions() -> Vec<String> {
    crate::actions::get_action_names()
}
