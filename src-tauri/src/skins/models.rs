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

// -- Buddies API types --------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct BuddiesApiResponse {
    pub(super) data: Vec<BuddyApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct BuddyApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "displayName")]
    pub(super) display_name: String,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "assetPath")]
    pub(super) asset_path: Option<String>,
    pub(super) levels: Vec<BuddyLevelApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct BuddyLevelApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "charmLevel")]
    pub(super) charm_level: Option<i32>,
    #[serde(rename = "displayName")]
    pub(super) display_name: Option<String>,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "assetPath")]
    pub(super) asset_path: Option<String>,
}

// -- Flex API types -----------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct FlexApiResponse {
    pub(super) data: Vec<FlexApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct FlexApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "displayName")]
    pub(super) display_name: String,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "assetPath")]
    pub(super) asset_path: Option<String>,
}

// -- Playercards API types ----------------------------------------------------

#[derive(Deserialize)]
pub(super) struct PlayercardsApiResponse {
    pub(super) data: Vec<PlayercardApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct PlayercardApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "displayName")]
    pub(super) display_name: String,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "smallArt")]
    pub(super) small_art: Option<String>,
    #[serde(rename = "wideArt")]
    pub(super) wide_art: Option<String>,
    #[serde(rename = "largeArt")]
    pub(super) large_art: Option<String>,
    #[serde(rename = "assetPath")]
    pub(super) asset_path: Option<String>,
}

// -- Sprays API types ---------------------------------------------------------

#[derive(Deserialize)]
pub(super) struct SpraysApiResponse {
    pub(super) data: Vec<SprayApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct SprayApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "displayName")]
    pub(super) display_name: String,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "fullTransparentIcon")]
    pub(super) full_transparent_icon: Option<String>,
    #[serde(rename = "animationGif")]
    pub(super) animation_gif: Option<String>,
    #[serde(rename = "assetPath")]
    pub(super) asset_path: Option<String>,
    pub(super) levels: Vec<SprayLevelApiEntry>,
}

#[derive(Deserialize)]
pub(super) struct SprayLevelApiEntry {
    pub(super) uuid: String,
    #[serde(rename = "sprayLevel")]
    pub(super) spray_level: Option<i32>,
    #[serde(rename = "displayName")]
    pub(super) display_name: Option<String>,
    #[serde(rename = "displayIcon")]
    pub(super) display_icon: Option<String>,
    #[serde(rename = "assetPath")]
    pub(super) asset_path: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
pub struct BuddyItem {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub asset_path: Option<String>,
    pub level_uuid: String,
    pub charm_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlexItem {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub asset_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayercardItem {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub small_art: Option<String>,
    pub wide_art: Option<String>,
    pub large_art: Option<String>,
    pub asset_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SprayItem {
    pub uuid: String,
    pub display_name: String,
    pub display_icon: Option<String>,
    pub full_transparent_icon: Option<String>,
    pub animation_gif: Option<String>,
    pub asset_path: Option<String>,
    pub level_uuid: String,
    pub spray_level: Option<i32>,
}
