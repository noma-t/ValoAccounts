use std::time::Duration;

use super::db;
use super::error::SkinsError;
use super::models::{
    BuddiesApiResponse, ContentTiersApiResponse, FlexApiResponse, PlayercardsApiResponse,
    SkinsApiResponse, SpraysApiResponse, VersionApiResponse,
};

const CONTENT_TIERS_URL: &str = "https://valorant-api.com/v1/contenttiers";
const WEAPON_SKINS_URL: &str = "https://valorant-api.com/v1/weapons/skins";
const BUDDIES_URL: &str = "https://valorant-api.com/v1/buddies";
const FLEX_URL: &str = "https://valorant-api.com/v1/flex";
const PLAYERCARDS_URL: &str = "https://valorant-api.com/v1/playercards";
const SPRAYS_URL: &str = "https://valorant-api.com/v1/sprays";
const VERSION_URL: &str = "https://valorant-api.com/v1/version";

fn build_client() -> Result<reqwest::Client, SkinsError> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(SkinsError::from)
}

async fn fetch_content_tiers(
    client: &reqwest::Client,
) -> Result<ContentTiersApiResponse, SkinsError> {
    let resp = client.get(CONTENT_TIERS_URL).send().await?;

    if !resp.status().is_success() {
        return Err(SkinsError::ApiFailed(format!(
            "contenttiers returned status {}",
            resp.status()
        )));
    }

    resp.json().await.map_err(SkinsError::from)
}

async fn fetch_weapon_skins(client: &reqwest::Client) -> Result<SkinsApiResponse, SkinsError> {
    let resp = client.get(WEAPON_SKINS_URL).send().await?;

    if !resp.status().is_success() {
        return Err(SkinsError::ApiFailed(format!(
            "weapons/skins returned status {}",
            resp.status()
        )));
    }

    resp.json().await.map_err(SkinsError::from)
}

async fn fetch_buddies(client: &reqwest::Client) -> Result<BuddiesApiResponse, SkinsError> {
    let resp = client.get(BUDDIES_URL).send().await?;

    if !resp.status().is_success() {
        return Err(SkinsError::ApiFailed(format!(
            "buddies returned status {}",
            resp.status()
        )));
    }

    resp.json().await.map_err(SkinsError::from)
}

async fn fetch_flex(client: &reqwest::Client) -> Result<FlexApiResponse, SkinsError> {
    let resp = client.get(FLEX_URL).send().await?;

    if !resp.status().is_success() {
        return Err(SkinsError::ApiFailed(format!(
            "flex returned status {}",
            resp.status()
        )));
    }

    resp.json().await.map_err(SkinsError::from)
}

async fn fetch_playercards(client: &reqwest::Client) -> Result<PlayercardsApiResponse, SkinsError> {
    let resp = client.get(PLAYERCARDS_URL).send().await?;

    if !resp.status().is_success() {
        return Err(SkinsError::ApiFailed(format!(
            "playercards returned status {}",
            resp.status()
        )));
    }

    resp.json().await.map_err(SkinsError::from)
}

async fn fetch_sprays(client: &reqwest::Client) -> Result<SpraysApiResponse, SkinsError> {
    let resp = client.get(SPRAYS_URL).send().await?;

    if !resp.status().is_success() {
        return Err(SkinsError::ApiFailed(format!(
            "sprays returned status {}",
            resp.status()
        )));
    }

    resp.json().await.map_err(SkinsError::from)
}

async fn fetch_version(client: &reqwest::Client) -> Result<String, SkinsError> {
    let resp = client.get(VERSION_URL).send().await?;

    if !resp.status().is_success() {
        return Err(SkinsError::ApiFailed(format!(
            "version returned status {}",
            resp.status()
        )));
    }

    let api: VersionApiResponse = resp.json().await.map_err(SkinsError::from)?;
    Ok(api.data.version)
}

/// Sync the skins database with valorant-api.com.
///
/// Returns `Ok(true)` if new data was written, `Ok(false)` if already up to date.
pub async fn sync_skins_database() -> Result<bool, SkinsError> {
    let client = build_client()?;
    let remote_version = fetch_version(&client).await?;
    let stored_version = db::get_stored_version()?;

    let version_changed = stored_version.as_deref() != Some(&remote_version);
    let status = db::get_table_status()?;

    if !version_changed && !status.any_empty() {
        log::info!("Skins database already up to date (version {})", remote_version);
        return Ok(false);
    }

    if version_changed {
        log::info!(
            "Syncing skins database: {:?} -> {}",
            stored_version,
            remote_version
        );
    } else {
        log::info!(
            "Partial sync: filling empty tables (version {})",
            remote_version
        );
    }

    // Tiers are fetched together with weapons since they share a foreign key.
    if version_changed || status.weapons_empty {
        let tiers = fetch_content_tiers(&client).await?;
        db::insert_tiers(&tiers.data)?;
        log::info!("Synced {} content tiers", tiers.data.len());

        let skins = fetch_weapon_skins(&client).await?;
        db::insert_skins(&skins.data)?;
        log::info!("Inserted/updated {} weapon skins", skins.data.len());
    }

    if version_changed || status.buddies_empty {
        let buddies = fetch_buddies(&client).await?;
        db::insert_buddies(&buddies.data)?;
        log::info!("Inserted/updated {} buddies", buddies.data.len());
    }

    if version_changed || status.flex_empty {
        let flex = fetch_flex(&client).await?;
        db::insert_flex(&flex.data)?;
        log::info!("Inserted/updated {} flex items", flex.data.len());
    }

    if version_changed || status.playercards_empty {
        let playercards = fetch_playercards(&client).await?;
        db::insert_playercards(&playercards.data)?;
        log::info!("Inserted/updated {} playercards", playercards.data.len());
    }

    if version_changed || status.sprays_empty {
        let sprays = fetch_sprays(&client).await?;
        db::insert_sprays(&sprays.data)?;
        log::info!("Inserted/updated {} sprays", sprays.data.len());
    }

    // Version is only written after successful data insertion (retry-safe).
    // Skip the write if version was already correct (partial sync for empty tables).
    if version_changed {
        db::set_stored_version(&remote_version)?;
        log::info!("Skins database synced to version {}", remote_version);
    }

    Ok(true)
}
