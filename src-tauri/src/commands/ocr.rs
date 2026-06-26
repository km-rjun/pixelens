use crate::ocr::create_engine;
use crate::types::OcrResult;

#[tauri::command]
pub fn perform_ocr(image_path: String, language: Option<String>) -> Result<OcrResult, String> {
    let engine = create_engine()?;
    let lang = language.unwrap_or_else(|| "eng".to_string());
    engine
        .perform_ocr(&image_path, &lang)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_ocr_tools() -> Vec<String> {
    crate::ocr::check_tools()
}
