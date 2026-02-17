use crate::db;
use super::types::{DailyOffer, NightMarketOffer, Storefront};

fn current_unix_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Load a cached storefront for the given account if it has not expired.
///
/// Returns `None` when there is no cache, the cache has expired, or any
/// database / deserialization error occurs (all non-fatal).
pub fn load_cached_storefront(account_id: i64) -> Option<Storefront> {
    let conn = match db::init::get_connection(None) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Cache: failed to open db: {}", e);
            return None;
        }
    };

    let row: Option<(String, Option<String>, i64, Option<i64>)> = conn
        .query_row(
            "SELECT daily_offers_json, night_market_json, expires_at, nm_expires_at
               FROM storefront_cache
              WHERE account_id = ?1",
            [account_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .ok();

    let (daily_json, night_json, expires_at, nm_expires_at) = match row {
        Some(r) => r,
        None => {
            log::info!("Cache: miss (no entry) for account {}", account_id);
            return None;
        }
    };

    let now = current_unix_secs();
    if expires_at <= now {
        log::info!("Cache: miss (expired) for account {}", account_id);
        return None;
    }

    let daily_offers: Vec<DailyOffer> = match serde_json::from_str(&daily_json) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Cache: failed to deserialize daily_offers: {}", e);
            return None;
        }
    };

    let night_market: Option<Vec<NightMarketOffer>> = match night_json {
        Some(ref json) => match serde_json::from_str(json) {
            Ok(v) => Some(v),
            Err(e) => {
                log::warn!("Cache: failed to deserialize night_market: {}", e);
                return None;
            }
        },
        None => None,
    };

    let remaining = (expires_at - now) as u64;

    let night_market_remaining_secs = nm_expires_at
        .filter(|&ea| ea > now)
        .map(|ea| (ea - now) as u64);

    log::info!(
        "Cache: hit for account {} ({} secs remaining)",
        account_id,
        remaining
    );

    Some(Storefront {
        daily_offers,
        daily_remaining_secs: remaining,
        night_market,
        night_market_remaining_secs,
    })
}

/// Persist the storefront result so subsequent calls can skip the API.
///
/// Errors are logged but never propagated -- caching is best-effort.
pub fn save_storefront_cache(account_id: i64, storefront: &Storefront) {
    let conn = match db::init::get_connection(None) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Cache: failed to open db for save: {}", e);
            return;
        }
    };

    let daily_json = match serde_json::to_string(&storefront.daily_offers) {
        Ok(j) => j,
        Err(e) => {
            log::warn!("Cache: failed to serialize daily_offers: {}", e);
            return;
        }
    };

    let night_json: Option<String> = storefront.night_market.as_ref().and_then(|nm| {
        serde_json::to_string(nm)
            .map_err(|e| log::warn!("Cache: failed to serialize night_market: {}", e))
            .ok()
    });

    let now = current_unix_secs();
    let expires_at = now + storefront.daily_remaining_secs as i64;
    let nm_expires_at: Option<i64> = storefront
        .night_market_remaining_secs
        .map(|secs| now + secs as i64);

    let result = conn.execute(
        "INSERT INTO storefront_cache (account_id, daily_offers_json, night_market_json, expires_at, nm_expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(account_id) DO UPDATE SET
             daily_offers_json = excluded.daily_offers_json,
             night_market_json = excluded.night_market_json,
             expires_at = excluded.expires_at,
             nm_expires_at = excluded.nm_expires_at,
             cached_at = CURRENT_TIMESTAMP",
        rusqlite::params![account_id, daily_json, night_json, expires_at, nm_expires_at],
    );

    match result {
        Ok(_) => log::info!("Cache: saved for account {} (expires_at={})", account_id, expires_at),
        Err(e) => log::warn!("Cache: failed to save for account {}: {}", account_id, e),
    }
}
