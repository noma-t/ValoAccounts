use std::collections::HashMap;
use std::sync::Arc;

use reqwest::cookie::{CookieStore, Jar};
use reqwest::Client;
use serde::Deserialize;

use super::error::ShopError;
use super::storefront::{extract_access_token, parse_storefront};
use super::types::{ApiStorefront, EntitlementsResponse, RiotCookies, Storefront, UserInfoResponse};

const VALORANT_API_BUNDLE_URL: &str = "https://valorant-api.com/v1/bundles/";

#[derive(Deserialize)]
struct BundleApiResponse {
    data: BundleApiData,
}

#[derive(Deserialize)]
struct BundleApiData {
    #[serde(rename = "displayName")]
    display_name: String,
}

/// Fetch the display name for a bundle from valorant-api.com.
///
/// Returns `None` on any network or parse error (non-fatal).
async fn fetch_bundle_display_name(uuid: &str) -> Option<String> {
    let url = format!("{}{}", VALORANT_API_BUNDLE_URL, uuid);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let resp: BundleApiResponse = client
        .get(&url)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    Some(resp.data.display_name)
}

const AUTH_COOKIES_URL: &str = "https://auth.riotgames.com/api/v1/authorization";
const AUTH_REAUTH_URL: &str = "https://auth.riotgames.com/authorize";
const ENTITLEMENTS_URL: &str = "https://entitlements.auth.riotgames.com/api/token/v1";
const USERINFO_URL: &str = "https://auth.riotgames.com/userinfo";

const CLIENT_PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";
const RIOT_AUTH_URL: &str = "https://auth.riotgames.com";
const RIOT_GAMES_URL: &str = "https://riotgames.com";

const AUTH_PARAMS: &[(&str, &str)] = &[
    ("client_id", "play-valorant-web-prod"),
    ("nonce", "1"),
    ("redirect_uri", "https://playvalorant.com/opt_in"),
    ("response_type", "token id_token"),
    ("scope", "account openid"),
];

/// Derive the shard from the `clid` cookie value by stripping trailing digits.
///
/// Examples: "ap1" -> "ap", "na1" -> "na", "eu3" -> "eu", "kr" -> "kr"
pub(super) fn shard_from_clid(clid: &str) -> &str {
    clid.trim_end_matches(|c: char| c.is_ascii_digit())
}

pub(super) struct ShopClient {
    shard: String,
    puuid: Option<String>,
    client: Client,
    jar: Arc<Jar>,
}

impl ShopClient {
    pub(super) fn new(
        cookies: RiotCookies,
        user_agent: &str,
    ) -> Result<Self, ShopError> {
        let shard = cookies
            .clid
            .as_deref()
            .map(shard_from_clid)
            .unwrap_or("ap")
            .to_string();

        let puuid = cookies.sub.clone();

        let jar = Arc::new(Jar::default());

        let auth_url: reqwest::Url = RIOT_AUTH_URL
            .parse()
            .map_err(|e| ShopError::ParseError(format!("Invalid URL constant: {}", e)))?;
        let riot_url: reqwest::Url = RIOT_GAMES_URL
            .parse()
            .map_err(|e| ShopError::ParseError(format!("Invalid URL constant: {}", e)))?;

        let auth_cookies: &[(&str, &Option<String>)] = &[
            ("ssid", &cookies.ssid),
            ("asid", &cookies.asid),
            ("csid", &cookies.csid),
            ("ccid", &cookies.ccid),
            ("clid", &cookies.clid),
            ("sub", &cookies.sub),
        ];

        for &(name, value) in auth_cookies {
            if let Some(v) = value {
                jar.add_cookie_str(&format!("{}={}", name, v), &auth_url);
            }
        }

        if let Some(ref v) = cookies.tdid {
            jar.add_cookie_str(&format!("tdid={}", v), &riot_url);
        }

        // Clone the Arc before passing to cookie_provider, which consumes it.
        // This lets us read cookies back from the jar after authentication.
        let jar_ref = Arc::clone(&jar);

        let client = Client::builder()
            .cookie_provider(jar)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(user_agent)
            .build()?;

        Ok(Self {
            shard,
            puuid,
            client,
            jar: jar_ref,
        })
    }

