use std::os::windows::process::CommandExt;
use std::path::PathBuf;

use crate::db::{
    create_account, get_account, get_all_accounts, get_settings, is_current_data_available,
    update_account, CreateAccountData, NewAccount, UpdateAccount,
};

#[tauri::command]
pub fn add_account(account: NewAccount) -> Result<crate::db::models::Account, String> {
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
        log::info!(
            "Auto-selecting account {} after current data import",
            created.id
        );
        perform_account_switch(Some(created.id))?;
    }

    Ok(created)
}

#[tauri::command]
pub fn list_accounts() -> Result<Vec<crate::db::models::Account>, String> {
    get_all_accounts()
}

#[tauri::command]
pub fn edit_account(account: UpdateAccount) -> Result<crate::db::models::Account, String> {
    update_account(account)
}

#[tauri::command]
pub fn check_current_data_available() -> Result<bool, String> {
    is_current_data_available()
}

#[tauri::command]
pub fn switch_account(account_id: Option<i64>) -> Result<(), String> {
    log::info!("Starting account switch: {:?}", account_id);

    if crate::process::check_valorant_running() {
        log::warn!("Cannot switch accounts: Valorant is running");
        return Err("Cannot switch accounts while Valorant is running".to_string());
    }

    let riot_client_was_running = crate::process::check_riot_client_running();

    if riot_client_was_running {
        log::info!("Riot Client is running, killing it before account switch");
        crate::process::kill_riot_client()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    perform_account_switch(account_id)?;

    if riot_client_was_running {
        log::info!("Relaunching Riot Client after account switch");
        crate::process::launch_riot_client()?;
    }

    log::info!("Account switch completed successfully");
    Ok(())
}

fn perform_account_switch(account_id: Option<i64>) -> Result<(), String> {
    let settings = get_settings()?;

    let riot_data_path = match settings.riot_client_data_path {
        Some(path) => PathBuf::from(path),
        None => crate::db::init::get_default_riot_client_data_path()?,
    };

    let account_data_path = match settings.account_data_path {
        Some(path) => PathBuf::from(path),
        None => crate::db::init::get_default_account_data_path()?,
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
        crate::fs::create_dir_with_marker(&target)?;
    }

    // Force cleanup of any existing path (junction, directory, or broken link)
    log::info!(
        "Cleaning up riot data path if it exists: {}",
        riot_data_path.display()
    );

    let path_exists =
        riot_data_path.exists() || crate::fs::is_symlink(&riot_data_path).unwrap_or(false);

    if path_exists {
        if crate::fs::is_symlink(&riot_data_path).unwrap_or(false) {
            log::info!("Detected junction point, removing");
            crate::fs::remove_junction(&riot_data_path)?;
        } else if riot_data_path.is_dir() {
            log::info!("Detected regular directory, moving contents to target");
            crate::fs::move_directory_contents(&riot_data_path, &target)?;
            std::fs::remove_dir(&riot_data_path)
                .map_err(|e| format!("Failed to remove directory: {}", e))?;
        }
    }

    // Force remove anything that might still exist (including broken junctions)
    let output = std::process::Command::new("cmd")
        .args(["/C", "rmdir", &riot_data_path.to_string_lossy()])
        .creation_flags(0x08000000)
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            log::info!("Successfully cleaned up path with rmdir");
        }
    }

    log::info!(
        "Creating junction: {} -> {}",
        riot_data_path.display(),
        target.display()
    );
    crate::fs::create_junction(&riot_data_path, &target)?;

    let conn = crate::db::init::get_connection(None)?;
    conn.execute(
        "UPDATE settings SET active_account_id = ?1 WHERE id = 1",
        [account_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
