use std::process::Command;

#[tauri::command]
pub fn perform_ocr(image_path: String, language: Option<String>) -> Result<String, String> {
    let lang = language.unwrap_or_else(|| "eng".to_string());
    
    let output = Command::new("tesseract")
        .args([&image_path, "stdout", "-l", &lang])
        .output()
        .map_err(|e| format!("Failed to run tesseract: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Tesseract failed: {}", stderr));
    }
    
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}
