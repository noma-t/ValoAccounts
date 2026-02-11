use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use super::accounts;

const SCHEMA_SQL: &str = include_str!("schema.sql");

static DB_PATH: Mutex<Option<String>> = Mutex::new(None);

pub fn get_default_db_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    let exe_dir = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?;

    Ok(exe_dir.join("data.db"))
}


pub fn initialize_database(db_path: Option<PathBuf>) -> Result<Connection, String> {
    let default_path = get_default_db_path()?;
    let path = db_path.unwrap_or(default_path);
    let path_str = path.to_string_lossy().to_string();
    *DB_PATH.lock().unwrap() = Some(path_str.clone());

    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| format!("Failed to initialize schema: {}", e))?;

    run_migrations(&conn)?;

    let default_service_path = get_default_riot_client_service_path()
        .map(|p| p.to_string_lossy().to_string())
        .ok();
    let default_data_path = get_default_riot_client_data_path()
        .map(|p| p.to_string_lossy().to_string())
        .ok();

    conn.execute(
        "UPDATE settings
         SET riot_client_service_path = COALESCE(riot_client_service_path, ?1),
             riot_client_data_path = COALESCE(riot_client_data_path, ?2)
         WHERE id = 1",
        (default_service_path, default_data_path),
    )
    .map_err(|e| format!("Failed to set default paths: {}", e))?;

    Ok(conn)
}

pub fn get_default_account_data_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?;
    Ok(exe_dir.join("Data"))
}

pub fn get_default_riot_client_data_path() -> Result<PathBuf, String> {
    let localappdata = std::env::var("LOCALAPPDATA")
        .map_err(|_| "LOCALAPPDATA environment variable not found".to_string())?;
    Ok(PathBuf::from(localappdata).join("Riot Games").join("Riot Client").join("Data"))
}

pub fn get_default_riot_client_service_path() -> Result<PathBuf, String> {
    Ok(PathBuf::from(r"C:\Riot Games\Riot Client\RiotClientServices.exe"))
}

fn run_migrations(conn: &Connection) -> Result<(), String> {
    let _ = conn.execute(
        "ALTER TABLE accounts ADD COLUMN data_folder TEXT",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE accounts RENAME COLUMN email TO username",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE settings RENAME COLUMN riot_client_path TO riot_client_service_path",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN riot_client_data_path TEXT",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN launched INTEGER NOT NULL DEFAULT 0",
        [],
    );

    let _ = conn.execute(
        "ALTER TABLE settings ADD COLUMN region TEXT",
        [],
    );

    migrate_existing_accounts(conn)?;

    Ok(())
}

fn migrate_existing_accounts(conn: &Connection) -> Result<(), String> {
    let account_data_path: Option<String> = conn
        .query_row(
            "SELECT account_data_path FROM settings WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if account_data_path.is_none() {
        return Ok(());
    }

    let account_data_path = PathBuf::from(account_data_path.unwrap());

    let mut stmt = conn
        .prepare("SELECT id FROM accounts WHERE data_folder IS NULL")
        .map_err(|e| e.to_string())?;

    let account_ids: Vec<i64> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for id in account_ids {
        let data_folder = accounts::generate_data_folder_name(id);

        let dir_path = account_data_path.join(&data_folder);
        std::fs::create_dir_all(&dir_path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        conn.execute(
            "UPDATE accounts SET data_folder = ?1 WHERE id = ?2",
            (&data_folder, id),
        )
        .map_err(|e| e.to_string())?;
    }

    let unselected = account_data_path.join("_unselected");
    std::fs::create_dir_all(&unselected)
        .map_err(|e| format!("Failed to create _unselected: {}", e))?;

    Ok(())
}

pub fn get_connection(db_path: Option<&str>) -> Result<Connection, String> {
    let path = match db_path {
        Some(p) => p.to_string(),
        None => {
            DB_PATH.lock().unwrap()
                .clone()
                .unwrap_or_else(|| ":memory:".to_string())
        }
    };

    Connection::open(&path)
        .map_err(|e| format!("Failed to open database connection: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_database() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_valo_accounts.db");

        if db_path.exists() {
            std::fs::remove_file(&db_path).unwrap();
        }

        {
            let conn = initialize_database(Some(db_path.clone())).unwrap();

            let tables: Vec<String> = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table'")
                .unwrap()
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<Vec<String>, _>>()
                .unwrap();

            assert!(tables.contains(&"accounts".to_string()));
            assert!(tables.contains(&"settings".to_string()));
        }

        std::fs::remove_file(&db_path).unwrap();
    }

    #[test]
    fn test_default_paths() {
        let account_data_path = get_default_account_data_path().unwrap();
        assert!(account_data_path.to_string_lossy().ends_with("Data"));

        let riot_data_path = get_default_riot_client_data_path().unwrap();
        assert!(riot_data_path.to_string_lossy().contains("Riot Client"));
    }

    #[test]
    fn test_settings_start_empty() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_empty_settings.db");

        if db_path.exists() {
            std::fs::remove_file(&db_path).unwrap();
        }

        {
            let conn = initialize_database(Some(db_path.clone())).unwrap();

            let (account_data_path, riot_client_data_path): (Option<String>, Option<String>) = conn
                .query_row(
                    "SELECT account_data_path, riot_client_data_path FROM settings WHERE id = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .unwrap();

            assert!(account_data_path.is_none());
            assert!(riot_client_data_path.is_none());
        }

        std::fs::remove_file(&db_path).unwrap();
    }
}
