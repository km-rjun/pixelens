use std::process::Command;
use std::fs;

#[tauri::command]
pub fn capture_region() -> Result<String, String> {
    let output = Command::new("slurp")
        .output()
        .map_err(|e| format!("Failed to run slurp: {}", e))?;
    
    if !output.status.success() {
        return Err("Region selection cancelled".to_string());
    }
    
    let region = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    let output = Command::new("grim")
        .args(["-g", &region, "-"])
        .output()
        .map_err(|e| format!("Failed to run grim: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to capture screenshot".to_string());
    }
    
    let image_data = output.stdout;
    
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("pixelens_capture.png");
    fs::write(&file_path, &image_data).map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}
