use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};
use super::ActionHandler;

pub struct CopyHandler;

impl ActionHandler for CopyHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError> {
        Ok(payload.text.clone())
    }

    fn action_type(&self) -> ActionType {
        ActionType::CopyToClipboard
    }
}
