use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

pub(crate) static DEMO_MODE: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn is_demo_mode() -> bool {
    #[cfg(debug_assertions)]
    {
        DEMO_MODE.load(Ordering::Relaxed)
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_app_dir() -> Result<String, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?;
    Ok(exe_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_default_riot_client_service_path() -> Result<String, String> {
    crate::db::init::get_default_riot_client_service_path()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_default_riot_client_data_path() -> Result<String, String> {
    crate::db::init::get_default_riot_client_data_path()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn mark_launched() -> Result<(), String> {
    let conn = crate::db::init::get_connection(None)?;
    conn.execute("UPDATE settings SET launched = 1 WHERE id = 1", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn copy_account_password(account_id: i64) -> Result<(), String> {
    let account = crate::db::get_account(account_id)?;
    if account.encrypted_password.is_empty() {
        return Err("No password stored".to_string());
    }
    let password = crate::crypto::dpapi::unprotect_password(&account.encrypted_password)?;
    set_clipboard_text(&password)
}

fn set_clipboard_text(text: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winbase::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use winapi::um::winuser::{
        CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData, CF_UNICODETEXT,
    };

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
pub async fn open_shop_window(
    app: tauri::AppHandle,
    account_id: i64,
    title: String,
) -> Result<(), String> {
    let label = format!("shop-{}", account_id);

    if let Some(existing) = app.get_webview_window(&label) {
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        label,
        tauri::WebviewUrl::App(std::path::PathBuf::from("/")),
    )
    .title(title)
    .inner_size(1200.0, 650.0)
    .min_inner_size(960.0, 600.0)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
