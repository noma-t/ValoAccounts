use std::collections::HashMap;

use super::types::{ApiStorefront, DailyOffer, NightMarketOffer, Storefront};

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

pub(super) fn parse_storefront(raw: ApiStorefront) -> Storefront {
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

    Storefront {
        daily_offers,
        daily_remaining_secs: raw.skins_panel_layout.remaining_duration_secs,
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
        };

        let sf = parse_storefront(raw);
        assert_eq!(sf.daily_remaining_secs, 86400);
        assert_eq!(sf.daily_offers.len(), 2);
        assert_eq!(sf.daily_offers[0], DailyOffer { skin_uuid: "skin-a".to_string(), vp_cost: 1775 });
        assert_eq!(sf.daily_offers[1], DailyOffer { skin_uuid: "skin-b".to_string(), vp_cost: 2175 });
        assert!(sf.night_market.is_none());
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
        };
        assert_eq!(parse_storefront(raw).daily_offers[0].vp_cost, 0);
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
        };

        let nm = parse_storefront(raw).night_market.unwrap();
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
        };
        assert!(parse_storefront(raw).night_market.is_none());
    }
}
