use crate::db::{get_settings, update_settings, Settings, UpdateSettings};

#[tauri::command]
pub fn get_app_settings() -> Result<Settings, String> {
    get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_app_settings(settings: UpdateSettings) -> Result<Settings, String> {
    update_settings(settings)
}
