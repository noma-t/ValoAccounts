#[tauri::command]
pub fn get_riot_client_status() -> bool {
    crate::process::check_riot_client_running()
}

#[tauri::command]
pub fn get_valorant_status() -> bool {
    crate::process::check_valorant_running()
}

#[tauri::command]
pub fn kill_riot_client() -> Result<(), String> {
    crate::process::kill_riot_client()
}

#[tauri::command]
pub fn launch_riot_client() -> Result<(), String> {
    crate::process::launch_riot_client()
}
