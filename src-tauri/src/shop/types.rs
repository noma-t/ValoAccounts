use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// -- Riot account cookies -----------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiotCookies {
    pub asid: Option<String>,
    pub ccid: Option<String>,
    pub clid: Option<String>,
    pub sub: Option<String>,
    pub csid: Option<String>,
    pub ssid: Option<String>,
    pub tdid: Option<String>,
}

// -- Public output types ------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyOffer {
    pub skin_uuid: String,
    pub vp_cost: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NightMarketOffer {
    pub skin_uuid: String,
    pub base_cost: u64,
    pub discount_cost: u64,
    pub discount_percent: f64,
}

/// Individual item within a featured bundle.
///
/// `item_type_id` identifies the cosmetic category (weapon skin, buddy, spray, etc.).
/// `discount_percent` is stored as a percentage (0–100), matching `NightMarketOffer`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BundleItem {
    // `skin_uuid` alias preserves compatibility with caches written before the rename.
    #[serde(alias = "skin_uuid")]
    pub item_uuid: String,
    // Old cache entries pre-date this field; default to weapon skin so they still render.
    #[serde(default = "default_skin_type_id")]
    pub item_type_id: String,
    pub base_cost: u64,
    pub discounted_cost: u64,
    pub discount_percent: f64,
}

fn default_skin_type_id() -> String {
    "e7c63390-eda7-46e0-bb7a-a6abdacd2433".to_string()
}

/// A featured bundle shown in the shop.
///
/// `total_discount_percent` is stored as a percentage (0–100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub name: String,
    pub total_base_cost: u64,
    pub total_discounted_cost: u64,
    pub total_discount_percent: f64,
    pub bundle_remaining_secs: u64,
    pub items: Vec<BundleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storefront {
    pub daily_offers: Vec<DailyOffer>,
    pub daily_remaining_secs: u64,
    pub bundles: Option<Vec<Bundle>>,
    pub night_market: Option<Vec<NightMarketOffer>>,
    pub night_market_remaining_secs: Option<u64>,
}

// -- Internal API response types ----------------------------------------------

#[derive(Deserialize)]
pub(super) struct ApiStorefront {
    #[serde(rename = "SkinsPanelLayout")]
    pub(super) skins_panel_layout: SkinsPanelLayout,
    #[serde(rename = "BonusStore")]
    pub(super) bonus_store: Option<BonusStoreData>,
    #[serde(rename = "FeaturedBundle")]
    pub(super) featured_bundle: Option<FeaturedBundleWrapper>,
}

#[derive(Deserialize)]
pub(super) struct FeaturedBundleWrapper {
    /// The individual bundles currently featured.  Usually 1–2 entries.
    #[serde(rename = "Bundles")]
    pub(super) bundles: Vec<ApiBundleData>,
}

#[derive(Deserialize)]
pub(super) struct ApiBundleData {
    /// UUID used to look up the bundle display name on valorant-api.com.
    #[serde(rename = "DataAssetID")]
    pub(super) data_asset_id: String,
    #[serde(rename = "Items")]
    pub(super) items: Vec<ApiBundleItem>,
    #[serde(rename = "TotalBaseCost")]
    pub(super) total_base_cost: Option<HashMap<String, u64>>,
    #[serde(rename = "TotalDiscountedCost")]
    pub(super) total_discounted_cost: Option<HashMap<String, u64>>,
    /// Fraction in [0, 1].
    #[serde(rename = "TotalDiscountPercent")]
    pub(super) total_discount_percent: f64,
    #[serde(rename = "DurationRemainingInSeconds")]
    pub(super) duration_remaining_secs: u64,
}

#[derive(Deserialize)]
pub(super) struct ApiBundleItem {
    #[serde(rename = "Item")]
    pub(super) item: ApiBundleItemDetail,
    #[serde(rename = "BasePrice")]
    pub(super) base_price: u64,
    /// Fraction in [0, 1].
    #[serde(rename = "DiscountPercent")]
    pub(super) discount_percent: f64,
    #[serde(rename = "DiscountedPrice")]
    pub(super) discounted_price: u64,
}

#[derive(Deserialize)]
pub(super) struct ApiBundleItemDetail {
    #[serde(rename = "ItemTypeID")]
    pub(super) item_type_id: String,
    #[serde(rename = "ItemID")]
    pub(super) item_id: String,
}

#[derive(Deserialize)]
pub(super) struct SkinsPanelLayout {
    #[serde(rename = "SingleItemOffers")]
    pub(super) single_item_offers: Vec<String>,
    #[serde(rename = "SingleItemOffersRemainingDurationInSeconds")]
    pub(super) remaining_duration_secs: u64,
    /// Contains VP cost per skin; absent in some API versions.
    #[serde(rename = "SingleItemStoreOffers")]
    pub(super) single_item_store_offers: Option<Vec<SingleItemStoreOffer>>,
}

#[derive(Deserialize)]
pub(super) struct SingleItemStoreOffer {
    #[serde(rename = "OfferID")]
    pub(super) offer_id: String,
    #[serde(rename = "Cost")]
    pub(super) cost: HashMap<String, u64>,
}

#[derive(Deserialize)]
pub(super) struct BonusStoreData {
    #[serde(rename = "BonusStoreOffers")]
    pub(super) bonus_store_offers: Vec<BonusStoreOffer>,
    #[serde(rename = "BonusStoreRemainingDurationInSeconds")]
    pub(super) remaining_duration_secs: Option<u64>,
}

#[derive(Deserialize)]
pub(super) struct BonusStoreOffer {
    #[serde(rename = "Offer")]
    pub(super) offer: BonusOffer,
    #[serde(rename = "DiscountPercent")]
    pub(super) discount_percent: f64,
    #[serde(rename = "DiscountCosts")]
    pub(super) discount_costs: HashMap<String, u64>,
}

#[derive(Deserialize)]
pub(super) struct BonusOffer {
    #[serde(rename = "OfferID")]
    pub(super) offer_id: String,
    #[serde(rename = "Cost")]
    pub(super) cost: HashMap<String, u64>,
}

#[derive(Deserialize)]
pub(super) struct EntitlementsResponse {
    pub(super) entitlements_token: String,
}

#[derive(Deserialize)]
pub(super) struct UserInfoResponse {
    pub(super) sub: String,
}
