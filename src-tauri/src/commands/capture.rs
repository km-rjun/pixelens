use crate::capture::detect_backend;
use crate::types::CaptureResult;

#[tauri::command]
pub fn capture_region() -> Result<CaptureResult, String> {
    let backend = detect_backend()?;
    let region = backend.select_region()?;
    backend.capture(&region).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_capture_tools() -> Vec<String> {
    crate::capture::check_tools()
}
