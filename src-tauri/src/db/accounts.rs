use super::{get_connection, models::{Account, UpdateAccount}};
use crate::crypto::keyring::{encrypt_password, get_or_create_encryption_key};
use crate::fs::create_dir_with_marker;
use chrono::Local;

pub struct CreateAccountData {
    pub riot_id: String,
    pub tagline: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub rank: Option<String>,
    pub use_current_data: bool,
}

pub fn generate_data_folder_name(account_id: i64) -> String {
    let now = Local::now();
    format!("{:03}_{}", account_id, now.format("%Y%m%d%H%M%S"))
}

pub fn create_account(data: CreateAccountData) -> Result<Account, String> {
    log::info!(
        "Creating new account: {}#{} (use_current_data: {})",
        data.riot_id,
        data.tagline,
        data.use_current_data
    );

    let conn = get_connection(None)?;

    let encrypted_password = if let Some(ref pw) = data.password {
        let key = get_or_create_encryption_key()?;
        encrypt_password(pw, &key)?
    } else {
        vec![]
    };

    conn.execute(
        "INSERT INTO accounts (riot_id, tagline, username, encrypted_password, rank, data_folder)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
        (
            &data.riot_id,
            &data.tagline,
            &data.username,
            &encrypted_password,
            &data.rank,
        ),
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    let generated_folder = generate_data_folder_name(id);

    log::debug!("Generated folder name: {}", generated_folder);

    conn.execute(
        "UPDATE accounts SET data_folder = ?1 WHERE id = ?2",
        (&generated_folder, id),
    )
    .map_err(|e| e.to_string())?;

    let settings = super::settings::get_settings()?;
    let account_data_path = match settings.account_data_path {
        Some(path) => std::path::PathBuf::from(path),
        None => super::init::get_default_account_data_path()?,
    };

    log::debug!("Account data path: {}", account_data_path.display());

    if data.use_current_data {
        log::info!("Using current data mode");
        let unselected_path = account_data_path.join("_unselected");
        let new_path = account_data_path.join(&generated_folder);

        if unselected_path.exists() {
            log::info!(
                "Renaming _unselected directory: {} -> {}",
                unselected_path.display(),
                new_path.display()
            );
            std::fs::rename(&unselected_path, &new_path)
                .map_err(|e| format!("Failed to rename _unselected: {}", e))?;
        } else {
            log::warn!("_unselected directory not found, creating new directory: {}", new_path.display());
            create_dir_with_marker(&new_path)?;
        }
    } else {
        log::info!("Creating new data directory");
        let dir_path = account_data_path.join(&generated_folder);
        log::debug!("Creating directory: {}", dir_path.display());
        create_dir_with_marker(&dir_path)?;
    }

    log::info!("Account created successfully with ID: {}", id);
    get_account_by_id(&conn, id)
}

pub fn get_account(account_id: i64) -> Result<Account, String> {
    let conn = get_connection(None)?;
    get_account_by_id(&conn, account_id)
}

pub fn get_all_accounts() -> Result<Vec<Account>, String> {
    let conn = get_connection(None)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, riot_id, tagline, username, encrypted_password, rank, is_active, data_folder, created_at, updated_at
             FROM accounts ORDER BY created_at ASC",
        )
        .map_err(|e| e.to_string())?;

    let accounts = stmt
        .query_map([], |row| {
            let encrypted_password: Vec<u8> = row.get(4)?;
            let has_password = !encrypted_password.is_empty();
            Ok(Account {
                id: row.get(0)?,
                riot_id: row.get(1)?,
                tagline: row.get(2)?,
                username: row.get(3)?,
                encrypted_password,
                has_password,
                rank: row.get(5)?,
                is_active: row.get(6)?,
                data_folder: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(accounts)
}

pub fn update_account(data: UpdateAccount) -> Result<Account, String> {
    let conn = get_connection(None)?;

    if let Some(ref pw) = data.password {
        let key = get_or_create_encryption_key()?;
        let encrypted = encrypt_password(pw, &key)?;
        conn.execute(
            "UPDATE accounts SET riot_id=?1, tagline=?2, username=?3, encrypted_password=?4, rank=?5, updated_at=datetime('now') WHERE id=?6",
            (&data.riot_id, &data.tagline, &data.username, &encrypted, &data.rank, data.id),
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE accounts SET riot_id=?1, tagline=?2, username=?3, rank=?4, updated_at=datetime('now') WHERE id=?5",
            (&data.riot_id, &data.tagline, &data.username, &data.rank, data.id),
        )
        .map_err(|e| e.to_string())?;
    }

    get_account_by_id(&conn, data.id)
}

pub fn is_current_data_available() -> Result<bool, String> {
    let conn = get_connection(None)?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE data_folder = '_unselected'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(count == 0)
}

fn get_account_by_id(conn: &rusqlite::Connection, id: i64) -> Result<Account, String> {
    conn.query_row(
        "SELECT id, riot_id, tagline, username, encrypted_password, rank, is_active, data_folder, created_at, updated_at
         FROM accounts WHERE id = ?1",
        [id],
        |row| {
            let encrypted_password: Vec<u8> = row.get(4)?;
            let has_password = !encrypted_password.is_empty();
            Ok(Account {
                id: row.get(0)?,
                riot_id: row.get(1)?,
                tagline: row.get(2)?,
                username: row.get(3)?,
                encrypted_password,
                has_password,
                rank: row.get(5)?,
                is_active: row.get(6)?,
                data_folder: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}
