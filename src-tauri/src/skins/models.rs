use serde::{Deserialize, Serialize};

// -- Internal API deserialization types (valorant-api.com) ---------------------

#[derive(Deserialize)]
pub(super) struct ContentTiersApiResponse {
    pub(super) data: Vec<ContentTierApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct ContentTierApiEntry {
    pub(super) uuid: String,
    pub(super) rank: Option<i32>,
    #[serde(rename = "highlightColor")]
    pub(super) highlight_color: Option<String>,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct SkinsApiResponse {
    pub(super) data: Vec<SkinApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct SkinApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "displayName")]
    pub(super) display_name: String,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "contentTierUuid")]
    pub(super) content_tier_uuid: Option<String>,
    pub(super) chromas: Vec<ChromaApiEntry>,
    pub(super) levels: Vec<LevelApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct ChromaApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "displayName")]
    pub(super) display_name: String,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "streamedVideo")]
    pub(super) streamed_video: Option<String>,
    pub(super) swatch: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct LevelApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "displayName")]
    pub(super) display_name: Option<String>,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "streamedVideo")]
    pub(super) streamed_video: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct VersionApiResponse {
    pub(super) data: VersionApiData,
}

#[derive(Deserialize)]
pub(super) struct VersionApiData {
    pub(super) version: String,
}

// -- Public query result types ------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SkinWeapon {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub tier_uuid: Option<String>,
    pub tier_color: Option<String>,
    pub tier_rank: Option<i32>,
    pub tier_icon: Option<String>,
}
