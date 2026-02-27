use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};

use super::error::SkinsError;
use super::models::{
    BuddyApiEntry, BuddyItem, BuddyLevelApiEntry, ChromaApiEntry, ContentTierApiEntry,
    FlexApiEntry, FlexItem, LevelApiEntry, PlayercardApiEntry, PlayercardItem, SkinApiEntry,
    SkinWeapon, SprayApiEntry, SprayItem, SprayLevelApiEntry,
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

pub(super) struct TableStatus {
    pub weapons_empty: bool,
    pub buddies_empty: bool,
    pub flex_empty: bool,
    pub playercards_empty: bool,
    pub sprays_empty: bool,
}

impl TableStatus {
    pub fn any_empty(&self) -> bool {
        self.weapons_empty
            || self.buddies_empty
            || self.flex_empty
            || self.playercards_empty
            || self.sprays_empty
    }
}

fn is_table_empty(conn: &Connection, table: &str) -> Result<bool, SkinsError> {
    let count: i64 = conn
        .query_row(
            &format!("SELECT COUNT(*) FROM {}", table),
            [],
            |row| row.get(0),
        )
        .map_err(SkinsError::from)?;
    Ok(count == 0)
}

pub(super) fn get_table_status() -> Result<TableStatus, SkinsError> {
    let conn = get_connection()?;
    Ok(TableStatus {
        weapons_empty: is_table_empty(&conn, "weapons")?,
        buddies_empty: is_table_empty(&conn, "buddies")?,
        flex_empty: is_table_empty(&conn, "flex")?,
        playercards_empty: is_table_empty(&conn, "playercards")?,
        sprays_empty: is_table_empty(&conn, "sprays")?,
    })
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

// -- Buddies ------------------------------------------------------------------

pub(super) fn insert_buddies(buddies: &[BuddyApiEntry]) -> Result<(), SkinsError> {
    let conn = get_connection()?;
    let tx = conn.unchecked_transaction().map_err(SkinsError::from)?;

    {
        let mut buddy_stmt = tx
            .prepare(
                "INSERT OR REPLACE INTO buddies (uuid, displayName, displayIcon, assetPath) \
                 VALUES (?1, ?2, ?3, ?4)",
            )
            .map_err(SkinsError::from)?;
        let mut level_stmt = tx
            .prepare(
                "INSERT OR REPLACE INTO buddy_levels \
                 (uuid, buddyUuid, charmLevel, displayName, displayIcon, assetPath) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(SkinsError::from)?;

        for buddy in buddies {
            buddy_stmt
                .execute((&buddy.uuid, &buddy.display_name, &buddy.display_icon, &buddy.asset_path))
                .map_err(SkinsError::from)?;

            for level in &buddy.levels {
                insert_buddy_level(&mut level_stmt, level, &buddy.uuid)?;
            }
        }
    }

    tx.commit().map_err(SkinsError::from)?;
    Ok(())
}

fn insert_buddy_level(
    stmt: &mut rusqlite::Statement,
    level: &BuddyLevelApiEntry,
    buddy_uuid: &str,
) -> Result<(), SkinsError> {
    stmt.execute((
        &level.uuid,
        buddy_uuid,
        level.charm_level,
        &level.display_name,
        &level.display_icon,
        &level.asset_path,
    ))
    .map_err(SkinsError::from)?;
    Ok(())
}

fn map_buddy_item_row(row: &rusqlite::Row) -> rusqlite::Result<BuddyItem> {
    Ok(BuddyItem {
        uuid: row.get(0)?,
        display_name: row.get(1)?,
        display_icon: row.get(2)?,
        asset_path: row.get(3)?,
        level_uuid: row.get(4)?,
        charm_level: row.get(5)?,
    })
}

// Looks up a buddy by either its level UUID or its parent UUID (UNION covers both cases,
// since the storefront may send either depending on the item variant).
const BUDDY_LOOKUP_SQL: &str =
    "SELECT b.uuid, b.displayName, b.displayIcon, b.assetPath, bl.uuid, bl.charmLevel \
     FROM buddy_levels bl \
     JOIN buddies b ON bl.buddyUuid = b.uuid \
     WHERE bl.uuid = ?1 \
     UNION ALL \
     SELECT b.uuid, b.displayName, b.displayIcon, b.assetPath, b.uuid, NULL \
     FROM buddies b \
     WHERE b.uuid = ?1 \
     LIMIT 1";

pub fn get_buddy_by_level_uuid(level_uuid: &str) -> Result<Option<BuddyItem>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(BUDDY_LOOKUP_SQL)
        .map_err(SkinsError::from)?;

    stmt.query_row([level_uuid], map_buddy_item_row)
        .optional()
        .map_err(SkinsError::from)
}

pub fn get_buddies_by_level_uuids(
    level_uuids: &[String],
) -> Result<Vec<Option<BuddyItem>>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(BUDDY_LOOKUP_SQL)
        .map_err(SkinsError::from)?;

    level_uuids
        .iter()
        .map(|uuid| {
            stmt.query_row([uuid.as_str()], map_buddy_item_row)
                .optional()
                .map_err(SkinsError::from)
        })
        .collect()
}

// -- Flex ---------------------------------------------------------------------

pub(super) fn insert_flex(items: &[FlexApiEntry]) -> Result<(), SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO flex (uuid, displayName, displayIcon, assetPath) \
             VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(SkinsError::from)?;

    for item in items {
        stmt.execute((&item.uuid, &item.display_name, &item.display_icon, &item.asset_path))
            .map_err(SkinsError::from)?;
    }

    Ok(())
}

