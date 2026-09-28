mod cache;
mod client;
mod error;
mod region;
mod storefront;
mod types;
mod version;

pub use cache::{load_cached_storefront, save_storefront_cache};
pub use error::ShopError;
#[allow(unused_imports)]
pub use types::{AccessoryOffer, Bundle, BundleItem, DailyOffer, NightMarketOffer, RiotCookies, Storefront};

use client::ShopClient;
use version::fetch_version_info;

/// Fetch the Valorant daily shop and night market using account cookies.
///
/// # Arguments
/// * `cookies` - Riot account session parsed from RiotGamesPrivateSettings.yaml.
/// * `region_setting` - The Region setting, used as a shard fallback.
///
/// The shard is resolved from `clid` (e.g. "ap1" -> "ap"), then the `id_token`
/// `lol_region` claim, then `region_setting`. The PUUID comes from `sub` or
/// the userinfo endpoint.
pub async fn fetch_storefront(
    cookies: RiotCookies,
    region_setting: Option<&str>,
) -> Result<(Storefront, RiotCookies), ShopError> {
    log::debug!("fetch_storefront: starting version info fetch");
    let info = fetch_version_info().await?;
    log::debug!(
        "fetch_storefront: version={}, user_agent={}",
        info.client_version,
        info.user_agent
    );

    let mut shop_client = ShopClient::new(cookies, &info.user_agent, region_setting)?;
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

    /// Parse RiotGamesPrivateSettings.yaml and extract all cookies.
    fn parse_yaml_cookies(path: &str) -> RiotCookies {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));

        let doc: serde_yaml::Value = serde_yaml::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse YAML: {}", e));

        let cookies = crate::commands::cookies::parse_riot_cookies(&doc);
        assert!(cookies.has_session(), "no session found in YAML");
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
        println!("  refresh_token: {}", if cookies.refresh_token.is_some() { "present" } else { "missing" });

        let region_setting = std::env::var("TEST_REGION").ok();
        let result = fetch_storefront(cookies, region_setting.as_deref()).await;
        assert!(result.is_ok(), "Storefront fetch failed: {:?}", result.unwrap_err());

        let (sf, updated_cookies) = result.unwrap();

        println!("\n--- Updated Cookies ---");
        println!("  ssid: {}", if updated_cookies.ssid.is_some() { "present" } else { "missing" });
        println!("  tdid: {}", if updated_cookies.tdid.is_some() { "present" } else { "missing" });
        println!("  refresh_token rotated: {}", updated_cookies.refresh_token.is_some());

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
