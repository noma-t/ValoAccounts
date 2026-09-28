use std::path::{Path, PathBuf};

use crate::db::{get_account, get_settings};

#[tauri::command]
pub fn get_account_cookies(
    account_id: i64,
) -> Result<Option<crate::shop::RiotCookies>, String> {
    log::info!("get_account_cookies: account {}", account_id);

    let yaml_path = match resolve_account_yaml_path(account_id)? {
        Some(path) => path,
        None => {
            log::warn!(
                "get_account_cookies: no session for account {} (YAML file not found)",
                account_id
            );
            return Ok(None);
        }
    };

    log_yaml_file_metadata(&yaml_path);

    let content = std::fs::read_to_string(&yaml_path).map_err(|e| {
        log::error!(
            "get_account_cookies: failed to read {}: {}",
            yaml_path.display(),
            e
        );
        format!("Failed to read settings file: {}", e)
    })?;

    let doc: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
        log::error!(
            "get_account_cookies: failed to parse YAML {}: {}",
            yaml_path.display(),
            e
        );
        format!("Failed to parse YAML: {}", e)
    })?;

    log_yaml_session_structure(&doc);

    let cookies = parse_riot_cookies(&doc);

    log::info!(
        "get_account_cookies: found ssid={} asid={} ccid={} clid={} sub={} csid={} tdid={} refresh_token={} id_token={}",
        cookies.ssid.is_some(),
        cookies.asid.is_some(),
        cookies.ccid.is_some(),
        cookies.clid.is_some(),
        cookies.sub.is_some(),
        cookies.csid.is_some(),
        cookies.tdid.is_some(),
        cookies.refresh_token.is_some(),
        cookies.id_token.is_some()
    );

    if !cookies.has_session() {
        log::warn!(
            "get_account_cookies: no session for account {} (neither ssid cookie nor riot-client refresh_token in {})",
            account_id,
            yaml_path.display()
        );
        return Ok(None);
    }

    Ok(Some(cookies))
}

