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
        format!("{},{} {}x{}", self.x, self.y, self.width, self.height)
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
    fn test_to_grim_geometry() {
        let region = CaptureRegion {
            x: 424,
            y: 220,
            width: 615,
            height: 227,
        };
        assert_eq!(region.to_grim_geometry(), "424,220 615x227");
    }

    #[test]
    fn test_to_grim_geometry_negative() {
        let region = CaptureRegion {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        assert_eq!(region.to_grim_geometry(), "-1920,0 1920x1080");
    }

    #[test]
    fn test_roundtrip_slurp_to_grim() {
        let slurp_output = "424,220 615x227";
        let parts: Vec<&str> = slurp_output.split_whitespace().collect();
        let xy: Vec<&str> = parts[0].split(',').collect();
        let wh: Vec<&str> = parts[1].split('x').collect();

        let region = CaptureRegion {
            x: xy[0].parse().unwrap(),
            y: xy[1].parse().unwrap(),
            width: wh[0].parse().unwrap(),
            height: wh[1].parse().unwrap(),
        };

        assert_eq!(region.to_grim_geometry(), "424,220 615x227");
    }
}
