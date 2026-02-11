use super::{get_connection, models::Settings, models::UpdateSettings};
use std::path::PathBuf;

pub fn get_settings() -> Result<Settings, String> {
    let conn = get_connection(None)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, active_account_id, riot_client_service_path, riot_client_data_path, account_data_path, henrikdev_api_key, launched, created_at, updated_at
             FROM settings
             WHERE id = 1",
        )
        .map_err(|e| e.to_string())?;

    let settings = stmt
        .query_row([], |row| {
            Ok(Settings {
                id: row.get(0)?,
                active_account_id: row.get(1)?,
                riot_client_service_path: row.get(2)?,
                riot_client_data_path: row.get(3)?,
                account_data_path: row.get(4)?,
                henrikdev_api_key: row.get(5)?,
                launched: row.get::<_, i64>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

pub fn update_settings(update: UpdateSettings) -> Result<Settings, String> {
    let conn = get_connection(None)?;

    let prev_settings = get_settings()?;

    if update.riot_client_data_path.is_some() || update.account_data_path.is_some() {
        let riot_data_path = if let Some(ref path) = update.riot_client_data_path {
            PathBuf::from(path)
        } else if let Some(ref path) = prev_settings.riot_client_data_path {
            PathBuf::from(path)
        } else {
            super::init::get_default_riot_client_data_path()?
        };

        let account_data_path = if let Some(ref path) = update.account_data_path {
            PathBuf::from(path)
        } else if let Some(ref path) = prev_settings.account_data_path {
            PathBuf::from(path)
        } else {
            super::init::get_default_account_data_path()?
        };

        if !crate::fs::is_symlink(&riot_data_path)? {
            let unselected = account_data_path.join("_unselected");
            std::fs::create_dir_all(&unselected)
                .map_err(|e| format!("Failed to create _unselected: {}", e))?;

            if riot_data_path.exists() {
                crate::fs::move_directory_contents(&riot_data_path, &unselected)?;

                std::fs::remove_dir(&riot_data_path)
                    .map_err(|e| format!("Failed to remove old directory: {}", e))?;
            }

            crate::fs::create_junction(&riot_data_path, &unselected)?;
        }
    }

    conn.execute(
        "UPDATE settings
         SET riot_client_service_path = COALESCE(?1, riot_client_service_path),
             riot_client_data_path = COALESCE(?2, riot_client_data_path),
             account_data_path = COALESCE(?3, account_data_path),
             henrikdev_api_key = COALESCE(?4, henrikdev_api_key)
         WHERE id = 1",
        (
            &update.riot_client_service_path,
            &update.riot_client_data_path,
            &update.account_data_path,
            &update.henrikdev_api_key,
        ),
    )
    .map_err(|e| e.to_string())?;

    get_settings()
}
