use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

/// Shard used when no other source can determine one.
const DEFAULT_SHARD: &str = "ap";

/// Derive the shard from the `clid` cookie value by stripping trailing digits.
///
/// Examples: "ap1" -> "ap", "na1" -> "na", "eu3" -> "eu", "kr" -> "kr"
pub(super) fn shard_from_clid(clid: &str) -> &str {
    clid.trim_end_matches(|c: char| c.is_ascii_digit())
}

/// Map the Region setting (the henrikdev region: eu/na/latam/br/ap/kr) to a
/// storefront shard. LATAM and BR accounts are served by the NA shard.
pub(super) fn shard_from_region_setting(region: &str) -> Option<&'static str> {
    match region.trim().to_ascii_lowercase().as_str() {
        "ap" => Some("ap"),
        "eu" => Some("eu"),
        "kr" => Some("kr"),
        "na" | "latam" | "br" => Some("na"),
        _ => None,
    }
}

/// Map a Riot platform ID (the `pid` of an `id_token` `lol_region` entry) to a
/// storefront shard.
fn shard_from_platform_id(pid: &str) -> Option<&'static str> {
    match pid.to_ascii_uppercase().as_str() {
        "JP1" | "OC1" | "SG2" | "PH2" | "TH2" | "TW2" | "VN2" => Some("ap"),
        "KR" => Some("kr"),
        "EUW1" | "EUN1" | "TR1" | "RU" | "ME1" => Some("eu"),
        "NA1" | "BR1" | "LA1" | "LA2" => Some("na"),
        _ => None,
    }
}

/// Decode the payload of a JWT without verifying its signature.
fn decode_jwt_claims(token: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Infer the shard from the `lol_region` claim of an `id_token`.
///
/// The active entry is preferred; otherwise the first entry is used.
/// Note that `lol_region` is the account's League of Legends platform, which
/// matches the Valorant region for most (but not all) accounts.
pub(super) fn shard_from_id_token(id_token: &str) -> Option<&'static str> {
    let claims = decode_jwt_claims(id_token)?;
    let regions = claims.get("lol_region")?.as_array()?;
    let entry = regions
        .iter()
        .find(|r| r.get("active").and_then(|v| v.as_bool()) == Some(true))
        .or_else(|| regions.first())?;
    let pid = entry.get("pid").and_then(|v| v.as_str())?;
    shard_from_platform_id(pid)
}

/// Resolve the storefront shard.
///
/// Priority: `clid` cookie -> `id_token` `lol_region` -> Region setting -> "ap".
pub(super) fn resolve_shard(
    clid: Option<&str>,
    id_token: Option<&str>,
    region_setting: Option<&str>,
) -> String {
    if let Some(shard) = clid.map(shard_from_clid).filter(|s| !s.is_empty()) {
        log::debug!("resolve_shard: {} (from clid cookie)", shard);
        return shard.to_string();
    }
    if let Some(shard) = id_token.and_then(shard_from_id_token) {
        log::debug!("resolve_shard: {} (from id_token lol_region)", shard);
        return shard.to_string();
    }
    if let Some(shard) = region_setting.and_then(shard_from_region_setting) {
        log::debug!("resolve_shard: {} (from Region setting)", shard);
        return shard.to_string();
    }
    log::debug!("resolve_shard: {} (default)", DEFAULT_SHARD);
    DEFAULT_SHARD.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an unsigned JWT whose payload is the given JSON.
    fn fake_jwt(payload: serde_json::Value) -> String {
        let body = URL_SAFE_NO_PAD.encode(payload.to_string());
        format!("eyJhbGciOiJub25lIn0.{}.sig", body)
    }

    #[test]
    fn test_shard_from_clid() {
        assert_eq!(shard_from_clid("ap1"), "ap");
        assert_eq!(shard_from_clid("na1"), "na");
        assert_eq!(shard_from_clid("eu3"), "eu");
        assert_eq!(shard_from_clid("kr"), "kr");
        assert_eq!(shard_from_clid(""), "");
    }

    #[test]
    fn test_shard_from_region_setting() {
        assert_eq!(shard_from_region_setting("ap"), Some("ap"));
        assert_eq!(shard_from_region_setting("eu"), Some("eu"));
        assert_eq!(shard_from_region_setting("kr"), Some("kr"));
        assert_eq!(shard_from_region_setting("na"), Some("na"));
        assert_eq!(shard_from_region_setting("latam"), Some("na"));
        assert_eq!(shard_from_region_setting("br"), Some("na"));
        assert_eq!(shard_from_region_setting(" AP "), Some("ap"));
        assert_eq!(shard_from_region_setting(""), None);
        assert_eq!(shard_from_region_setting("unknown"), None);
    }

    #[test]
    fn test_shard_from_id_token_uses_active_region() {
        let token = fake_jwt(serde_json::json!({
            "lol_region": [
                { "pid": "NA1", "active": false },
                { "pid": "JP1", "active": true }
            ]
        }));
        assert_eq!(shard_from_id_token(&token), Some("ap"));
    }

    #[test]
    fn test_shard_from_id_token_falls_back_to_first_entry() {
        let token = fake_jwt(serde_json::json!({
            "lol_region": [{ "pid": "EUW1" }]
        }));
        assert_eq!(shard_from_id_token(&token), Some("eu"));
    }

    #[test]
    fn test_shard_from_id_token_maps_platforms() {
        for (pid, shard) in [("KR", "kr"), ("BR1", "na"), ("LA2", "na"), ("OC1", "ap"), ("TR1", "eu")] {
            let token = fake_jwt(serde_json::json!({ "lol_region": [{ "pid": pid, "active": true }] }));
            assert_eq!(shard_from_id_token(&token), Some(shard), "pid {}", pid);
        }
    }

    #[test]
    fn test_shard_from_id_token_returns_none_when_unavailable() {
        assert_eq!(shard_from_id_token("not-a-jwt"), None);
        assert_eq!(shard_from_id_token(&fake_jwt(serde_json::json!({}))), None);
        assert_eq!(
            shard_from_id_token(&fake_jwt(serde_json::json!({ "lol_region": [] }))),
            None
        );
        assert_eq!(
            shard_from_id_token(&fake_jwt(serde_json::json!({ "lol_region": [{ "pid": "PBE1" }] }))),
            None
        );
    }

    #[test]
    fn test_resolve_shard_priority() {
        let kr_token = fake_jwt(serde_json::json!({ "lol_region": [{ "pid": "KR", "active": true }] }));

        assert_eq!(resolve_shard(Some("eu1"), Some(&kr_token), Some("na")), "eu");
        assert_eq!(resolve_shard(None, Some(&kr_token), Some("na")), "kr");
        assert_eq!(resolve_shard(Some(""), Some("bad"), Some("latam")), "na");
        assert_eq!(resolve_shard(None, None, Some("unknown")), "ap");
        assert_eq!(resolve_shard(None, None, None), "ap");
    }
}
