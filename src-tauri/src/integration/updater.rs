//! GitHub Releases update checker.
//!
//! Queries the public GitHub Releases API to detect newer versions of PokeTokenBar,
//! parses semver tags, and retrieves release notes and download links.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_name: String,
    pub release_url: String,
    pub release_notes: String,
    pub published_at: String,
}

/// Check the GitHub Releases API for the latest published release.
pub fn check_github_update() -> Result<Option<UpdateInfo>, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let url = "https://api.github.com/repos/aschwehm/PokeTokenBar/releases/latest";

    let resp: serde_json::Value = ureq::get(url)
        .set("User-Agent", &format!("PokeTokenBar/{}", current_version))
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .map_err(|e| format!("Network request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("Failed to parse release JSON: {e}"))?;

    let tag_name = resp
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing tag_name in release response".to_string())?;

    let remote_version = tag_name.trim_start_matches('v').trim();

    if is_newer_version(remote_version, current_version) {
        let release_name = resp
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(tag_name)
            .to_string();
        let release_url = resp
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://github.com/aschwehm/PokeTokenBar/releases/latest")
            .to_string();
        let release_notes = resp
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let published_at = resp
            .get("published_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(Some(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: remote_version.to_string(),
            release_name,
            release_url,
            release_notes,
            published_at,
        }))
    } else {
        Ok(None)
    }
}

/// Robust semver comparison checking if `remote` is strictly newer than `current`.
pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let parse_segments = |v: &str| -> Vec<u32> {
        let clean = v.trim().trim_start_matches('v');
        clean
            .split('.')
            .map(|seg| {
                seg.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0)
            })
            .collect()
    };

    let r_parts = parse_segments(remote);
    let c_parts = parse_segments(current);

    let max_len = r_parts.len().max(c_parts.len());
    for i in 0..max_len {
        let r = r_parts.get(i).copied().unwrap_or(0);
        let c = c_parts.get(i).copied().unwrap_or(0);
        if r > c {
            return true;
        } else if r < c {
            return false;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.4.1", "0.4.0"));
        assert!(is_newer_version("0.5.0", "0.4.0"));
        assert!(is_newer_version("1.0.0", "0.4.0"));
        assert!(is_newer_version("v0.4.1", "0.4.0"));
        assert!(is_newer_version("0.4.1-rc1", "0.4.0"));
        assert!(is_newer_version("0.4.0.1", "0.4.0"));

        assert!(!is_newer_version("0.4.0", "0.4.0"));
        assert!(!is_newer_version("v0.4.0", "0.4.0"));
        assert!(!is_newer_version("0.3.9", "0.4.0"));
        assert!(!is_newer_version("0.3.99", "0.4.0"));
        assert!(!is_newer_version("0.1.0", "0.4.0"));
    }
}
