use std::time::Duration;

use serde::Deserialize;

use super::error::ShopError;

const VERSION_URL: &str = "https://valorant-api.com/v1/version";

pub(super) struct VersionInfo {
    pub(super) client_version: String,
    pub(super) user_agent: String,
}

#[derive(Deserialize)]
struct VersionApiResponse {
    data: VersionData,
}

#[derive(Deserialize)]
struct VersionData {
    #[serde(rename = "riotClientVersion")]
    riot_client_version: String,
    #[serde(rename = "riotClientBuild")]
    riot_client_build: String,
}

/// Fetch the current Valorant client version and build a matching User-Agent.
///
/// Uses a throwaway `reqwest::Client` (no cookies needed for this public API).
/// Returns an error if the API is unreachable or returns unexpected data.
pub(super) async fn fetch_version_info() -> Result<VersionInfo, ShopError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let resp = client
        .get(VERSION_URL)
        .send()
        .await
        .map_err(|e| ShopError::VersionFetchFailed(format!("request failed: {}", e)))?;

    if !resp.status().is_success() {
        return Err(ShopError::VersionFetchFailed(format!(
            "API returned status {}",
            resp.status()
        )));
    }

    let api: VersionApiResponse = resp
        .json()
        .await
        .map_err(|e| ShopError::VersionFetchFailed(format!("failed to parse response: {}", e)))?;

    let user_agent = format!(
        "RiotClient/{} rso-auth (Windows;10;;Professional, x64)",
        api.data.riot_client_build
    );

    Ok(VersionInfo {
        client_version: api.data.riot_client_version,
        user_agent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_format() {
        let build = "70.0.0.4888690.4873386";
        let ua = format!(
            "RiotClient/{} rso-auth (Windows;10;;Professional, x64)",
            build
        );
        assert!(ua.starts_with("RiotClient/"));
        assert!(ua.contains("rso-auth"));
        assert!(ua.contains(build));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_version_info_live() {
        let info = fetch_version_info().await.expect("should fetch version");
        println!("Client version: {}", info.client_version);
        println!("User-Agent: {}", info.user_agent);
        assert!(!info.client_version.is_empty());
        assert!(info.user_agent.starts_with("RiotClient/"));
    }
}
