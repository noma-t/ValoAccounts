mod crypto;
mod db;
mod fs;
mod process;
mod shop;
mod skins;

use db::{
    create_account, get_account, get_all_accounts, get_settings, initialize_database, is_current_data_available,
    update_account, update_settings, CreateAccountData, NewAccount, Settings, UpdateAccount,
    UpdateSettings,
};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

static DEMO_MODE: AtomicBool = AtomicBool::new(false);

#[tauri::command]
fn is_demo_mode() -> bool {
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

#[tauri::command]
fn get_account_cookies(account_id: i64) -> Result<Option<shop::RiotCookies>, String> {
    let yaml_path = match resolve_account_yaml_path(account_id)? {
        Some(path) => path,
        None => return Ok(None),
    };

    let content = std::fs::read_to_string(&yaml_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let doc: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let session_cookies = doc
        .get("riot-login")
        .and_then(|v| v.get("persist"))
        .and_then(|v| v.get("session"))
        .and_then(|v| v.get("cookies"))
        .and_then(|v| v.as_sequence());

    let mut cookies = shop::RiotCookies {
        asid: None,
        ccid: None,
        clid: None,
        sub: None,
        csid: None,
        ssid: None,
        tdid: None,
    };

    if let Some(cookie_list) = session_cookies {
        for cookie in cookie_list {
            let name = cookie.get("name").and_then(|v| v.as_str());
            let value = cookie.get("value").and_then(|v| v.as_str());
            if let (Some(n), Some(v)) = (name, value) {
                match n {
                    "asid" => cookies.asid = Some(v.to_string()),
                    "ccid" => cookies.ccid = Some(v.to_string()),
                    "clid" => cookies.clid = Some(v.to_string()),
                    "sub" => cookies.sub = Some(v.to_string()),
                    "csid" => cookies.csid = Some(v.to_string()),
                    "ssid" => cookies.ssid = Some(v.to_string()),
                    _ => {}
                }
            }
        }
    }

    cookies.tdid = doc
        .get("rso-authenticator")
        .and_then(|v| v.get("tdid"))
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    if cookies.ssid.is_none() {
        return Ok(None);
    }

    Ok(Some(cookies))
}

/// Resolve the path to an account's RiotGamesPrivateSettings.yaml.
fn resolve_account_yaml_path(account_id: i64) -> Result<Option<PathBuf>, String> {
    let account = get_account(account_id)?;
    let data_folder = account
        .data_folder
        .ok_or("Account has no data directory assigned")?;

    let settings = get_settings()?;
    let account_data_path = match settings.account_data_path {
        Some(path) => PathBuf::from(path),
        None => db::init::get_default_account_data_path()?,
    };

    let yaml_path = account_data_path
        .join(&data_folder)
        .join("RiotGamesPrivateSettings.yaml");

    if yaml_path.exists() {
        Ok(Some(yaml_path))
    } else {
        Ok(None)
    }
}

/// Update cookie values in the YAML content string without altering formatting.
///
/// For session cookies under `riot-login.persist.session.cookies`, this finds
/// each `- name: <cookie_name>` block and replaces the `value:` line.
/// For `tdid`, it finds `rso-authenticator` > `tdid` > `value:` and replaces it.
fn update_yaml_cookie_values(content: &str, cookies: &shop::RiotCookies) -> String {
    log::debug!("update_yaml_cookie_values: starting YAML cookie replacement");
    let cookie_updates: &[(&str, &Option<String>)] = &[
        ("ssid", &cookies.ssid),
        ("asid", &cookies.asid),
        ("csid", &cookies.csid),
        ("ccid", &cookies.ccid),
        ("clid", &cookies.clid),
        ("sub", &cookies.sub),
    ];

    let mut result = content.to_string();

    for &(cookie_name, cookie_value) in cookie_updates {
        if let Some(new_val) = cookie_value {
            // Actual YAML structure:
            //     -   domain: "auth.riotgames.com"
            //         hostOnly: true
            //         ...
            //         name: "ssid"
            //         ...
            //         value: "old_value"
            // Match `name: "cookie_name"`, skip intermediate fields, then
            // capture up to and including `value: ` and replace the quoted value.
            let pattern = format!(
                r#"(?m)(name:\s*"?{}"?\s*\n(?:\s+\w+:.*\n)*?\s+value:\s*)"[^"]*""#,
                regex::escape(cookie_name)
            );
            if let Ok(re) = regex::Regex::new(&pattern) {
                let had_match = re.is_match(&result);
                let replacement = new_val.clone();
                result = re
                    .replace(&result, |caps: &regex::Captures| {
                        format!("{}\"{}\"", &caps[1], replacement)
                    })
                    .to_string();
                if had_match {
                    log::debug!(
                        "update_yaml_cookie_values: replaced {} ({} chars)",
                        cookie_name,
                        new_val.len()
                    );
                } else {
                    log::debug!(
                        "update_yaml_cookie_values: no match for {} in YAML",
                        cookie_name
                    );
                }
            }
        } else {
            log::debug!(
                "update_yaml_cookie_values: skipping {} (no updated value)",
                cookie_name
            );
        }
    }

    if let Some(new_tdid) = &cookies.tdid {
        // Actual YAML structure:
        //   rso-authenticator:
        //       tdid:
        //           domain: "riotgames.com"
        //           ...
        //           value: "old_value"
        let pattern =
            r#"(?m)(rso-authenticator:\s*\n\s+tdid:\s*\n(?:\s+\w+:.*\n)*?\s+value:\s*)"[^"]*""#;
        if let Ok(re) = regex::Regex::new(pattern) {
            let had_match = re.is_match(&result);
            let replacement = new_tdid.clone();
            result = re
                .replace(&result, |caps: &regex::Captures| {
                    format!("{}\"{}\"", &caps[1], replacement)
                })
                .to_string();
            if had_match {
                log::debug!(
                    "update_yaml_cookie_values: replaced tdid ({} chars)",
                    new_tdid.len()
                );
            } else {
                log::debug!("update_yaml_cookie_values: no match for tdid in YAML");
            }
        }
    } else {
        log::debug!("update_yaml_cookie_values: skipping tdid (no updated value)");
    }

    let changed = content != result;
    log::debug!(
        "update_yaml_cookie_values: done, content_changed={}",
        changed
    );

    result
}

fn save_account_cookies(account_id: i64, cookies: &shop::RiotCookies) -> Result<(), String> {
    log::debug!("save_account_cookies: starting for account {}", account_id);

    let yaml_path = match resolve_account_yaml_path(account_id)? {
        Some(path) => {
            log::debug!("save_account_cookies: resolved YAML path: {}", path.display());
            path
        }
        None => {
            log::info!(
                "Skipping cookie save for account {}: YAML file does not exist",
                account_id
            );
            return Ok(());
        }
    };

    let content = std::fs::read_to_string(&yaml_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    log::debug!(
        "save_account_cookies: read YAML file ({} bytes)",
        content.len()
    );

    let updated_content = update_yaml_cookie_values(&content, cookies);

    if content == updated_content {
        log::debug!("save_account_cookies: no changes detected, skipping write");
        return Ok(());
    }

    // Atomic write: write to a temp file, then rename over the original
    let tmp_path = yaml_path.with_extension("yaml.tmp");
    log::debug!(
        "save_account_cookies: writing {} bytes to temp file: {}",
        updated_content.len(),
        tmp_path.display()
    );
    std::fs::write(&tmp_path, &updated_content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    log::debug!("save_account_cookies: renaming temp file to YAML path");
    std::fs::rename(&tmp_path, &yaml_path)
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    log::info!(
        "save_account_cookies: successfully saved updated cookies for account {}",
        account_id
    );
    Ok(())
}

/// Fetch the daily shop and night market, returning a cached result when valid.
#[tauri::command]
async fn get_shop(account_id: i64, cookies: shop::RiotCookies) -> Result<shop::Storefront, String> {
    log::debug!("get_shop: called for account {}", account_id);

    if let Some(cached) = shop::load_cached_storefront(account_id) {
        log::debug!("get_shop: returning cached storefront for account {}", account_id);
        return Ok(cached);
    }

    log::debug!("get_shop: no cache, fetching storefront for account {}", account_id);
    let (storefront, updated_cookies) = shop::fetch_storefront(cookies)
        .await
        .map_err(|e| e.to_string())?;

    log::debug!("get_shop: storefront fetched, saving cache");
    shop::save_storefront_cache(account_id, &storefront);

    log::debug!("get_shop: persisting updated cookies to YAML");
    if let Err(e) = save_account_cookies(account_id, &updated_cookies) {
        log::warn!("Failed to save updated cookies for account {}: {}", account_id, e);
    }

    Ok(storefront)
}

#[tauri::command]
fn get_skin_info(level_uuid: String) -> Result<Option<skins::SkinWeapon>, String> {
    skins::get_skin_by_level_uuid(&level_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_skin_info_batch(level_uuids: Vec<String>) -> Result<Vec<Option<skins::SkinWeapon>>, String> {
    skins::get_skins_by_level_uuids(&level_uuids).map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_skins() -> Result<bool, String> {
    skins::sync_skins_database()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_shop_window(app: tauri::AppHandle, account_id: i64, title: String) -> Result<(), String> {
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

    #[cfg(debug_assertions)]
    if std::env::args().any(|a| a == "--demo") {
        DEMO_MODE.store(true, Ordering::Relaxed);
        log::info!("Demo mode enabled");
    }

    if let Err(e) = initialize_database(None) {
        log::error!("Failed to initialize database: {}", e);
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = skins::initialize_skins_db(None) {
        log::error!("Failed to initialize skins database: {}", e);
    }

    tauri::Builder::default()
        .setup(|app| {
            process::start_process_monitor(app.handle().clone());

            tauri::async_runtime::spawn(async {
                match skins::sync_skins_database().await {
                    Ok(true) => log::info!("Skins database synced successfully"),
                    Ok(false) => log::info!("Skins database already up to date"),
                    Err(e) => log::warn!("Failed to sync skins database: {}", e),
                }
            });

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
            get_account_cookies,
            get_shop,
            get_skin_info,
            get_skin_info_batch,
            sync_skins,
            open_shop_window,
            is_demo_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
