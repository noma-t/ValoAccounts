use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::cookie::Jar;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// ─── Constants ────────────────────────────────────────────────────────────────

const AUTH_COOKIES_URL: &str = "https://auth.riotgames.com/api/v1/authorization";
const AUTH_REAUTH_URL: &str = "https://auth.riotgames.com/authorize";
const ENTITLEMENTS_URL: &str = "https://entitlements.auth.riotgames.com/api/token/v1";
const USERINFO_URL: &str = "https://auth.riotgames.com/userinfo";
const VERSION_URL: &str = "https://valorant-api.com/v1/version";

const DEFAULT_CLIENT_VERSION: &str = "release-12.02-shipping-9-4226954";

/// Base64-encoded JSON: { platformType: PC, platformOS: Windows, ... }
const CLIENT_PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";

/// UUID identifying Valorant Points (VP) as a currency in Riot's API.
const VP_CURRENCY_ID: &str = "85ca954a-00f9-6c41-a35e-4b6c24cd4e36";

const USER_AGENT: &str =
    "RiotClient/70.0.0.4888690.4873386 rso-auth (Windows;10;;Professional, x64)";

/// Base URL used when injecting the SSID cookie into the jar.
const RIOT_AUTH_DOMAIN: &str = "https://auth.riotgames.com";

/// OAuth parameters sent to the re-authorization endpoint.
const AUTH_PARAMS: &[(&str, &str)] = &[
    ("client_id", "play-valorant-web-prod"),
    ("nonce", "1"),
    ("redirect_uri", "https://playvalorant.com/opt_in"),
    ("response_type", "token id_token"),
    ("scope", "account openid"),
];

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ShopError {
    Http(reqwest::Error),
    AuthFailed(String),
    ParseError(String),
    StorefrontFailed,
}

impl std::fmt::Display for ShopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {}", e),
            Self::AuthFailed(msg) => write!(f, "Authentication failed: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::StorefrontFailed => write!(f, "All storefront endpoints failed"),
        }
    }
}

impl std::error::Error for ShopError {}

impl From<reqwest::Error> for ShopError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

// ─── Public output types ──────────────────────────────────────────────────────

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

// ─── Internal API response types ─────────────────────────────────────────────

#[derive(Deserialize)]
struct ApiStorefront {
    #[serde(rename = "SkinsPanelLayout")]
    skins_panel_layout: SkinsPanelLayout,
    #[serde(rename = "BonusStore")]
    bonus_store: Option<BonusStoreData>,
}

#[derive(Deserialize)]
struct SkinsPanelLayout {
    #[serde(rename = "SingleItemOffers")]
    single_item_offers: Vec<String>,
    #[serde(rename = "SingleItemOffersRemainingDurationInSeconds")]
    remaining_duration_secs: u64,
    /// Contains VP cost per skin; absent in some API versions.
    #[serde(rename = "SingleItemStoreOffers")]
    single_item_store_offers: Option<Vec<SingleItemStoreOffer>>,
}

#[derive(Deserialize)]
struct SingleItemStoreOffer {
    #[serde(rename = "OfferID")]
    offer_id: String,
    #[serde(rename = "Cost")]
    cost: HashMap<String, u64>,
}

#[derive(Deserialize)]
struct BonusStoreData {
    #[serde(rename = "BonusStoreOffers")]
    bonus_store_offers: Vec<BonusStoreOffer>,
}

#[derive(Deserialize)]
struct BonusStoreOffer {
    #[serde(rename = "Offer")]
    offer: BonusOffer,
    #[serde(rename = "DiscountPercent")]
    discount_percent: f64,
    #[serde(rename = "DiscountCosts")]
    discount_costs: HashMap<String, u64>,
}

#[derive(Deserialize)]
struct BonusOffer {
    #[serde(rename = "OfferID")]
    offer_id: String,
    #[serde(rename = "Cost")]
    cost: HashMap<String, u64>,
}

#[derive(Deserialize)]
struct EntitlementsResponse {
    entitlements_token: String,
}

#[derive(Deserialize)]
struct UserInfoResponse {
    sub: String,
}

