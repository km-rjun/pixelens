use pixelens_common::{ActionType, CaptureRegion, OcrResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Capture,
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
    CheckTools,
    GetConfig,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
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
    ToolsStatus {
        capture_missing: Vec<String>,
        ocr_missing: Vec<String>,
    },
    Config {
        api_endpoint: String,
        model: String,
        ocr_language: String,
    },
    Pong,
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
    fn test_capture_request() {
        let request = Request::Capture;
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Capture"));
    }

    #[test]
    fn test_ocr_request() {
        let request = Request::Ocr {
            image_path: "/tmp/test.png".to_string(),
            language: "eng".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Ocr"));
        assert!(json.contains("test.png"));
    }
}
