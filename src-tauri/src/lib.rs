mod crypto;
mod db;
mod fs;
mod process;
mod shop;

use db::{
    create_account, get_account, get_all_accounts, get_settings, initialize_database, is_current_data_available,
    update_account, update_settings, CreateAccountData, NewAccount, Settings, UpdateAccount,
    UpdateSettings,
};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_app_dir() -> Result<String, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?;
    Ok(exe_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn get_default_riot_client_service_path() -> Result<String, String> {
    db::init::get_default_riot_client_service_path()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn get_default_riot_client_data_path() -> Result<String, String> {
    db::init::get_default_riot_client_data_path()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn get_riot_client_status() -> bool {
    process::check_riot_client_running()
}

#[tauri::command]
fn get_valorant_status() -> bool {
    process::check_valorant_running()
}

#[tauri::command]
fn kill_riot_client() -> Result<(), String> {
    process::kill_riot_client()
}

#[tauri::command]
fn launch_riot_client() -> Result<(), String> {
    process::launch_riot_client()
}

#[tauri::command]
fn get_app_settings() -> Result<Settings, String> {
    get_settings().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_app_settings(settings: UpdateSettings) -> Result<Settings, String> {
    update_settings(settings)
}

#[tauri::command]
fn add_account(account: NewAccount) -> Result<db::models::Account, String> {
    let use_current_data = account.use_current_data;
    let data = CreateAccountData {
        riot_id: account.riot_id,
        tagline: account.tagline,
        username: account.username,
        password: account.password,
        rank: account.rank,
        use_current_data,
    };

    let created = create_account(data)?;

    if use_current_data {
        log::info!("Auto-selecting account {} after current data import", created.id);
        perform_account_switch(Some(created.id))?;
    }

    Ok(created)
}

#[tauri::command]
fn list_accounts() -> Result<Vec<db::models::Account>, String> {
    get_all_accounts()
}

#[tauri::command]
fn edit_account(account: UpdateAccount) -> Result<db::models::Account, String> {
    update_account(account)
}

#[tauri::command]
fn check_current_data_available() -> Result<bool, String> {
    is_current_data_available()
}

#[tauri::command]
fn mark_launched() -> Result<(), String> {
    let conn = db::init::get_connection(None)?;
    conn.execute("UPDATE settings SET launched = 1 WHERE id = 1", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn perform_account_switch(account_id: Option<i64>) -> Result<(), String> {
    let settings = get_settings()?;

    let riot_data_path = match settings.riot_client_data_path {
        Some(path) => PathBuf::from(path),
        None => db::init::get_default_riot_client_data_path()?,
    };

    let account_data_path = match settings.account_data_path {
        Some(path) => PathBuf::from(path),
        None => db::init::get_default_account_data_path()?,
    };

    log::debug!("Riot data path: {}", riot_data_path.display());
    log::debug!("Account data path: {}", account_data_path.display());

    let target = if let Some(id) = account_id {
        let account = get_account(id)?;
        let data_folder = account
            .data_folder
            .ok_or("Account has no data directory assigned")?;
        log::info!("Switching to account {} ({})", id, data_folder);
        account_data_path.join(data_folder)
    } else {
        log::info!("Switching to unselected state");
        account_data_path.join("_unselected")
    };

    log::debug!("Target directory: {}", target.display());

    if !target.exists() {
        log::info!("Creating target directory: {}", target.display());
        fs::create_dir_with_marker(&target)?;
    }

    // Force cleanup of any existing path (junction, directory, or broken link)
    // Use Windows rmdir command for robust removal
    log::info!("Cleaning up riot data path if it exists: {}", riot_data_path.display());

    // First, try to detect and handle the existing path
    let path_exists = riot_data_path.exists() || fs::is_symlink(&riot_data_path).unwrap_or(false);

    if path_exists {
        if fs::is_symlink(&riot_data_path).unwrap_or(false) {
            log::info!("Detected junction point, removing");
            fs::remove_junction(&riot_data_path)?;
        } else if riot_data_path.is_dir() {
            log::info!("Detected regular directory, moving contents to target");
            fs::move_directory_contents(&riot_data_path, &target)?;
            std::fs::remove_dir(&riot_data_path)
                .map_err(|e| format!("Failed to remove directory: {}", e))?;
        }
    }

    // Force remove anything that might still exist (including broken junctions)
    // This is safe because we've already moved any real data
    let output = std::process::Command::new("cmd")
        .args(["/C", "rmdir", &riot_data_path.to_string_lossy()])
        .creation_flags(0x08000000)
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            log::info!("Successfully cleaned up path with rmdir");
        }
    }

    log::info!("Creating junction: {} -> {}", riot_data_path.display(), target.display());
    fs::create_junction(&riot_data_path, &target)?;

    let conn = db::init::get_connection(None)?;
    conn.execute(
        "UPDATE settings SET active_account_id = ?1 WHERE id = 1",
        [account_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn set_clipboard_text(text: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use winapi::um::winuser::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData, CF_UNICODETEXT};

    let wide: Vec<u16> = OsStr::new(text).encode_wide().chain(once(0)).collect();
    let byte_size = wide.len() * std::mem::size_of::<u16>();

    unsafe {
        let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_size);
        if hmem.is_null() {
            return Err("Failed to allocate clipboard memory".to_string());
        }
        let ptr = GlobalLock(hmem) as *mut u16;
        if ptr.is_null() {
            return Err("Failed to lock clipboard memory".to_string());
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        GlobalUnlock(hmem);

        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err("Failed to open clipboard".to_string());
        }
        EmptyClipboard();
        if SetClipboardData(CF_UNICODETEXT, hmem as _).is_null() {
            CloseClipboard();
            return Err("Failed to set clipboard data".to_string());
        }
        CloseClipboard();
    }

    Ok(())
}

#[tauri::command]
fn copy_account_password(account_id: i64) -> Result<(), String> {
    let account = get_account(account_id)?;
    if account.encrypted_password.is_empty() {
        return Err("No password stored".to_string());
    }
    let password = crypto::dpapi::unprotect_password(&account.encrypted_password)?;
    set_clipboard_text(&password)
}

/// Fetch the daily shop and night market for the given SSID and shard.
///
/// `ssid`  - Value of the `ssid` cookie from `auth.riotgames.com`.
/// `shard` - Region shard, e.g. `"ap"` (Asia-Pacific) or `"na"` (North America).
///
/// Note: SSID auto-detection from the active account's data directory will be
/// implemented in a future iteration.
#[tauri::command]
async fn get_shop(ssid: String, shard: String) -> Result<shop::Storefront, String> {
    shop::fetch_storefront(ssid, shard, None)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn switch_account(account_id: Option<i64>) -> Result<(), String> {
    log::info!("Starting account switch: {:?}", account_id);

    if process::check_riot_client_running() {
        log::warn!("Cannot switch accounts: Riot Client is running");
        return Err("Cannot switch accounts while Riot Client is running".to_string());
    }
    if process::check_valorant_running() {
        log::warn!("Cannot switch accounts: Valorant is running");
        return Err("Cannot switch accounts while Valorant is running".to_string());
    }

    perform_account_switch(account_id)?;

    log::info!("Account switch completed successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting valo-accounts application");

    if let Err(e) = initialize_database(None) {
        log::error!("Failed to initialize database: {}", e);
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    tauri::Builder::default()
        .setup(|app| {
            process::start_process_monitor(app.handle().clone());
            let window = app.get_webview_window("main")
                .ok_or("main window not found")?;
            window.show().map_err(|e| e.to_string())?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_dir,
            get_default_riot_client_service_path,
            get_default_riot_client_data_path,
            get_app_settings,
            update_app_settings,
            add_account,
            list_accounts,
            edit_account,
            check_current_data_available,
            mark_launched,
            switch_account,
            get_riot_client_status,
            kill_riot_client,
            launch_riot_client,
            get_valorant_status,
            copy_account_password,
            get_shop
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
