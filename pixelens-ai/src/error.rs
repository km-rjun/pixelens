use std::fmt;

/// Distinguishes transient 429s from hard quota exhaustion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitKind {
    /// Retry after the given seconds (or ASAP if `None`).
    Temporary { retry_after_secs: Option<u64> },
    /// Hard quota/billing limit — no point retrying.
    QuotaExhausted,
}

impl fmt::Display for RateLimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimitKind::Temporary { retry_after_secs } => match retry_after_secs {
                Some(secs) => write!(f, "rate limited, retry after {}s", secs),
                None => write!(f, "rate limited (transient)"),
            },
            RateLimitKind::QuotaExhausted => write!(f, "quota exhausted / billing limit reached"),
        }
    }
}

/// Errors raised by the AI client.
#[derive(Debug)]
pub enum AiError {
    /// The request could not be built or dispatched.
    RequestFailed(String),
    /// The provider returned a body we could not parse into a chat completion.
    InvalidResponse(String),
    /// Missing/invalid API key (401-equivalent).
    Unauthorized {
        endpoint: String,
        config_path: String,
    },
    /// Throttled by the provider.
    RateLimited { kind: RateLimitKind },
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiError::RequestFailed(msg) => write!(f, "AI request failed: {}", msg),
            AiError::InvalidResponse(msg) => write!(f, "invalid AI response: {}", msg),
            AiError::Unauthorized {
                endpoint,
                config_path,
            } => write!(
                f,
                "API key is missing or invalid for {}. Set [ai].api_key in {} (Ollama/llava does not require a key).",
                endpoint, config_path
            ),
            AiError::RateLimited { kind } => write!(f, "rate limited: {}", kind),
        }
    }
}

impl std::error::Error for AiError {}

impl PartialEq for AiError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AiError::RequestFailed(a), AiError::RequestFailed(b)) => a == b,
            (AiError::InvalidResponse(a), AiError::InvalidResponse(b)) => a == b,
            (
                AiError::Unauthorized {
                    endpoint: e1,
                    config_path: c1,
                },
                AiError::Unauthorized {
                    endpoint: e2,
                    config_path: c2,
                },
            ) => e1 == e2 && c1 == c2,
            (AiError::RateLimited { kind: k1 }, AiError::RateLimited { kind: k2 }) => k1 == k2,
            _ => false,
        }
    }
}
