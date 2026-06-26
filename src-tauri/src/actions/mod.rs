pub mod copy;
pub mod search;
pub mod translate;

use crate::error::PixelensError;
use crate::types::{ActionPayload, ActionType};

pub trait ActionHandler {
    fn execute(&self, payload: &ActionPayload) -> Result<String, PixelensError>;
    fn action_type(&self) -> ActionType;
}

pub fn get_handler(action: &ActionType) -> Box<dyn ActionHandler> {
    match action {
        ActionType::CopyToClipboard => Box::new(copy::CopyHandler),
        ActionType::SearchWeb => Box::new(search::SearchHandler),
        ActionType::AskAi(_) => unimplemented!("AI handler requires provider"),
        ActionType::Translate(ref lang) => Box::new(translate::TranslateHandler {
            target_lang: lang.clone(),
        }),
    }
}
