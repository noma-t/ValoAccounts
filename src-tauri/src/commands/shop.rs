use super::cookies::save_account_cookies;

/// Fetch the daily shop and night market.
///
/// When `force` is `false` a valid cached result is returned without hitting
/// the API.  When `force` is `true` the cache is bypassed and the API is
/// always called (used by the manual refresh button).
#[tauri::command]
pub async fn get_shop(
    account_id: i64,
    cookies: crate::shop::RiotCookies,
    force: bool,
) -> Result<crate::shop::Storefront, String> {
    log::debug!(
        "get_shop: called for account {} (force={})",
        account_id,
        force
    );

    if !force {
        if let Some(cached) = crate::shop::load_cached_storefront(account_id) {
            log::debug!(
                "get_shop: returning cached storefront for account {}",
                account_id
            );
            return Ok(cached);
        }
    } else {
        log::debug!(
            "get_shop: force refresh, skipping cache for account {}",
            account_id
        );
    }

    log::debug!("get_shop: fetching storefront for account {}", account_id);
    let (storefront, updated_cookies) = crate::shop::fetch_storefront(cookies)
        .await
        .map_err(|e| e.to_string())?;

    log::debug!("get_shop: storefront fetched, saving cache");
    crate::shop::save_storefront_cache(account_id, &storefront);

    log::debug!("get_shop: persisting updated cookies to YAML");
    if let Err(e) = save_account_cookies(account_id, &updated_cookies) {
        log::warn!(
            "Failed to save updated cookies for account {}: {}",
            account_id,
            e
        );
    }

    Ok(storefront)
}