    async fn authenticate(&self) -> Result<String, ShopError> {
        let auth_body = serde_json::json!({
            "client_id": "play-valorant-web-prod",
            "nonce": "1",
            "redirect_uri": "https://playvalorant.com/opt_in",
            "response_type": "token id_token",
            "scope": "account openid",
        });

        self.client
            .post(AUTH_COOKIES_URL)
            .header("Content-Type", "application/json")
            .json(&auth_body)
            .send()
            .await?;

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

    pub(super) async fn fetch(&self, client_version: &str) -> Result<Storefront, ShopError> {
        let access_token = self.authenticate().await?;
        let entitlements_token = self.get_entitlements_token(&access_token).await?;

        let puuid = match &self.puuid {
            Some(p) => p.clone(),
            None => self.get_puuid(&access_token).await?,
        };

        let raw = self
            .get_storefront_raw(&access_token, &entitlements_token, &puuid, client_version)
            .await?;

        // Collect DataAssetIDs before raw is consumed by parse_storefront
        let asset_ids: Vec<String> = raw
            .featured_bundle
            .as_ref()
            .map(|fb| {
                fb.bundles
                    .iter()
                    .map(|b| b.data_asset_id.clone())
                    .collect()
            })
            .unwrap_or_default();

        // Fetch bundle display names from the public valorant-api.com (non-fatal)
        let mut bundle_names: HashMap<String, String> = HashMap::new();
        for asset_id in &asset_ids {
            match fetch_bundle_display_name(asset_id).await {
                Some(name) => {
                    log::debug!("fetch: bundle name for {} = \"{}\"", asset_id, name);
                    bundle_names.insert(asset_id.clone(), name);
                }
                None => log::warn!("fetch: could not get bundle name for {}", asset_id),
            }
        }

        Ok(parse_storefront(raw, bundle_names))
    }

    /// Extract the current cookie values from the jar after authentication.
    ///
    /// The auth flow may have updated cookies via Set-Cookie headers; this
    /// reads them back so the caller can persist them.
    pub(super) fn extract_updated_cookies(&self) -> RiotCookies {
        log::debug!("Extracting updated cookies from jar");
        let auth_url: reqwest::Url = RIOT_AUTH_URL.parse().expect("constant URL is valid");
        let riot_url: reqwest::Url = RIOT_GAMES_URL.parse().expect("constant URL is valid");

        let mut cookies = RiotCookies {
            asid: None,
            ccid: None,
            clid: None,
            sub: None,
            csid: None,
            ssid: None,
            tdid: None,
        };

        if let Some(header) = self.jar.cookies(&auth_url) {
            let header_str = header.to_str().unwrap_or("");
            log::debug!("Auth cookie header has {} chars", header_str.len());
            for pair in header_str.split("; ") {
                if let Some((name, value)) = pair.split_once('=') {
                    let matched = match name {
                        "ssid" => { cookies.ssid = Some(value.to_string()); true }
                        "asid" => { cookies.asid = Some(value.to_string()); true }
                        "csid" => { cookies.csid = Some(value.to_string()); true }
                        "ccid" => { cookies.ccid = Some(value.to_string()); true }
                        "clid" => { cookies.clid = Some(value.to_string()); true }
                        "sub" => { cookies.sub = Some(value.to_string()); true }
                        _ => false,
                    };
                    if matched {
                        log::debug!("Extracted cookie: {} ({} chars)", name, value.len());
                    }
                }
            }
        } else {
            log::debug!("No cookies found in jar for auth URL");
        }

        if let Some(header) = self.jar.cookies(&riot_url) {
            let header_str = header.to_str().unwrap_or("");
            log::debug!("Riot cookie header has {} chars", header_str.len());
            for pair in header_str.split("; ") {
                if let Some((name, value)) = pair.split_once('=') {
                    if name == "tdid" {
                        cookies.tdid = Some(value.to_string());
                        log::debug!("Extracted cookie: tdid ({} chars)", value.len());
                    }
                }
            }
        } else {
            log::debug!("No cookies found in jar for riot URL");
        }

        let present: Vec<&str> = [
            cookies.ssid.as_ref().map(|_| "ssid"),
            cookies.asid.as_ref().map(|_| "asid"),
            cookies.csid.as_ref().map(|_| "csid"),
            cookies.ccid.as_ref().map(|_| "ccid"),
            cookies.clid.as_ref().map(|_| "clid"),
            cookies.sub.as_ref().map(|_| "sub"),
            cookies.tdid.as_ref().map(|_| "tdid"),
        ]
        .into_iter()
        .flatten()
        .collect();
        log::debug!("Extracted cookies summary: [{}]", present.join(", "));

        cookies
    }
}
