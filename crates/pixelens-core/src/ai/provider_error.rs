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

fn is_quota_error(detail: &ProviderErrorDetail) -> bool {
    if let Some(code) = &detail.code {
        if matches!(
            code.as_str(),
            "insufficient_quota" | "quota_exceeded" | "billing"
        ) {
            return true;
        }
    }
    if let Some(t) = &detail.error_type {
        if matches!(t.as_str(), "insufficient_quota" | "quota_exceeded") {
            return true;
        }
    }
    if let Some(msg) = &detail.message {
        let lower = msg.to_lowercase();
        if lower.contains("insufficient")
            || lower.contains("quota")
            || lower.contains("billing")
            || lower.contains("limit reached")
        {
            return true;
        }
    }
    false
}

fn is_rate_limit(detail: &ProviderErrorDetail) -> bool {
    if let Some(code) = &detail.code {
        if matches!(
            code.as_str(),
            "rate_limit_exceeded" | "requests" | "rate_limit"
        ) {
            return true;
        }
    }
    if let Some(t) = &detail.error_type {
        if matches!(t.as_str(), "rate_limit_exceeded" | "rate_limit") {
            return true;
        }
    }
    if let Some(msg) = &detail.message {
        let lower = msg.to_lowercase();
        if lower.contains("rate")
            || lower.contains("too many requests")
            || lower.contains("throttl")
        {
            return true;
        }
    }
    false
}

pub fn parse_429_response(body: &str) -> RateLimitKind {
    if let Ok(err) = serde_json::from_str::<ProviderError>(body) {
        if let Some(detail) = err.error {
            if is_quota_error(&detail) {
                return RateLimitKind::QuotaExhausted;
            }
            if is_rate_limit(&detail) {
                return RateLimitKind::Temporary {
                    retry_after_secs: None,
                };
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
    fn test_quota_by_type_field() {
        let body =
            r#"{"error": {"type": "insufficient_quota", "message": "You exceeded your quota"}}"#;
        let kind = parse_429_response(body);
        assert_eq!(kind, RateLimitKind::QuotaExhausted);
    }

    #[test]
    fn test_quota_by_code_field() {
        let body = r#"{"error": {"code": "insufficient_quota", "message": "Quota exceeded"}}"#;
        let kind = parse_429_response(body);
        assert_eq!(kind, RateLimitKind::QuotaExhausted);
    }

    #[test]
    fn test_quota_by_message() {
        let body = r#"{"error": {"message": "You have exceeded your quota"}}"#;
        let kind = parse_429_response(body);
        assert_eq!(kind, RateLimitKind::QuotaExhausted);
    }

    #[test]
    fn test_temporary_by_code() {
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
    fn test_temporary_by_type() {
        let body = r#"{"error": {"type": "rate_limit", "message": "Slow down"}}"#;
        let kind = parse_429_response(body);
        assert_eq!(
            kind,
            RateLimitKind::Temporary {
                retry_after_secs: None
            }
        );
    }

    #[test]
    fn test_temporary_by_message() {
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
    fn test_malformed_body() {
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
    fn test_empty_body() {
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
    fn test_retry_after() {
        assert_eq!(parse_retry_after("30"), Some(30));
        assert_eq!(parse_retry_after(" 60 "), Some(60));
        assert_eq!(parse_retry_after("abc"), None);
        assert_eq!(parse_retry_after(""), None);
    }
}
