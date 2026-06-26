use pixelens_common::{ActionType, CaptureRegion, OcrResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Ping,
    Status,
    Stop,
    CheckTools,
    GetConfig,
    Grab {
        search: bool,
        ai: Option<String>,
    },
    Ocr {
        image_path: String,
        language: String,
    },
    Ai {
        prompt: String,
        image_path: Option<String>,
    },
    Action {
        action: ActionType,
        text: String,
        image_path: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Pong,
    Stopped,
    Status {
        running: bool,
        capture_missing: Vec<String>,
        ocr_missing: Vec<String>,
    },
    ToolsStatus {
        capture_missing: Vec<String>,
        ocr_missing: Vec<String>,
    },
    Config {
        api_endpoint: String,
        model: String,
        ocr_language: String,
    },
    GrabResult {
        image_path: String,
        text: Option<String>,
        ai_response: Option<String>,
    },
    CaptureResult {
        image_path: String,
        region: CaptureRegion,
    },
    OcrResult(OcrResult),
    AiResult {
        content: String,
        model: String,
    },
    ActionResult(String),
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request::Ping;
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Request = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Request::Ping));
    }

    #[test]
    fn test_response_serialization() {
        let response = Response::Pong;
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Response::Pong));
    }

    #[test]
    fn test_grab_request() {
        let request = Request::Grab {
            search: false,
            ai: Some("What is this?".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Grab"));
        assert!(json.contains("What is this?"));
    }

    #[test]
    fn test_status_response() {
        let response = Response::Status {
            running: true,
            capture_missing: vec![],
            ocr_missing: vec![],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Status"));
        assert!(json.contains("true"));
    }
}
