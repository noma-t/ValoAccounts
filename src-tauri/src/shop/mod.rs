mod client;
mod error;
mod storefront;
mod types;
mod version;

pub use error::ShopError;
#[allow(unused_imports)]
pub use types::{DailyOffer, NightMarketOffer, Storefront};

use client::ShopClient;
use version::fetch_version_info;

/// Fetch the Valorant daily shop and night market for a given session.
///
/// # Arguments
/// * `ssid` - Value of the `ssid` cookie from `auth.riotgames.com`.
/// * `shard` - Region shard (e.g. `"ap"` for Asia-Pacific, `"na"` for North America).
pub async fn fetch_storefront(
    ssid: impl Into<String>,
    shard: impl Into<String>,
) -> Result<Storefront, ShopError> {
    let info = fetch_version_info().await?;
    let shop_client = ShopClient::new(ssid, shard, &info.user_agent)?;
    shop_client.fetch(&info.client_version).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run with:
    ///   TEST_SSID=<your_ssid> TEST_SHARD=ap cargo test test_fetch_storefront_live -- --ignored
    #[tokio::test]
    #[ignore = "requires TEST_SSID env var and network access"]
    async fn test_fetch_storefront_live() {
        let ssid = std::env::var("TEST_SSID").expect("TEST_SSID must be set");
        let shard = std::env::var("TEST_SHARD").unwrap_or_else(|_| "ap".to_string());

        let result = fetch_storefront(ssid, shard).await;
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
