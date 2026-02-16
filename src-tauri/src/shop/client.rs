use std::sync::Arc;

use reqwest::cookie::Jar;
use reqwest::Client;

use super::error::ShopError;
use super::storefront::{extract_access_token, parse_storefront};
use super::types::{ApiStorefront, EntitlementsResponse, Storefront, UserInfoResponse};

const AUTH_COOKIES_URL: &str = "https://auth.riotgames.com/api/v1/authorization";
const AUTH_REAUTH_URL: &str = "https://auth.riotgames.com/authorize";
const ENTITLEMENTS_URL: &str = "https://entitlements.auth.riotgames.com/api/token/v1";
const USERINFO_URL: &str = "https://auth.riotgames.com/userinfo";

const CLIENT_PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";
const RIOT_AUTH_DOMAIN: &str = "https://auth.riotgames.com";

const AUTH_PARAMS: &[(&str, &str)] = &[
    ("client_id", "play-valorant-web-prod"),
    ("nonce", "1"),
    ("redirect_uri", "https://playvalorant.com/opt_in"),
    ("response_type", "token id_token"),
    ("scope", "account openid"),
];

pub(super) struct ShopClient {
    ssid: String,
    shard: String,
    client: Client,
    jar: Arc<Jar>,
}

impl ShopClient {
    pub(super) fn new(
        ssid: impl Into<String>,
        shard: impl Into<String>,
        user_agent: &str,
    ) -> Result<Self, ShopError> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(user_agent)
            .build()?;

        Ok(Self {
            ssid: ssid.into(),
            shard: shard.into(),
            client,
            jar,
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

        let riot_url: reqwest::Url = RIOT_AUTH_DOMAIN
            .parse()
            .map_err(|e| ShopError::ParseError(format!("Invalid URL constant: {}", e)))?;
        self.jar
            .add_cookie_str(&format!("ssid={}", self.ssid), &riot_url);

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
        let puuid = self.get_puuid(&access_token).await?;
        let raw = self
            .get_storefront_raw(&access_token, &entitlements_token, &puuid, client_version)
            .await?;
        Ok(parse_storefront(raw))
    }
}
