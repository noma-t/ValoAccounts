use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};

use super::error::SkinsError;
use super::models::{
    ChromaApiEntry, ContentTierApiEntry, LevelApiEntry, SkinApiEntry, SkinWeapon,
};

const SCHEMA_SQL: &str = include_str!("schema.sql");

static SKINS_DB_PATH: Mutex<Option<String>> = Mutex::new(None);

fn get_default_skins_db_path() -> Result<PathBuf, String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_dir = exe_path
        .parent()
        .ok_or("Failed to get executable directory")?;
    Ok(exe_dir.join("skins.db"))
}

pub fn initialize_skins_db(db_path: Option<PathBuf>) -> Result<(), String> {
    let default_path = get_default_skins_db_path()?;
    let path = db_path.unwrap_or(default_path);
    let path_str = path.to_string_lossy().to_string();
    *SKINS_DB_PATH.lock().unwrap_or_else(|e| e.into_inner()) = Some(path_str.clone());

    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open skins database: {}", e))?;

    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| format!("Failed to initialize skins schema: {}", e))?;

    Ok(())
}

pub(super) fn get_connection() -> Result<Connection, SkinsError> {
    let path = SKINS_DB_PATH
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .ok_or_else(|| SkinsError::Database("Skins DB not initialized".to_string()))?;

    Connection::open(&path).map_err(SkinsError::from)
}

pub(super) fn get_stored_version() -> Result<Option<String>, SkinsError> {
    let conn = get_connection()?;
    let version: Option<String> = conn
        .query_row("SELECT version FROM info WHERE rowid = 1", [], |row| {
            row.get(0)
        })
        .map_err(SkinsError::from)?;
    Ok(version)
}

pub(super) fn set_stored_version(version: &str) -> Result<(), SkinsError> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE info SET version = ?1 WHERE rowid = 1",
        [version],
    )
    .map_err(SkinsError::from)?;
    Ok(())
}

pub(super) fn insert_tiers(tiers: &[ContentTierApiEntry]) -> Result<(), SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("INSERT OR REPLACE INTO tiers (uuid, color, rank, displayIcon) VALUES (?1, ?2, ?3, ?4)")
        .map_err(SkinsError::from)?;

    for tier in tiers {
        stmt.execute((
            &tier.uuid,
            &tier.highlight_color,
            tier.rank,
            &tier.display_icon,
        ))
        .map_err(SkinsError::from)?;
    }

    Ok(())
}

pub(super) fn insert_skins(skins: &[SkinApiEntry]) -> Result<(), SkinsError> {
    let conn = get_connection()?;
    let tx = conn.unchecked_transaction().map_err(SkinsError::from)?;

    {
        let mut weapon_stmt = tx
            .prepare("INSERT OR REPLACE INTO weapons (uuid, displayName, displayIcon, tierUuid) VALUES (?1, ?2, ?3, ?4)")
            .map_err(SkinsError::from)?;
        let mut level_stmt = tx
            .prepare("INSERT OR REPLACE INTO levels (uuid, weaponUuid, displayName, displayIcon, streamedVideo) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(SkinsError::from)?;
        let mut chroma_stmt = tx
            .prepare("INSERT OR REPLACE INTO chromas (uuid, weaponUuid, displayName, displayIcon, streamedVideo, swatch) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .map_err(SkinsError::from)?;

        for skin in skins {
            weapon_stmt
                .execute((
                    &skin.uuid,
                    &skin.display_name,
                    &skin.display_icon,
                    &skin.content_tier_uuid,
                ))
                .map_err(SkinsError::from)?;

            for level in &skin.levels {
                insert_level(&mut level_stmt, level, &skin.uuid)?;
            }

            for chroma in &skin.chromas {
                insert_chroma(&mut chroma_stmt, chroma, &skin.uuid)?;
            }
        }
    }

    tx.commit().map_err(SkinsError::from)?;
    Ok(())
}

fn insert_level(
    stmt: &mut rusqlite::Statement,
    level: &LevelApiEntry,
    weapon_uuid: &str,
) -> Result<(), SkinsError> {
    stmt.execute((
        &level.uuid,
        weapon_uuid,
        &level.display_name,
        &level.display_icon,
        &level.streamed_video,
    ))
    .map_err(SkinsError::from)?;
    Ok(())
}

fn insert_chroma(
    stmt: &mut rusqlite::Statement,
    chroma: &ChromaApiEntry,
    weapon_uuid: &str,
) -> Result<(), SkinsError> {
    stmt.execute((
        &chroma.uuid,
        weapon_uuid,
        &chroma.display_name,
        &chroma.display_icon,
        &chroma.streamed_video,
        &chroma.swatch,
    ))
    .map_err(SkinsError::from)?;
    Ok(())
}

fn map_skin_weapon_row(row: &rusqlite::Row) -> rusqlite::Result<SkinWeapon> {
    Ok(SkinWeapon {
        uuid: row.get(0)?,
        display_name: row.get(1)?,
        display_icon: row.get(2)?,
        tier_uuid: row.get(3)?,
        tier_color: row.get(4)?,
        tier_rank: row.get(5)?,
        tier_icon: row.get(6)?,
    })
}

const LEVEL_LOOKUP_SQL: &str =
    "SELECT w.uuid, w.displayName, w.displayIcon, w.tierUuid,
            t.color, t.rank, t.displayIcon
     FROM levels l
     JOIN weapons w ON l.weaponUuid = w.uuid
     LEFT JOIN tiers t ON w.tierUuid = t.uuid
     WHERE l.uuid = ?1";

pub fn get_skin_by_level_uuid(level_uuid: &str) -> Result<Option<SkinWeapon>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(LEVEL_LOOKUP_SQL).map_err(SkinsError::from)?;

    let result = stmt
        .query_row([level_uuid], map_skin_weapon_row)
        .optional()
        .map_err(SkinsError::from)?;

    Ok(result)
}

pub fn get_skins_by_level_uuids(
    level_uuids: &[String],
) -> Result<Vec<Option<SkinWeapon>>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(LEVEL_LOOKUP_SQL).map_err(SkinsError::from)?;

    level_uuids
        .iter()
        .map(|uuid| {
            stmt.query_row([uuid.as_str()], map_skin_weapon_row)
                .optional()
                .map_err(SkinsError::from)
        })
        .collect()
}