fn map_flex_item_row(row: &rusqlite::Row) -> rusqlite::Result<FlexItem> {
    Ok(FlexItem {
        uuid: row.get(0)?,
        display_name: row.get(1)?,
        display_icon: row.get(2)?,
        asset_path: row.get(3)?,
    })
}

pub fn get_flex_by_uuid(uuid: &str) -> Result<Option<FlexItem>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("SELECT uuid, displayName, displayIcon, assetPath FROM flex WHERE uuid = ?1")
        .map_err(SkinsError::from)?;

    stmt.query_row([uuid], map_flex_item_row)
        .optional()
        .map_err(SkinsError::from)
}

pub fn get_flex_by_uuids(uuids: &[String]) -> Result<Vec<Option<FlexItem>>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare("SELECT uuid, displayName, displayIcon, assetPath FROM flex WHERE uuid = ?1")
        .map_err(SkinsError::from)?;

    uuids
        .iter()
        .map(|uuid| {
            stmt.query_row([uuid.as_str()], map_flex_item_row)
                .optional()
                .map_err(SkinsError::from)
        })
        .collect()
}

// -- Playercards --------------------------------------------------------------

pub(super) fn insert_playercards(cards: &[PlayercardApiEntry]) -> Result<(), SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO playercards \
             (uuid, displayName, displayIcon, smallArt, wideArt, largeArt, assetPath) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(SkinsError::from)?;

    for card in cards {
        stmt.execute((
            &card.uuid,
            &card.display_name,
            &card.display_icon,
            &card.small_art,
            &card.wide_art,
            &card.large_art,
            &card.asset_path,
        ))
        .map_err(SkinsError::from)?;
    }

    Ok(())
}

fn map_playercard_item_row(row: &rusqlite::Row) -> rusqlite::Result<PlayercardItem> {
    Ok(PlayercardItem {
        uuid: row.get(0)?,
        display_name: row.get(1)?,
        display_icon: row.get(2)?,
        small_art: row.get(3)?,
        wide_art: row.get(4)?,
        large_art: row.get(5)?,
        asset_path: row.get(6)?,
    })
}

const PLAYERCARD_LOOKUP_SQL: &str =
    "SELECT uuid, displayName, displayIcon, smallArt, wideArt, largeArt, assetPath \
     FROM playercards WHERE uuid = ?1";

pub fn get_playercard_by_uuid(uuid: &str) -> Result<Option<PlayercardItem>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(PLAYERCARD_LOOKUP_SQL)
        .map_err(SkinsError::from)?;

    stmt.query_row([uuid], map_playercard_item_row)
        .optional()
        .map_err(SkinsError::from)
}

