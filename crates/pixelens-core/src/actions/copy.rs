use std::process::Command;

use crate::actions::ActionHandler;
use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub struct CopyHandler;

impl ActionHandler for CopyHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        let mut child = Command::new("wl-copy")
            .arg("--")
            .arg(&payload.text)
            .spawn()
            .map_err(|e| {
                PixelensError::Config(format!(
                    "Failed to run wl-copy: {}. Is wl-clipboard installed?",
                    e
                ))
            })?;

        child
            .wait()
            .map_err(|e| PixelensError::Config(format!("wl-copy failed: {}", e)))?;

        Ok(payload.text.clone())
    }

    fn action_type(&self) -> ActionType {
        ActionType::CopyToClipboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type() {
        let handler = CopyHandler;
        assert!(matches!(handler.action_type(), ActionType::CopyToClipboard));
    }
}
