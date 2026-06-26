use crate::error::RateLimitKind;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProviderError {
    pub error: Option<ProviderErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderErrorDetail {
    pub message: Option<String>,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}

pub fn parse_429_response(body: &str) -> RateLimitKind {
    if let Ok(err) = serde_json::from_str::<ProviderError>(body) {
        if let Some(detail) = err.error {
            if let Some(code) = &detail.code {
                match code.as_str() {
                    "insufficient_quota" | "quota_exceeded" | "billing" => {
                        return RateLimitKind::QuotaExhausted;
                    }
                    "rate_limit_exceeded" | "requests" => {
                        return RateLimitKind::Temporary {
                            retry_after_secs: None,
                        };
                    }
                    _ => {}
                }
            }

            if let Some(msg) = &detail.message {
                let lower = msg.to_lowercase();
                if lower.contains("quota")
                    || lower.contains("billing")
                    || lower.contains("limit reached")
                    || lower.contains("insufficient")
                {
                    return RateLimitKind::QuotaExhausted;
                }
                if lower.contains("rate")
                    || lower.contains("too many requests")
                    || lower.contains("throttl")
                {
                    return RateLimitKind::Temporary {
                        retry_after_secs: None,
                    };
                }
            }
        }
    }

    RateLimitKind::Temporary {
        retry_after_secs: None,
    }
}

pub fn parse_retry_after(header_value: &str) -> Option<u64> {
    header_value.trim().parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rate_limit_code() {
        let body = r#"{"error": {"code": "rate_limit_exceeded", "message": "Too many requests"}}"#;
        let kind = parse_429_response(body);
        assert_eq!(
            kind,
            RateLimitKind::Temporary {
                retry_after_secs: None
            }
        );
    }

    #[test]
    fn test_parse_quota_code() {
        let body = r#"{"error": {"code": "insufficient_quota", "message": "Quota exceeded"}}"#;
        let kind = parse_429_response(body);
        assert_eq!(kind, RateLimitKind::QuotaExhausted);
    }

    #[test]
    fn test_parse_quota_message() {
        let body = r#"{"error": {"message": "You have exceeded your quota"}}"#;
        let kind = parse_429_response(body);
        assert_eq!(kind, RateLimitKind::QuotaExhausted);
    }

    #[test]
    fn test_parse_rate_message() {
        let body = r#"{"error": {"message": "Rate limit exceeded. Slow down."}}"#;
        let kind = parse_429_response(body);
        assert_eq!(
            kind,
            RateLimitKind::Temporary {
                retry_after_secs: None
            }
        );
    }

    #[test]
    fn test_parse_malformed_body() {
        let body = "not json";
        let kind = parse_429_response(body);
        assert_eq!(
            kind,
            RateLimitKind::Temporary {
                retry_after_secs: None
            }
        );
    }

    #[test]
    fn test_parse_empty_body() {
        let body = "";
        let kind = parse_429_response(body);
        assert_eq!(
            kind,
            RateLimitKind::Temporary {
                retry_after_secs: None
            }
        );
    }

    #[test]
    fn test_parse_retry_after() {
        assert_eq!(parse_retry_after("30"), Some(30));
        assert_eq!(parse_retry_after(" 60 "), Some(60));
        assert_eq!(parse_retry_after("abc"), None);
        assert_eq!(parse_retry_after(""), None);
    }
}
