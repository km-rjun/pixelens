pub mod actions;
pub mod ai;
pub mod capture;
pub mod commands;
pub mod error;
pub mod ocr;
pub mod types;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::capture::capture_region,
            commands::capture::check_capture_tools,
            commands::ocr::perform_ocr,
            commands::ai::ask_ai,
            utils::config::get_config,
            utils::config::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
