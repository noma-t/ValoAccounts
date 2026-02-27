mod cache;
mod client;
mod error;
mod storefront;
mod types;
mod version;

pub use cache::{load_cached_storefront, save_storefront_cache};
pub use error::ShopError;
#[allow(unused_imports)]
pub use types::{Bundle, BundleItem, DailyOffer, NightMarketOffer, RiotCookies, Storefront};

use client::ShopClient;
use version::fetch_version_info;

/// Fetch the Valorant daily shop and night market using account cookies.
///
/// # Arguments
/// * `cookies` - Riot account cookies parsed from RiotGamesPrivateSettings.yaml.
///
/// The shard is derived from `clid` (e.g. "ap1" -> "ap") and the PUUID from `sub`.
pub async fn fetch_storefront(
    cookies: RiotCookies,
) -> Result<(Storefront, RiotCookies), ShopError> {
    log::debug!("fetch_storefront: starting version info fetch");
    let info = fetch_version_info().await?;
    log::debug!(
        "fetch_storefront: version={}, user_agent={}",
        info.client_version,
        info.user_agent
    );

    let shop_client = ShopClient::new(cookies, &info.user_agent)?;
    log::debug!("fetch_storefront: ShopClient created, fetching storefront");

    let storefront = shop_client.fetch(&info.client_version).await?;
    log::debug!(
        "fetch_storefront: storefront fetched, {} daily offers, night_market={}",
        storefront.daily_offers.len(),
        storefront.night_market.is_some()
    );

    let updated_cookies = shop_client.extract_updated_cookies();
    Ok((storefront, updated_cookies))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_from_clid() {
        assert_eq!(client::shard_from_clid("ap1"), "ap");
        assert_eq!(client::shard_from_clid("na1"), "na");
        assert_eq!(client::shard_from_clid("eu3"), "eu");
        assert_eq!(client::shard_from_clid("kr"), "kr");
        assert_eq!(client::shard_from_clid(""), "");
    }

    /// Parse RiotGamesPrivateSettings.yaml and extract all cookies.
    fn parse_yaml_cookies(path: &str) -> RiotCookies {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));

        let doc: serde_yaml::Value = serde_yaml::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse YAML: {}", e));

        let session_cookies = doc
            .get("riot-login")
            .and_then(|v| v.get("persist"))
            .and_then(|v| v.get("session"))
            .and_then(|v| v.get("cookies"))
            .and_then(|v| v.as_sequence())
            .expect("cookies array not found in YAML");

        let mut cookies = RiotCookies {
            asid: None,
            ccid: None,
            clid: None,
            sub: None,
            csid: None,
            ssid: None,
            tdid: None,
        };

        for cookie in session_cookies {
            let name = cookie.get("name").and_then(|v| v.as_str());
            let value = cookie.get("value").and_then(|v| v.as_str());
            if let (Some(n), Some(v)) = (name, value) {
                match n {
                    "asid" => cookies.asid = Some(v.to_string()),
                    "ccid" => cookies.ccid = Some(v.to_string()),
                    "clid" => cookies.clid = Some(v.to_string()),
                    "sub" => cookies.sub = Some(v.to_string()),
                    "csid" => cookies.csid = Some(v.to_string()),
                    "ssid" => cookies.ssid = Some(v.to_string()),
                    _ => {}
                }
            }
        }

        cookies.tdid = doc
            .get("rso-authenticator")
            .and_then(|v| v.get("tdid"))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        cookies
    }

    /// Fetch storefront using all cookies from a RiotGamesPrivateSettings.yaml file.
    ///
    /// Run with:
    ///   TEST_YAML=path/to/RiotGamesPrivateSettings.yaml cargo test test_fetch_storefront_from_yaml -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "requires TEST_YAML env var and network access"]
    async fn test_fetch_storefront_from_yaml() {
        let yaml_path = std::env::var("TEST_YAML")
            .expect("TEST_YAML must be set to the path of RiotGamesPrivateSettings.yaml");

        let cookies = parse_yaml_cookies(&yaml_path);

        println!("Cookies loaded:");
        println!("  ssid: {}", if cookies.ssid.is_some() { "present" } else { "missing" });
        println!("  asid: {}", if cookies.asid.is_some() { "present" } else { "missing" });
        println!("  csid: {}", if cookies.csid.is_some() { "present" } else { "missing" });
        println!("  ccid: {}", if cookies.ccid.is_some() { "present" } else { "missing" });
        println!("  clid: {:?}", cookies.clid);
        println!("  sub:  {:?}", cookies.sub);
        println!("  tdid: {}", if cookies.tdid.is_some() { "present" } else { "missing" });

        let shard = cookies.clid.as_deref().map(client::shard_from_clid).unwrap_or("ap");
        println!("  shard (derived): {}", shard);

        let result = fetch_storefront(cookies).await;
        assert!(result.is_ok(), "Storefront fetch failed: {:?}", result.unwrap_err());

        let (sf, updated_cookies) = result.unwrap();

        println!("\n--- Updated Cookies ---");
        println!("  ssid: {}", if updated_cookies.ssid.is_some() { "present" } else { "missing" });
        println!("  tdid: {}", if updated_cookies.tdid.is_some() { "present" } else { "missing" });

        println!("\n--- Daily Shop ({} sec remaining) ---", sf.daily_remaining_secs);
        for o in &sf.daily_offers {
            println!("  {} - {} VP", o.skin_uuid, o.vp_cost);
        }

        match &sf.night_market {
            Some(nm) => {
                println!("\n--- Night Market ({} offers) ---", nm.len());
                for o in nm {
                    println!(
                        "  {} - {} VP ({}% off, base {} VP)",
                        o.skin_uuid, o.discount_cost, o.discount_percent, o.base_cost
                    );
                }
            }
            None => println!("\nNo night market active."),
        }
    }
}
