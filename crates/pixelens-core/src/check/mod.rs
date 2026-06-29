use std::path::PathBuf;

use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct CheckItem {
    pub status: CheckStatus,
    pub message: String,
}

impl CheckItem {
    pub fn ok(msg: &str) -> Self {
        Self {
            status: CheckStatus::Ok,
            message: msg.to_string(),
        }
    }

    pub fn warn(msg: &str) -> Self {
        Self {
            status: CheckStatus::Warn,
            message: msg.to_string(),
        }
    }

    pub fn fail(msg: &str) -> Self {
        Self {
            status: CheckStatus::Fail,
            message: msg.to_string(),
        }
    }
}

pub struct CheckResult {
    pub items: Vec<CheckItem>,
}

impl Default for CheckResult {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckResult {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: CheckItem) {
        self.items.push(item);
    }

    pub fn ok_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == CheckStatus::Ok)
            .count()
    }

    pub fn warn_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == CheckStatus::Warn)
            .count()
    }

    pub fn fail_count(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == CheckStatus::Fail)
            .count()
    }

    pub fn has_failures(&self) -> bool {
        self.fail_count() > 0
    }

    pub fn print(&self) {
        for item in &self.items {
            let prefix = match item.status {
                CheckStatus::Ok => "[ok]",
                CheckStatus::Warn => "[warn]",
                CheckStatus::Fail => "[fail]",
            };
            println!("{} {}", prefix, item.message);
        }
        println!(
            "\nPixelens check: {} ok, {} warnings, {} failures",
            self.ok_count(),
            self.warn_count(),
            self.fail_count()
        );
    }
}

fn check_tool(name: &str) -> CheckItem {
    if tool_exists(name) {
        CheckItem::ok(&format!("{} found", name))
    } else {
        CheckItem::fail(&format!("{} not found", name))
    }
}

fn tool_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_wayland_session() -> CheckItem {
    match std::env::var("WAYLAND_DISPLAY") {
        Ok(val) if !val.is_empty() => CheckItem::ok("Wayland session detected"),
        _ => CheckItem::warn("No WAYLAND_DISPLAY set, not in a Wayland session"),
    }
}

fn check_daemon_running() -> CheckItem {
    let socket_path = crate::ipc::server::IpcServer::socket_path();

    if socket_path.exists() {
        CheckItem::ok("daemon socket exists")
    } else {
        CheckItem::warn("daemon not running")
    }
}

fn check_config() -> Vec<CheckItem> {
    let mut items = Vec::new();
    let config = Config::load();
    let config_dir = Config::config_path().parent().unwrap().to_path_buf();

    if config_dir.exists() {
        items.push(CheckItem::ok("config directory exists"));
    } else {
        items.push(CheckItem::warn("config directory does not exist"));
    }

    if std::fs::metadata(&config_dir)
        .map(|m| !m.permissions().readonly())
        .unwrap_or(false)
    {
        items.push(CheckItem::ok("config directory is writable"));
    } else {
        items.push(CheckItem::warn("config directory not writable"));
    }

    let config_path = Config::config_path();
    if config_path.exists() {
        items.push(CheckItem::ok("config file readable"));
    } else {
        items.push(CheckItem::warn("config file does not exist"));
    }

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pixelens");
    if cache_dir.exists() {
        items.push(CheckItem::ok("cache directory exists"));
    } else {
        items.push(CheckItem::warn("cache directory does not exist"));
    }

    items.push(CheckItem::ok(&format!(
        "configured endpoint: {}",
        config.api_endpoint
    )));
    items.push(CheckItem::ok(&format!(
        "configured model: {}",
        config.model
    )));

    if config.api_key.is_empty() {
        items.push(CheckItem::warn("API key not configured"));
    } else {
        items.push(CheckItem::ok("API key is set"));
    }

    items.push(CheckItem::ok(&format!(
        "reverse image provider: {}",
        config.reverse_image_provider
    )));

    items
}

fn check_ocr_language() -> CheckItem {
    let config = Config::load();
    let output = std::process::Command::new("tesseract")
        .arg("--list-langs")
        .output();

    match output {
        Ok(o) => {
            let langs = String::from_utf8_lossy(&o.stdout);
            if langs.contains(&config.ocr_language) {
                CheckItem::ok(&format!("OCR language '{}' available", config.ocr_language))
            } else {
                CheckItem::warn(&format!(
                    "OCR language '{}' not found in tesseract",
                    config.ocr_language
                ))
            }
        }
        Err(_) => CheckItem::warn("Could not list tesseract languages"),
    }
}

fn check_layer_shell_support() -> CheckItem {
    #[cfg(feature = "layer-shell")]
    {
        CheckItem::ok("layer-shell feature enabled at build time")
    }
    #[cfg(not(feature = "layer-shell"))]
    {
        CheckItem::warn("built without layer-shell feature")
    }
}

pub fn run_check() -> CheckResult {
    let mut result = CheckResult::new();

    result.push(check_tool("pixelensd"));
    result.push(check_tool("grim"));
    result.push(check_tool("slurp"));
    result.push(check_tool("pixelensd"));
    result.push(check_tool("wl-copy"));
    result.push(check_tool("xdg-open"));

    result.push(check_wayland_session());
    result.push(check_daemon_running());

    result.items.extend(check_config());

    result.push(check_ocr_language());
    result.push(check_layer_shell_support());

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_counts() {
        let mut result = CheckResult::new();
        result.push(CheckItem::ok("test"));
        result.push(CheckItem::warn("test"));
        result.push(CheckItem::fail("test"));
        assert_eq!(result.ok_count(), 1);
        assert_eq!(result.warn_count(), 1);
        assert_eq!(result.fail_count(), 1);
        assert!(result.has_failures());
    }

    #[test]
    fn test_check_result_no_failures() {
        let mut result = CheckResult::new();
        result.push(CheckItem::ok("test"));
        result.push(CheckItem::warn("test"));
        assert!(!result.has_failures());
    }

    #[test]
    fn test_tool_detection() {
        let _ = check_tool("nonexistent_tool_xyz");
    }

    #[test]
    fn test_wayland_check() {
        let _ = check_wayland_session();
    }
}
