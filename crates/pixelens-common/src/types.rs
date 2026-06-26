use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl CaptureRegion {
    pub fn to_grim_geometry(&self) -> String {
        format!("{}x{}+{}+{}", self.width, self.height, self.x, self.y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResult {
    pub image_path: String,
    pub region: CaptureRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub prompt: String,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPayload {
    pub text: String,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    CopyToClipboard,
    SearchWeb,
    ReverseImageSearch,
    AskAi(String),
    Translate(String),
}

impl FromStr for ActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "copy" | "clipboard" => Ok(ActionType::CopyToClipboard),
            "search" | "google" => Ok(ActionType::SearchWeb),
            "reverse_image" | "reverse" => Ok(ActionType::ReverseImageSearch),
            "translate" => Ok(ActionType::Translate("English".to_string())),
            _ => Err(format!("Unknown action: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type_from_str() {
        assert!(matches!(
            "copy".parse::<ActionType>().unwrap(),
            ActionType::CopyToClipboard
        ));
        assert!(matches!(
            "search".parse::<ActionType>().unwrap(),
            ActionType::SearchWeb
        ));
        assert!(matches!(
            "reverse_image".parse::<ActionType>().unwrap(),
            ActionType::ReverseImageSearch
        ));
        assert!(matches!(
            "translate".parse::<ActionType>().unwrap(),
            ActionType::Translate(_)
        ));
        assert!("invalid".parse::<ActionType>().is_err());
    }

    #[test]
    fn test_capture_region_to_grim_geometry() {
        let region = CaptureRegion {
            x: 100,
            y: 200,
            width: 800,
            height: 600,
        };
        assert_eq!(region.to_grim_geometry(), "800x600+100+200");
    }
}
