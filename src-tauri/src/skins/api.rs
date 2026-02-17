use std::time::Duration;

use super::db;
use super::error::SkinsError;
use super::models::{ContentTiersApiResponse, SkinsApiResponse, VersionApiResponse};

const CONTENT_TIERS_URL: &str = "https://valorant-api.com/v1/contenttiers";
const WEAPON_SKINS_URL: &str = "https://valorant-api.com/v1/weapons/skins";
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

    if stored_version.as_deref() == Some(&remote_version) {
        log::info!("Skins database already up to date (version {})", remote_version);
        return Ok(false);
    }

    log::info!(
        "Syncing skins database: {:?} -> {}",
        stored_version,
        remote_version
    );

    let tiers = fetch_content_tiers(&client).await?;
    db::insert_tiers(&tiers.data)?;
    log::info!("Synced {} content tiers", tiers.data.len());

    let skins = fetch_weapon_skins(&client).await?;
    let count = skins.data.len();
    db::insert_skins(&skins.data)?;
    log::info!("Inserted/updated {} weapon skins", count);

    // Version is only written after successful data insertion (retry-safe)
    db::set_stored_version(&remote_version)?;
    log::info!("Skins database synced to version {}", remote_version);

    Ok(true)
}