pub fn get_playercards_by_uuids(
    uuids: &[String],
) -> Result<Vec<Option<PlayercardItem>>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(PLAYERCARD_LOOKUP_SQL)
        .map_err(SkinsError::from)?;

    uuids
        .iter()
        .map(|uuid| {
            stmt.query_row([uuid.as_str()], map_playercard_item_row)
                .optional()
                .map_err(SkinsError::from)
        })
        .collect()
}

// -- Sprays -------------------------------------------------------------------

pub(super) fn insert_sprays(sprays: &[SprayApiEntry]) -> Result<(), SkinsError> {
    let conn = get_connection()?;
    let tx = conn.unchecked_transaction().map_err(SkinsError::from)?;

    {
        let mut spray_stmt = tx
            .prepare(
                "INSERT OR REPLACE INTO sprays \
                 (uuid, displayName, displayIcon, fullTransparentIcon, animationGif, assetPath) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(SkinsError::from)?;
        let mut level_stmt = tx
            .prepare(
                "INSERT OR REPLACE INTO spray_levels \
                 (uuid, sprayUuid, sprayLevel, displayName, displayIcon, assetPath) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(SkinsError::from)?;

        for spray in sprays {
            spray_stmt
                .execute((
                    &spray.uuid,
                    &spray.display_name,
                    &spray.display_icon,
                    &spray.full_transparent_icon,
                    &spray.animation_gif,
                    &spray.asset_path,
                ))
                .map_err(SkinsError::from)?;

            for level in &spray.levels {
                insert_spray_level(&mut level_stmt, level, &spray.uuid)?;
            }
        }
    }

    tx.commit().map_err(SkinsError::from)?;
    Ok(())
}

fn insert_spray_level(
    stmt: &mut rusqlite::Statement,
    level: &SprayLevelApiEntry,
    spray_uuid: &str,
) -> Result<(), SkinsError> {
    stmt.execute((
        &level.uuid,
        spray_uuid,
        level.spray_level,
        &level.display_name,
        &level.display_icon,
        &level.asset_path,
    ))
    .map_err(SkinsError::from)?;
    Ok(())
}

fn map_spray_item_row(row: &rusqlite::Row) -> rusqlite::Result<SprayItem> {
    Ok(SprayItem {
        uuid: row.get(0)?,
        display_name: row.get(1)?,
        display_icon: row.get(2)?,
        full_transparent_icon: row.get(3)?,
        animation_gif: row.get(4)?,
        asset_path: row.get(5)?,
        level_uuid: row.get(6)?,
        spray_level: row.get(7)?,
    })
}

// Looks up a spray by either its level UUID or its parent UUID (UNION covers both cases,
// since the storefront may send either depending on the item variant).
const SPRAY_LOOKUP_SQL: &str =
    "SELECT s.uuid, s.displayName, s.displayIcon, s.fullTransparentIcon, s.animationGif, \
            s.assetPath, sl.uuid, sl.sprayLevel \
     FROM spray_levels sl \
     JOIN sprays s ON sl.sprayUuid = s.uuid \
     WHERE sl.uuid = ?1 \
     UNION ALL \
     SELECT s.uuid, s.displayName, s.displayIcon, s.fullTransparentIcon, s.animationGif, \
            s.assetPath, s.uuid, NULL \
     FROM sprays s \
     WHERE s.uuid = ?1 \
     LIMIT 1";

pub fn get_spray_by_level_uuid(level_uuid: &str) -> Result<Option<SprayItem>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(SPRAY_LOOKUP_SQL)
        .map_err(SkinsError::from)?;

    stmt.query_row([level_uuid], map_spray_item_row)
        .optional()
        .map_err(SkinsError::from)
}

pub fn get_sprays_by_level_uuids(
    level_uuids: &[String],
) -> Result<Vec<Option<SprayItem>>, SkinsError> {
    let conn = get_connection()?;
    let mut stmt = conn
        .prepare(SPRAY_LOOKUP_SQL)
        .map_err(SkinsError::from)?;

    level_uuids
        .iter()
        .map(|uuid| {
            stmt.query_row([uuid.as_str()], map_spray_item_row)
                .optional()
                .map_err(SkinsError::from)
        })
        .collect()
}