/// Extract the session credentials from a parsed RiotGamesPrivateSettings.yaml.
///
/// Reads both the cookie session (`riot-login.persist.session.cookies`) and
/// the OAuth session (`psl.authorization.riot-client`) persisted by newer
/// Riot Client versions.
pub(crate) fn parse_riot_cookies(doc: &serde_yaml::Value) -> crate::shop::RiotCookies {
    let mut cookies = crate::shop::RiotCookies::default();

    let session_cookies = doc
        .get("riot-login")
        .and_then(|v| v.get("persist"))
        .and_then(|v| v.get("session"))
        .and_then(|v| v.get("cookies"))
        .and_then(|v| v.as_sequence());

    if let Some(cookie_list) = session_cookies {
        for cookie in cookie_list {
            let name = cookie.get("name").and_then(|v| v.as_str());
            let value = cookie.get("value").and_then(|v| v.as_str());
            // Never log cookie values; only their names and lengths
            log::debug!(
                "parse_riot_cookies: cookie entry name={:?} value_len={:?}",
                name,
                value.map(str::len)
            );
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
    }

    cookies.tdid = yaml_str(doc, &["rso-authenticator", "tdid", "value"]);

    cookies.refresh_token = yaml_str(doc, &["psl", "authorization", "riot-client", "refresh_token"]);
    cookies.id_token = yaml_str(doc, &["psl", "authorization", "riot-client", "id_token"]);

    cookies
}

/// Read a non-empty string at the given key path.
fn yaml_str(doc: &serde_yaml::Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(doc, |current, key| current.get(*key))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn log_yaml_file_metadata(yaml_path: &Path) {
    match std::fs::metadata(yaml_path) {
        Ok(meta) => {
            let modified = meta
                .modified()
                .map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339())
                .unwrap_or_else(|e| format!("unknown ({})", e));
            log::info!(
                "get_account_cookies: YAML {} size={} bytes modified={}",
                yaml_path.display(),
                meta.len(),
                modified
            );
        }
        Err(e) => log::warn!(
            "get_account_cookies: failed to read metadata of {}: {}",
            yaml_path.display(),
            e
        ),
    }
}

/// Log which keys exist along `riot-login.persist.session.cookies`,
/// so a missing session can be traced to the exact missing level.
/// A missing cookie session is expected for newer Riot Client versions,
/// which persist an OAuth session under `psl.authorization.riot-client`.
fn log_yaml_session_structure(doc: &serde_yaml::Value) {
    log::debug!("get_account_cookies: top-level keys: {:?}", mapping_keys(doc));

    let path = ["riot-login", "persist", "session", "cookies"];
    let mut current = doc;
    for (depth, key) in path.iter().enumerate() {
        match current.get(*key) {
            Some(next) => current = next,
            None => {
                log::debug!(
                    "get_account_cookies: key '{}' not found under '{}' (available keys: {:?})",
                    key,
                    path[..depth].join("."),
                    mapping_keys(current)
                );
                return;
            }
        }
    }

    match current.as_sequence() {
        Some(list) => log::info!(
            "get_account_cookies: riot-login.persist.session.cookies has {} entries",
            list.len()
        ),
        None => log::warn!(
            "get_account_cookies: riot-login.persist.session.cookies is not a sequence ({})",
            value_kind(current)
        ),
    }
}

fn mapping_keys(value: &serde_yaml::Value) -> Vec<String> {
    value
        .as_mapping()
        .map(|m| {
            m.keys()
                .map(|k| k.as_str().map_or_else(|| format!("{:?}", k), str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn value_kind(value: &serde_yaml::Value) -> &'static str {
    match value {
        serde_yaml::Value::Null => "null",
        serde_yaml::Value::Bool(_) => "bool",
        serde_yaml::Value::Number(_) => "number",
        serde_yaml::Value::String(_) => "string",
        serde_yaml::Value::Sequence(_) => "sequence",
        serde_yaml::Value::Mapping(_) => "mapping",
        serde_yaml::Value::Tagged(_) => "tagged",
    }
}

/// Resolve the path to an account's RiotGamesPrivateSettings.yaml.
pub(super) fn resolve_account_yaml_path(account_id: i64) -> Result<Option<PathBuf>, String> {
    let account = get_account(account_id)?;
    let data_folder = account.data_folder.ok_or_else(|| {
        log::warn!(
            "resolve_account_yaml_path: account {} has no data directory assigned",
            account_id
        );
        "Account has no data directory assigned".to_string()
    })?;

    let settings = get_settings()?;
    let account_data_path = match settings.account_data_path {
        Some(path) => PathBuf::from(path),
        None => crate::db::init::get_default_account_data_path()?,
    };

    let account_dir = account_data_path.join(&data_folder);
    let yaml_path = account_dir.join("RiotGamesPrivateSettings.yaml");

    log::debug!(
        "resolve_account_yaml_path: account {} data_folder={} account_dir={} (exists={})",
        account_id,
        data_folder,
        account_dir.display(),
        account_dir.exists()
    );
    log_riot_data_junction(settings.riot_client_data_path.as_deref(), &account_dir);

    if yaml_path.exists() {
        Ok(Some(yaml_path))
    } else {
        log::warn!(
            "resolve_account_yaml_path: YAML not found: {} (entries in account dir: {:?})",
            yaml_path.display(),
            list_file_names(&account_dir)
        );
        Ok(None)
    }
}

/// Log where the Riot Client data junction currently points, to tell whether
/// Riot Client is writing into this account's directory.
fn log_riot_data_junction(configured_path: Option<&str>, account_dir: &Path) {
    let riot_data_path = match configured_path {
        Some(path) => PathBuf::from(path),
        None => match crate::db::init::get_default_riot_client_data_path() {
            Ok(path) => path,
            Err(e) => {
                log::debug!("resolve_account_yaml_path: riot data path unavailable: {}", e);
                return;
            }
        },
    };

    match crate::fs::get_junction_target(&riot_data_path) {
        Ok(target) => log::debug!(
            "resolve_account_yaml_path: riot data junction {} -> {} (points to this account: {})",
            riot_data_path.display(),
            target.display(),
            paths_equal(&target, account_dir)
        ),
        Err(e) => log::debug!(
            "resolve_account_yaml_path: riot data path {} is not a junction: {}",
            riot_data_path.display(),
            e
        ),
    }
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn list_file_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Update cookie values in the YAML content string without altering formatting.
///
/// For session cookies under `riot-login.persist.session.cookies`, this finds
/// each `- name: <cookie_name>` block and replaces the `value:` line.
/// For `tdid`, it finds `rso-authenticator` > `tdid` > `value:` and replaces it.
pub(super) fn update_yaml_cookie_values(
    content: &str,
    cookies: &crate::shop::RiotCookies,
) -> String {
    log::debug!("update_yaml_cookie_values: starting YAML cookie replacement");
    let cookie_updates: &[(&str, &Option<String>)] = &[
        ("ssid", &cookies.ssid),
        ("asid", &cookies.asid),
        ("csid", &cookies.csid),
        ("ccid", &cookies.ccid),
        ("clid", &cookies.clid),
        ("sub", &cookies.sub),
    ];

    let mut result = content.to_string();

    for &(cookie_name, cookie_value) in cookie_updates {
        if let Some(new_val) = cookie_value {
            let pattern = format!(
                r#"(?m)(name:\s*"?{}"?\s*\n(?:\s+\w+:.*\n)*?\s+value:\s*)"[^"]*""#,
                regex::escape(cookie_name)
            );
            if let Ok(re) = regex::Regex::new(&pattern) {
                let had_match = re.is_match(&result);
                let replacement = new_val.clone();
                result = re
                    .replace(&result, |caps: &regex::Captures| {
                        format!("{}\"{}\"", &caps[1], replacement)
                    })
                    .to_string();
                if had_match {
                    log::debug!(
                        "update_yaml_cookie_values: replaced {} ({} chars)",
                        cookie_name,
                        new_val.len()
                    );
                } else {
                    log::debug!(
                        "update_yaml_cookie_values: no match for {} in YAML",
                        cookie_name
                    );
                }
            }
        } else {
            log::debug!(
                "update_yaml_cookie_values: skipping {} (no updated value)",
                cookie_name
            );
        }
    }

    if let Some(new_tdid) = &cookies.tdid {
        let pattern =
            r#"(?m)(rso-authenticator:\s*\n\s+tdid:\s*\n(?:\s+\w+:.*\n)*?\s+value:\s*)"[^"]*""#;
        if let Ok(re) = regex::Regex::new(pattern) {
            let had_match = re.is_match(&result);
            let replacement = new_tdid.clone();
            result = re
                .replace(&result, |caps: &regex::Captures| {
                    format!("{}\"{}\"", &caps[1], replacement)
                })
                .to_string();
            if had_match {
                log::debug!(
                    "update_yaml_cookie_values: replaced tdid ({} chars)",
                    new_tdid.len()
                );
            } else {
                log::debug!("update_yaml_cookie_values: no match for tdid in YAML");
            }
        }
    } else {
        log::debug!("update_yaml_cookie_values: skipping tdid (no updated value)");
    }

    // OAuth tokens under `psl.authorization.riot-client` (only set when rotated).
    // The trailing `:` keeps `refresh_token` from matching `refresh_token_write_count`.
    let token_updates: &[(&str, &Option<String>)] = &[
        ("refresh_token", &cookies.refresh_token),
        ("id_token", &cookies.id_token),
    ];
    for &(key, token_value) in token_updates {
        let Some(new_val) = token_value else {
            continue;
        };
        let pattern = format!(r#"(?m)^(\s+{}:\s*)"[^"]*""#, regex::escape(key));
        if let Ok(re) = regex::Regex::new(&pattern) {
            let had_match = re.is_match(&result);
            result = re
                .replace(&result, |caps: &regex::Captures| {
                    format!("{}\"{}\"", &caps[1], new_val)
                })
                .to_string();
            log::debug!(
                "update_yaml_cookie_values: {} {} ({} chars)",
                if had_match { "replaced" } else { "no match for" },
                key,
                new_val.len()
            );
        }
    }

    let changed = content != result;
    log::debug!(
        "update_yaml_cookie_values: done, content_changed={}",
        changed
    );

    result
}

pub(super) fn save_account_cookies(
    account_id: i64,
    cookies: &crate::shop::RiotCookies,
) -> Result<(), String> {
    log::debug!("save_account_cookies: starting for account {}", account_id);

    let yaml_path = match resolve_account_yaml_path(account_id)? {
        Some(path) => {
            log::debug!(
                "save_account_cookies: resolved YAML path: {}",
                path.display()
            );
            path
        }
        None => {
            log::info!(
                "Skipping cookie save for account {}: YAML file does not exist",
                account_id
            );
            return Ok(());
        }
    };

    let content = std::fs::read_to_string(&yaml_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    log::debug!(
        "save_account_cookies: read YAML file ({} bytes)",
        content.len()
    );

    let updated_content = update_yaml_cookie_values(&content, cookies);

    if content == updated_content {
        log::debug!("save_account_cookies: no changes detected, skipping write");
        return Ok(());
    }

    // Atomic write: write to a temp file, then rename over the original
    let tmp_path = yaml_path.with_extension("yaml.tmp");
    log::debug!(
        "save_account_cookies: writing {} bytes to temp file: {}",
        updated_content.len(),
        tmp_path.display()
    );
    std::fs::write(&tmp_path, &updated_content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    log::debug!("save_account_cookies: renaming temp file to YAML path");
    std::fs::rename(&tmp_path, &yaml_path)
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    log::info!(
        "save_account_cookies: successfully saved updated cookies for account {}",
        account_id
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const COOKIE_SESSION_YAML: &str = r#"riot-login:
    persist:
        session:
            cookies:
            -   domain: "auth.riotgames.com"
                name: "ssid"
                value: "old-ssid"
            -   domain: "auth.riotgames.com"
                name: "clid"
                value: "ap1"
rso-authenticator:
    tdid:
        name: "tdid"
        value: "tdid-1"
"#;

    const OAUTH_SESSION_YAML: &str = r#"psl:
    authorization:
        riot-client:
            claims: []
            id_token: "old-id"
            is_dpop_bound: false
            refresh_token: "old-rt"
            refresh_token_write_count: 1
            refresh_tokens_session_id: "session-1"
riot-login:
    persist: null
rso-authenticator:
    tdid:
        name: "tdid"
        value: "tdid-1"
"#;

    fn parse(yaml: &str) -> crate::shop::RiotCookies {
        let doc: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap();
        parse_riot_cookies(&doc)
    }

    #[test]
    fn test_parse_cookie_session() {
        let cookies = parse(COOKIE_SESSION_YAML);
        assert_eq!(cookies.ssid.as_deref(), Some("old-ssid"));
        assert_eq!(cookies.clid.as_deref(), Some("ap1"));
        assert_eq!(cookies.tdid.as_deref(), Some("tdid-1"));
        assert_eq!(cookies.refresh_token, None);
        assert!(cookies.has_session());
    }

    #[test]
    fn test_parse_oauth_session() {
        let cookies = parse(OAUTH_SESSION_YAML);
        assert_eq!(cookies.ssid, None);
        assert_eq!(cookies.refresh_token.as_deref(), Some("old-rt"));
        assert_eq!(cookies.id_token.as_deref(), Some("old-id"));
        assert_eq!(cookies.tdid.as_deref(), Some("tdid-1"));
        assert!(cookies.has_session());
    }

    #[test]
    fn test_parse_without_session() {
        let cookies = parse("riot-login:\n    persist: null\n");
        assert!(!cookies.has_session());
    }

    #[test]
    fn test_update_yaml_without_rotation_keeps_content() {
        // Unrotated tokens come back as None and must not trigger a write.
        let updated = crate::shop::RiotCookies {
            tdid: Some("tdid-1".to_string()),
            ..Default::default()
        };
        assert_eq!(update_yaml_cookie_values(OAUTH_SESSION_YAML, &updated), OAUTH_SESSION_YAML);
    }

    #[test]
    fn test_update_yaml_writes_rotated_tokens() {
        let updated = crate::shop::RiotCookies {
            refresh_token: Some("new-rt".to_string()),
            id_token: Some("new-id".to_string()),
            ..Default::default()
        };
        let result = update_yaml_cookie_values(OAUTH_SESSION_YAML, &updated);
        let expected = OAUTH_SESSION_YAML
            .replace("refresh_token: \"old-rt\"", "refresh_token: \"new-rt\"")
            .replace("id_token: \"old-id\"", "id_token: \"new-id\"");
        assert_eq!(result, expected);
        assert!(result.contains("refresh_token_write_count: 1"));
        assert!(result.contains("refresh_tokens_session_id: \"session-1\""));
    }

    #[test]
    fn test_update_yaml_replaces_ssid_cookie() {
        let updated = crate::shop::RiotCookies {
            ssid: Some("new-ssid".to_string()),
            ..Default::default()
        };
        let result = update_yaml_cookie_values(COOKIE_SESSION_YAML, &updated);
        assert!(result.contains("value: \"new-ssid\""));
        assert!(!result.contains("old-ssid"));
    }
}
