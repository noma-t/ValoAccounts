use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storefront {
    pub daily_offers: Vec<DailyOffer>,
    pub daily_remaining_secs: u64,
    pub night_market: Option<Vec<NightMarketOffer>>,
}

// -- Internal API response types ----------------------------------------------

#[derive(Deserialize)]
pub(super) struct ApiStorefront {
    #[serde(rename = "SkinsPanelLayout")]
    pub(super) skins_panel_layout: SkinsPanelLayout,
    #[serde(rename = "BonusStore")]
    pub(super) bonus_store: Option<BonusStoreData>,
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