#[derive(Deserialize)]
struct VersionApiResponse {
    data: VersionData,
}

#[derive(Deserialize)]
struct VersionData {
    #[serde(rename = "riotClientVersion")]
    riot_client_version: String,
}

// ─── Token extraction ─────────────────────────────────────────────────────────

/// Extracts the `access_token` value from a Riot OAuth redirect URL.
///
/// Riot encodes the token in the URL fragment or query string:
/// `https://playvalorant.com/opt_in#access_token=TOKEN&...`
fn extract_access_token(location: &str) -> Option<String> {
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

// ─── Storefront parsing ───────────────────────────────────────────────────────

fn parse_storefront(raw: ApiStorefront) -> Storefront {
    let cost_map: HashMap<String, u64> = raw
        .skins_panel_layout
        .single_item_store_offers
        .unwrap_or_default()
        .into_iter()
        .map(|offer| {
            let vp = offer.cost.get(VP_CURRENCY_ID).copied().unwrap_or(0);
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

    let night_market = raw.bonus_store.map(|bs| {
        bs.bonus_store_offers
            .into_iter()
            .map(|o| NightMarketOffer {
                skin_uuid: o.offer.offer_id,
                base_cost: o.offer.cost.get(VP_CURRENCY_ID).copied().unwrap_or(0),
                discount_cost: o.discount_costs.get(VP_CURRENCY_ID).copied().unwrap_or(0),
                discount_percent: o.discount_percent,
            })
            .collect()
    });

    Storefront {
        daily_offers,
        daily_remaining_secs: raw.skins_panel_layout.remaining_duration_secs,
        night_market,
    }
}

// ─── ShopClient ───────────────────────────────────────────────────────────────

/// Low-level client that authenticates with Riot and fetches the storefront.
///
/// Create via [`ShopClient::new`], then call [`ShopClient::fetch`].
pub struct ShopClient {
    ssid: String,
    shard: String,
    client_version_override: Option<String>,
    client: Client,
    jar: Arc<Jar>,
}

impl ShopClient {
    /// Build a new client.
    ///
    /// - `ssid`: Value of the `ssid` cookie from `auth.riotgames.com`.
    /// - `shard`: Region identifier, e.g. `"ap"` (Asia-Pacific), `"na"` (North America).
    /// - `client_version_override`: Pin a specific client version string; auto-fetched when `None`.
    pub fn new(
        ssid: impl Into<String>,
        shard: impl Into<String>,
        client_version_override: Option<String>,
    ) -> Result<Self, ShopError> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            // Disable auto-redirect so we can capture the Location header from the
            // OAuth re-authorization step.
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self {
            ssid: ssid.into(),
            shard: shard.into(),
            client_version_override,
            client,
            jar,
        })
    }

    /// Authenticate using the stored SSID and return an access token.
    ///
    /// Flow:
    /// 1. POST to the authorization endpoint to obtain session cookies (tdid, clid, asid).
    /// 2. Inject the SSID cookie manually into the cookie jar.
    /// 3. GET the re-authorization endpoint with OAuth params; capture the redirect Location.
    /// 4. Extract `access_token` from the Location URL.
    async fn authenticate(&self) -> Result<String, ShopError> {
        let auth_body = serde_json::json!({
            "client_id": "play-valorant-web-prod",
            "nonce": "1",
            "redirect_uri": "https://playvalorant.com/opt_in",
            "response_type": "token id_token",
            "scope": "account openid",
        });

        // Step 1: Initialize the session; response sets tracking cookies in the jar.
        self.client
            .post(AUTH_COOKIES_URL)
            .header("Content-Type", "application/json")
            .json(&auth_body)
            .send()
            .await?;

        // Step 2: Inject the SSID so Riot treats this session as already authenticated.
        let riot_url: reqwest::Url = RIOT_AUTH_DOMAIN
            .parse()
            .map_err(|e| ShopError::ParseError(format!("Invalid URL constant: {}", e)))?;
        self.jar
            .add_cookie_str(&format!("ssid={}", self.ssid), &riot_url);

        // Step 3: Re-authorization request; expect a 301/302/303 redirect.
        let resp = self
            .client
            .get(AUTH_REAUTH_URL)
            .query(AUTH_PARAMS)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if status != 301 && status != 302 && status != 303 {
            return Err(ShopError::AuthFailed(format!(
                "Expected redirect (301/302/303), got {}",
                status
            )));
        }

        // Step 4: Parse access_token from the Location header fragment.
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        extract_access_token(location).ok_or_else(|| {
            ShopError::AuthFailed("Access token not found in redirect URL".to_string())
        })
    }

    async fn get_entitlements_token(&self, access_token: &str) -> Result<String, ShopError> {
        let data: EntitlementsResponse = self
            .client
            .post(ENTITLEMENTS_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(data.entitlements_token)
    }

    async fn get_puuid(&self, access_token: &str) -> Result<String, ShopError> {
        let data: UserInfoResponse = self
            .client
            .get(USERINFO_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(data.sub)
    }

    /// Returns the current Valorant client version string.
    /// Falls back to a hardcoded default if the API is unreachable.
    async fn get_client_version(&self) -> String {
        if let Some(ref v) = self.client_version_override {
            return v.clone();
        }
        match self
            .client
            .get(VERSION_URL)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => resp
                .json::<VersionApiResponse>()
                .await
                .map(|d| d.data.riot_client_version)
                .unwrap_or_else(|_| DEFAULT_CLIENT_VERSION.to_string()),
            _ => DEFAULT_CLIENT_VERSION.to_string(),
        }
    }

    /// Tries storefront endpoints in order: v2 GET → v3 POST → v1 GET.
    async fn get_storefront_raw(
        &self,
        access_token: &str,
        entitlements_token: &str,
        puuid: &str,
        client_version: &str,
    ) -> Result<ApiStorefront, ShopError> {
        let shard = &self.shard;
        let v2 = format!(
            "https://pd.{}.a.pvp.net/store/v2/storefront/{}",
            shard, puuid
        );
        let v3 = format!(
            "https://pd.{}.a.pvp.net/store/v3/storefront/{}",
            shard, puuid
        );
        let v1 = format!(
            "https://pd.{}.a.pvp.net/store/v1/storefront/{}",
            shard, puuid
        );

        let endpoints = [
            ("GET", v2.as_str()),
            ("POST", v3.as_str()),
            ("GET", v1.as_str()),
        ];

        for (method, url) in endpoints {
            let builder = if method == "POST" {
                self.client.post(url).json(&serde_json::json!({}))
            } else {
                self.client.get(url)
            };

            let resp = builder
                .header("Authorization", format!("Bearer {}", access_token))
                .header("X-Riot-Entitlements-JWT", entitlements_token)
                .header("X-Riot-ClientPlatform", CLIENT_PLATFORM)
                .header("X-Riot-ClientVersion", client_version)
                .send()
                .await?;

            if resp.status().is_success() {
                match resp.json::<ApiStorefront>().await {
                    Ok(data) => return Ok(data),
                    Err(_) => continue,
                }
            }
        }

        Err(ShopError::StorefrontFailed)
    }

    /// Authenticate and fetch the daily shop + night market.
    pub async fn fetch(&self) -> Result<Storefront, ShopError> {
        let access_token = self.authenticate().await?;
        let entitlements_token = self.get_entitlements_token(&access_token).await?;
        let puuid = self.get_puuid(&access_token).await?;
        let client_version = self.get_client_version().await;
        let raw = self
            .get_storefront_raw(&access_token, &entitlements_token, &puuid, &client_version)
            .await?;
        Ok(parse_storefront(raw))
    }
}

// ─── Public function ──────────────────────────────────────────────────────────

/// Fetch the Valorant daily shop and night market for a given session.
///
/// # Arguments
/// * `ssid` - Value of the `ssid` cookie from `auth.riotgames.com`.
/// * `shard` - Region shard (e.g. `"ap"` for Asia-Pacific, `"na"` for North America).
/// * `client_version` - Optional version override; fetched automatically when `None`.
pub async fn fetch_storefront(
    ssid: impl Into<String>,
    shard: impl Into<String>,
    client_version: Option<String>,
) -> Result<Storefront, ShopError> {
    ShopClient::new(ssid, shard, client_version)?.fetch().await
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helpers ---

    fn vp_cost_map(cost: u64) -> HashMap<String, u64> {
        let mut m = HashMap::new();
        m.insert(VP_CURRENCY_ID.to_string(), cost);
        m
    }

    // --- extract_access_token ---

    #[test]
    fn test_extract_token_from_fragment() {
        let loc =
            "https://playvalorant.com/opt_in#access_token=abc123&token_type=Bearer&expires_in=3600";
        assert_eq!(extract_access_token(loc), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_token_last_param() {
        let loc = "https://playvalorant.com/opt_in#token_type=Bearer&access_token=xyz789";
        assert_eq!(extract_access_token(loc), Some("xyz789".to_string()));
    }

    #[test]
    fn test_extract_token_only_param() {
        let loc = "https://example.com#access_token=only";
        assert_eq!(extract_access_token(loc), Some("only".to_string()));
    }

    #[test]
    fn test_extract_token_missing() {
        assert_eq!(
            extract_access_token("https://example.com?something=else"),
            None
        );
    }

    #[test]
    fn test_extract_token_empty_string() {
        assert_eq!(extract_access_token(""), None);
    }

    // --- parse_storefront ---

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
        assert_eq!(
            sf.daily_offers[0],
            DailyOffer {
                skin_uuid: "skin-a".to_string(),
                vp_cost: 1775
            }
        );
        assert_eq!(
            sf.daily_offers[1],
            DailyOffer {
                skin_uuid: "skin-b".to_string(),
                vp_cost: 2175
            }
        );
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

        let sf = parse_storefront(raw);
        assert_eq!(sf.daily_offers[0].vp_cost, 0);
    }

    #[test]
    fn test_parse_with_night_market() {
        let raw = ApiStorefront {
            skins_panel_layout: SkinsPanelLayout {
                single_item_offers: vec![],
                remaining_duration_secs: 0,
                single_item_store_offers: None,
            },
            bonus_store: Some(BonusStoreData {
                bonus_store_offers: vec![BonusStoreOffer {
                    offer: BonusOffer {
                        offer_id: "nm-skin".to_string(),
                        cost: vp_cost_map(2175),
                    },
                    discount_percent: 40.0,
                    discount_costs: vp_cost_map(1305),
                }],
            }),
        };

        let sf = parse_storefront(raw);
        let nm = sf.night_market.unwrap();

        assert_eq!(nm.len(), 1);
        assert_eq!(
            nm[0],
            NightMarketOffer {
                skin_uuid: "nm-skin".to_string(),
                base_cost: 2175,
                discount_cost: 1305,
                discount_percent: 40.0,
            }
        );
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

    // --- Integration test (requires real SSID; skipped in CI) ---

    /// Run with:
    ///   TEST_SSID=<your_ssid> TEST_SHARD=ap cargo test test_fetch_storefront_live -- --ignored
    #[tokio::test]
    #[ignore = "requires TEST_SSID env var and network access"]
    async fn test_fetch_storefront_live() {
        let ssid = std::env::var("TEST_SSID").expect("TEST_SSID must be set");
        let shard = std::env::var("TEST_SHARD").unwrap_or_else(|_| "ap".to_string());

        let result = fetch_storefront(ssid, shard, None).await;
        assert!(result.is_ok(), "Error: {:?}", result.unwrap_err());

        let sf = result.unwrap();
        println!("Daily offers ({} sec remaining):", sf.daily_remaining_secs);
        for o in &sf.daily_offers {
            println!("  {} - {} VP", o.skin_uuid, o.vp_cost);
        }
        match &sf.night_market {
            Some(nm) => {
                println!("Night market ({} offers):", nm.len());
                for o in nm {
                    println!(
                        "  {} - {} VP ({}% off, base {} VP)",
                        o.skin_uuid, o.discount_cost, o.discount_percent, o.base_cost
                    );
                }
            }
            None => println!("No night market active."),
        }
    }
}
