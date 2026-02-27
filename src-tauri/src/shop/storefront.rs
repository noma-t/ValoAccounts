use std::collections::HashMap;

use super::types::{ApiStorefront, Bundle, BundleItem, DailyOffer, NightMarketOffer, Storefront};

/// ItemTypeID for weapon skin levels in the Valorant API.
const WEAPON_SKIN_ITEM_TYPE_ID: &str = "e7c63390-eda7-46e0-bb7a-a6abdacd2433";

fn first_cost(cost: &HashMap<String, u64>) -> u64 {
    cost.values().next().copied().unwrap_or(0)
}

pub(super) fn extract_access_token(location: &str) -> Option<String> {
    let prefix = "access_token=";
    let start = location.find(prefix)?;
    let after = &location[start + prefix.len()..];
    let end = after.find('&').unwrap_or(after.len());
    let token = &after[..end];
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// Parse the raw API storefront response into the public `Storefront` type.
///
/// `bundle_names` maps `DataAssetID` → display name fetched from valorant-api.com.
/// Bundles whose name is missing fall back to their `DataAssetID`.
pub(super) fn parse_storefront(
    raw: ApiStorefront,
    bundle_names: HashMap<String, String>,
) -> Storefront {
    let cost_map: HashMap<String, u64> = raw
        .skins_panel_layout
        .single_item_store_offers
        .unwrap_or_default()
        .into_iter()
        .map(|offer| {
            let vp = first_cost(&offer.cost);
            (offer.offer_id, vp)
        })
        .collect();

    let daily_offers = raw
        .skins_panel_layout
        .single_item_offers
        .into_iter()
        .map(|uuid| DailyOffer {
            vp_cost: cost_map.get(&uuid).copied().unwrap_or(0),
            skin_uuid: uuid,
        })
        .collect();

    let night_market_remaining_secs = raw
        .bonus_store
        .as_ref()
        .and_then(|bs| bs.remaining_duration_secs);

    let night_market = raw.bonus_store.map(|bs| {
        bs.bonus_store_offers
            .into_iter()
            .map(|o| NightMarketOffer {
                skin_uuid: o.offer.offer_id,
                base_cost: first_cost(&o.offer.cost),
                discount_cost: first_cost(&o.discount_costs),
                discount_percent: o.discount_percent,
            })
            .collect()
    });

    let bundles = raw.featured_bundle.map(|fb| {
        fb.bundles
            .into_iter()
            .map(|bundle| {
                let name = bundle_names
                    .get(&bundle.data_asset_id)
                    .cloned()
                    .unwrap_or_else(|| bundle.data_asset_id.clone());

                let items: Vec<BundleItem> = bundle
                    .items
                    .into_iter()
                    .filter(|item| item.item.item_type_id == WEAPON_SKIN_ITEM_TYPE_ID)
                    .map(|item| BundleItem {
                        skin_uuid: item.item.item_id,
                        base_cost: item.base_price,
                        discounted_cost: item.discounted_price,
                        // API gives fraction (0–1); store as percentage (0–100)
                        discount_percent: item.discount_percent * 100.0,
                    })
                    .collect();

                Bundle {
                    name,
                    total_base_cost: first_cost(
                        &bundle.total_base_cost.unwrap_or_default(),
                    ),
                    total_discounted_cost: first_cost(
                        &bundle.total_discounted_cost.unwrap_or_default(),
                    ),
                    // API gives fraction (0–1); store as percentage (0–100)
                    total_discount_percent: bundle.total_discount_percent * 100.0,
                    bundle_remaining_secs: bundle.duration_remaining_secs,
                    items,
                }
            })
            .collect()
    });

    Storefront {
        daily_offers,
        daily_remaining_secs: raw.skins_panel_layout.remaining_duration_secs,
        bundles,
        night_market,
        night_market_remaining_secs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{
        BonusOffer, BonusStoreData, BonusStoreOffer, SingleItemStoreOffer,
        SkinsPanelLayout,
    };

    fn make_bonus_store(offers: Vec<BonusStoreOffer>) -> BonusStoreData {
        BonusStoreData {
            bonus_store_offers: offers,
            remaining_duration_secs: Some(604800),
        }
    }

    fn vp_cost_map(cost: u64) -> HashMap<String, u64> {
        let mut m = HashMap::new();
        m.insert("85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741".to_string(), cost);
        m
    }

    #[test]
    fn test_extract_token_from_fragment() {
        let url = "https://playvalorant.com/opt_in#access_token=abc123&token_type=Bearer&expires_in=3600";
        assert_eq!(extract_access_token(url), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_token_last_param() {
        let url = "https://playvalorant.com/opt_in#token_type=Bearer&access_token=xyz789";
        assert_eq!(extract_access_token(url), Some("xyz789".to_string()));
    }

    #[test]
    fn test_extract_token_only_param() {
        assert_eq!(extract_access_token("https://example.com#access_token=only"), Some("only".to_string()));
    }

    #[test]
    fn test_extract_token_missing() {
        assert_eq!(extract_access_token("https://example.com?something=else"), None);
    }

    #[test]
    fn test_extract_token_empty_string() {
        assert_eq!(extract_access_token(""), None);
    }

    #[test]
    fn test_parse_daily_offers_with_costs() {
        let raw = ApiStorefront {
            skins_panel_layout: SkinsPanelLayout {
                single_item_offers: vec!["skin-a".to_string(), "skin-b".to_string()],
                remaining_duration_secs: 86400,
                single_item_store_offers: Some(vec![
                    SingleItemStoreOffer {
                        offer_id: "skin-a".to_string(),
                        cost: vp_cost_map(1775),
                    },
                    SingleItemStoreOffer {
                        offer_id: "skin-b".to_string(),
                        cost: vp_cost_map(2175),
                    },
                ]),
            },
            bonus_store: None,
            featured_bundle: None,
        };

        let sf = parse_storefront(raw, HashMap::new());
        assert_eq!(sf.daily_remaining_secs, 86400);
        assert_eq!(sf.daily_offers.len(), 2);
        assert_eq!(sf.daily_offers[0], DailyOffer { skin_uuid: "skin-a".to_string(), vp_cost: 1775 });
        assert_eq!(sf.daily_offers[1], DailyOffer { skin_uuid: "skin-b".to_string(), vp_cost: 2175 });
        assert!(sf.night_market.is_none());
        assert!(sf.bundles.is_none());
    }

    #[test]
    fn test_parse_no_store_offers_gives_zero_cost() {
        let raw = ApiStorefront {
            skins_panel_layout: SkinsPanelLayout {
                single_item_offers: vec!["skin-a".to_string()],
                remaining_duration_secs: 0,
                single_item_store_offers: None,
            },
            bonus_store: None,
            featured_bundle: None,
        };
        assert_eq!(parse_storefront(raw, HashMap::new()).daily_offers[0].vp_cost, 0);
    }

    #[test]
    fn test_parse_with_night_market() {
        let raw = ApiStorefront {
            skins_panel_layout: SkinsPanelLayout {
                single_item_offers: vec![],
                remaining_duration_secs: 0,
                single_item_store_offers: None,
            },
            bonus_store: Some(make_bonus_store(vec![BonusStoreOffer {
                offer: BonusOffer {
                    offer_id: "nm-skin".to_string(),
                    cost: vp_cost_map(2175),
                },
                discount_percent: 40.0,
                discount_costs: vp_cost_map(1305),
            }])),
            featured_bundle: None,
        };

        let nm = parse_storefront(raw, HashMap::new()).night_market.unwrap();
        assert_eq!(nm.len(), 1);
        assert_eq!(nm[0], NightMarketOffer {
            skin_uuid: "nm-skin".to_string(),
            base_cost: 2175,
            discount_cost: 1305,
            discount_percent: 40.0,
        });
    }

    #[test]
    fn test_parse_no_night_market() {
        let raw = ApiStorefront {
            skins_panel_layout: SkinsPanelLayout {
                single_item_offers: vec![],
                remaining_duration_secs: 0,
                single_item_store_offers: None,
            },
            bonus_store: None,
            featured_bundle: None,
        };
        assert!(parse_storefront(raw, HashMap::new()).night_market.is_none());
    }

    #[test]
    fn test_parse_bundle_items_filtered_to_weapon_skins() {
        use super::super::types::{ApiBundleData, ApiBundleItem, ApiBundleItemDetail, FeaturedBundleWrapper};

        let raw = ApiStorefront {
            skins_panel_layout: SkinsPanelLayout {
                single_item_offers: vec![],
                remaining_duration_secs: 0,
                single_item_store_offers: None,
            },
            bonus_store: None,
            featured_bundle: Some(FeaturedBundleWrapper {
                bundles: vec![ApiBundleData {
                    data_asset_id: "bundle-uuid".to_string(),
                    items: vec![
                        ApiBundleItem {
                            item: ApiBundleItemDetail {
                                item_type_id: WEAPON_SKIN_ITEM_TYPE_ID.to_string(),
                                item_id: "skin-uuid".to_string(),
                            },
                            base_price: 2175,
                            discount_percent: 0.42,
                            discounted_price: 1262,
                        },
                        ApiBundleItem {
                            item: ApiBundleItemDetail {
                                // spray/buddy — should be filtered out
                                item_type_id: "d5f120f8-ff8c-4aac-92ea-f2b5acbe9475".to_string(),
                                item_id: "spray-uuid".to_string(),
                            },
                            base_price: 375,
                            discount_percent: 0.42,
                            discounted_price: 217,
                        },
                    ],
                    total_base_cost: Some(vp_cost_map(14025)),
                    total_discounted_cost: Some(vp_cost_map(8825)),
                    total_discount_percent: 0.371,
                    duration_remaining_secs: 259200,
                }],
            }),
        };

        let mut names = HashMap::new();
        names.insert("bundle-uuid".to_string(), "Spectrum".to_string());

        let sf = parse_storefront(raw, names);
        let bundles = sf.bundles.unwrap();
        assert_eq!(bundles.len(), 1);

        let bundle = &bundles[0];
        assert_eq!(bundle.name, "Spectrum");
        assert_eq!(bundle.total_base_cost, 14025);
        assert_eq!(bundle.total_discounted_cost, 8825);
        assert!((bundle.total_discount_percent - 37.1).abs() < 0.01);
        assert_eq!(bundle.bundle_remaining_secs, 259200);

        // Only weapon skin should be present
        assert_eq!(bundle.items.len(), 1);
        assert_eq!(bundle.items[0].skin_uuid, "skin-uuid");
        assert_eq!(bundle.items[0].base_cost, 2175);
        assert_eq!(bundle.items[0].discounted_cost, 1262);
        assert!((bundle.items[0].discount_percent - 42.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_bundle_name_fallback_to_uuid() {
        use super::super::types::{ApiBundleData, FeaturedBundleWrapper};

        let raw = ApiStorefront {
            skins_panel_layout: SkinsPanelLayout {
                single_item_offers: vec![],
                remaining_duration_secs: 0,
                single_item_store_offers: None,
            },
            bonus_store: None,
            featured_bundle: Some(FeaturedBundleWrapper {
                bundles: vec![ApiBundleData {
                    data_asset_id: "unknown-uuid".to_string(),
                    items: vec![],
                    total_base_cost: None,
                    total_discounted_cost: None,
                    total_discount_percent: 0.0,
                    duration_remaining_secs: 3600,
                }],
            }),
        };

        let sf = parse_storefront(raw, HashMap::new());
        let bundles = sf.bundles.unwrap();
        assert_eq!(bundles[0].name, "unknown-uuid");
    }
}
