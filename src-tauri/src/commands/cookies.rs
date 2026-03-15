use std::path::PathBuf;

use crate::db::{get_account, get_settings};

#[tauri::command]
pub fn get_account_cookies(
    account_id: i64,
) -> Result<Option<crate::shop::RiotCookies>, String> {
    let yaml_path = match resolve_account_yaml_path(account_id)? {
        Some(path) => path,
        None => return Ok(None),
    };

    let content = std::fs::read_to_string(&yaml_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let doc: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;

    let session_cookies = doc
        .get("riot-login")
        .and_then(|v| v.get("persist"))
        .and_then(|v| v.get("session"))
        .and_then(|v| v.get("cookies"))
        .and_then(|v| v.as_sequence());

    let mut cookies = crate::shop::RiotCookies {
        asid: None,
        ccid: None,
        clid: None,
        sub: None,
        csid: None,
        ssid: None,
        tdid: None,
    };

    if let Some(cookie_list) = session_cookies {
        for cookie in cookie_list {
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
    }

    cookies.tdid = doc
        .get("rso-authenticator")
        .and_then(|v| v.get("tdid"))
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    if cookies.ssid.is_none() {
        return Ok(None);
    }

    Ok(Some(cookies))
}

/// Resolve the path to an account's RiotGamesPrivateSettings.yaml.
pub(super) fn resolve_account_yaml_path(account_id: i64) -> Result<Option<PathBuf>, String> {
    let account = get_account(account_id)?;
    let data_folder = account
        .data_folder
        .ok_or("Account has no data directory assigned")?;

    let settings = get_settings()?;
    let account_data_path = match settings.account_data_path {
        Some(path) => PathBuf::from(path),
        None => crate::db::init::get_default_account_data_path()?,
    };

    let yaml_path = account_data_path
        .join(&data_folder)
        .join("RiotGamesPrivateSettings.yaml");

    if yaml_path.exists() {
        Ok(Some(yaml_path))
    } else {
        Ok(None)
    }
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
