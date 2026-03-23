//! Check for application updates from GitHub Releases

use anyhow::{Context, Result};
use serde::Deserialize;

const GITHUB_API_URL: &str = "https://api.github.com/repos/whooof/brewsty/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// GitHub release response
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub published_at: Option<String>,
}

/// Update check result
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
    pub release_notes: String,
}

/// Check for updates from GitHub
pub async fn check_for_updates() -> Result<Option<UpdateCheckResult>> {
    let client = reqwest::Client::builder()
        .user_agent("brewsty-update-checker")
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .context("Failed to fetch latest release")?;

    if !response.status().is_success() {
        anyhow::bail!("GitHub API returned status: {}", response.status());
    }

    let release: GitHubRelease = response
        .json()
        .await
        .context("Failed to parse release response")?;

    let latest_version = release.tag_name.trim_start_matches('v');

    if is_newer_version(latest_version, CURRENT_VERSION) {
        Ok(Some(UpdateCheckResult {
            update_available: true,
            current_version: CURRENT_VERSION.to_string(),
            latest_version: latest_version.to_string(),
            release_url: release.html_url,
            release_notes: release.body,
        }))
    } else {
        Ok(None)
    }
}

/// Compare two version strings
fn is_newer_version(latest: &str, current: &str) -> bool {
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("0.8.0", "0.7.0"));
        assert!(is_newer_version("0.7.1", "0.7.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("0.7.0", "0.7.0"));
        assert!(!is_newer_version("0.6.0", "0.7.0"));
    }
}
