// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Discovers repositories using IMIR badges through README parsing.
///
/// Scans repositories from stargazers and checks README files for badge
/// presence and metrics links to identify repositories using IMIR.
use std::collections::HashSet;

use indicatif::{ProgressBar, ProgressStyle};
use masterror::AppError;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::retry::{RetryConfig, retry_with_backoff};

const BADGE_SVG_FILENAME: &str = "badge.svg";
const IMIR_REPO_OWNER: &str = "RAprogramm";
const IMIR_REPO_NAME: &str = "infra-metrics-insight-renderer";

/// Configuration for repository discovery operations.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Maximum number of pages to fetch from GitHub API (default: 10).
    pub max_pages:    u32,
    /// Retry configuration for API calls.
    pub retry_config: RetryConfig
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            max_pages:    10,
            retry_config: RetryConfig::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredRepository {
    pub owner:      String,
    pub repository: String
}

impl std::fmt::Display for DiscoveredRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.repository)
    }
}

/// Discovers repositories using IMIR badges via stargazers.
///
/// This is an alias for [`discover_stargazer_repositories`] to maintain
/// backward compatibility with existing code.
///
/// # Arguments
///
/// * `token` - GitHub personal access token for API authentication
/// * `config` - Discovery configuration (max pages to fetch)
///
/// # Errors
///
/// Returns [`AppError`] when GitHub API requests fail or authentication fails.
///
/// # Example
///
/// ```no_run
/// use imir::{DiscoveryConfig, discover_badge_users};
///
/// # async fn example() -> Result<(), masterror::AppError> {
/// let token = std::env::var("GITHUB_TOKEN").unwrap();
/// let config = DiscoveryConfig::default();
/// let repos = discover_badge_users(&token, &config).await?;
/// for repo in repos {
///     println!("Found: {}", repo);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn discover_badge_users(
    token: &str,
    config: &DiscoveryConfig
) -> Result<Vec<DiscoveredRepository>, AppError> {
    discover_stargazer_repositories(token, config).await
}

/// Fetches README content from a repository and checks for IMIR badge.
///
/// # Arguments
///
/// * `octocrab` - Authenticated GitHub API client
/// * `owner` - Repository owner
/// * `repo` - Repository name
/// * `retry_config` - Retry configuration for API calls
///
/// # Returns
///
/// Repository name extracted from metrics link if badge is present, None
/// otherwise.
///
/// # Errors
///
/// Returns [`AppError`] when README fetch fails or API errors occur.
async fn check_repo_has_badge(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    retry_config: &RetryConfig
) -> Result<Option<String>, AppError> {
    let octocrab_clone = octocrab.clone();
    let owner_str = owner.to_string();
    let repo_str = repo.to_string();

    let readme_result = retry_with_backoff(
        retry_config,
        &format!("README for {}/{}", owner, repo),
        || {
            let octocrab = octocrab_clone.clone();
            let owner = owner_str.clone();
            let repo = repo_str.clone();
            async move {
                octocrab
                    .repos(&owner, &repo)
                    .get_readme()
                    .send()
                    .await
                    .map_err(|e| AppError::service(format!("failed to fetch README: {e}")))
            }
        }
    )
    .await;

    match readme_result {
        Ok(content) => {
            if let Some(decoded) = content.decoded_content() {
                Ok(extract_repo_from_readme(&decoded))
            } else {
                Ok(None)
            }
        }
        Err(_) => Ok(None)
    }
}

/// Discovers repositories from users who starred the IMIR repository.
///
/// # Arguments
///
/// * `token` - GitHub personal access token for API authentication
/// * `config` - Discovery configuration (max pages to fetch)
///
/// # Errors
///
/// Returns [`AppError`] when GitHub API requests fail or authentication fails.
///
/// # Example
///
/// ```no_run
/// use imir::{DiscoveryConfig, discover_stargazer_repositories};
///
/// # async fn example() -> Result<(), masterror::AppError> {
/// let token = std::env::var("GITHUB_TOKEN").unwrap();
/// let config = DiscoveryConfig::default();
/// let repos = discover_stargazer_repositories(&token, &config).await?;
/// for repo in repos {
///     println!("Found: {}", repo);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn discover_stargazer_repositories(
    token: &str,
    config: &DiscoveryConfig
) -> Result<Vec<DiscoveredRepository>, AppError> {
    debug!("Initializing GitHub client for stargazer discovery");
    let octocrab = Octocrab::builder()
        .personal_token(token)
        .build()
        .map_err(|e| AppError::unauthorized(format!("failed to initialize GitHub client: {e}")))?;

    info!(
        "Discovering repositories from stargazers of {}/{}",
        IMIR_REPO_OWNER, IMIR_REPO_NAME
    );

    let pb = ProgressBar::new_spinner();
    if let Ok(style) =
        ProgressStyle::default_spinner().template("{spinner:.cyan} [{elapsed_precise}] {msg}")
    {
        pb.set_style(style);
    }
    pb.set_message("Fetching stargazers...");

    let mut discovered = Vec::with_capacity(500);
    let mut seen = HashSet::with_capacity(500);
    let mut page = 1u32;

    loop {
        pb.set_message(format!(
            "Fetching stargazers page {}/{}...",
            page, config.max_pages
        ));
        debug!("Fetching page {} of stargazers", page);

        let octocrab_clone = octocrab.clone();
        let stargazers = retry_with_backoff(
            &config.retry_config,
            &format!("stargazers page {}", page),
            || {
                let octocrab = octocrab_clone.clone();
                async move {
                    octocrab
                        .repos(IMIR_REPO_OWNER, IMIR_REPO_NAME)
                        .list_stargazers()
                        .per_page(100)
                        .page(page)
                        .send()
                        .await
                        .map_err(|e| AppError::service(format!("failed to fetch stargazers: {e}")))
                }
            }
        )
        .await?;

        let items_count = stargazers.items.len();
        debug!("Processing {} stargazers on page {}", items_count, page);

        for (idx, stargazer) in stargazers.items.iter().enumerate() {
            let user = match &stargazer.user {
                Some(u) => u,
                None => continue
            };
            let username = &user.login;
            pb.set_message(format!(
                "Processing stargazer {}/{} on page {}...",
                idx + 1,
                items_count,
                page
            ));
            debug!("Fetching repositories for user: {}", username);

            let octocrab_clone = octocrab.clone();
            let username_clone = username.to_string();
            let user_repos = retry_with_backoff(
                &config.retry_config,
                &format!("repos for user {}", username),
                || {
                    let octocrab = octocrab_clone.clone();
                    let username = username_clone.clone();
                    async move {
                        octocrab
                            .users(&username)
                            .repos()
                            .per_page(100)
                            .page(1u32)
                            .send()
                            .await
                            .map_err(|e| {
                                AppError::service(format!(
                                    "failed to fetch repos for {username}: {e}"
                                ))
                            })
                    }
                }
            )
            .await?;

            for repo in &user_repos.items {
                if repo.fork.unwrap_or(false) {
                    continue;
                }

                let key = (username.to_string(), repo.name.clone());
                if seen.contains(&key) {
                    continue;
                }

                pb.set_message(format!("Checking README in {}/{}...", username, repo.name));
                debug!("Checking README in {}/{}", username, repo.name);

                let has_badge =
                    check_repo_has_badge(&octocrab, username, &repo.name, &config.retry_config)
                        .await?;

                if has_badge.is_some() {
                    seen.insert(key);
                    let repo_info = DiscoveredRepository {
                        owner:      username.to_string(),
                        repository: repo.name.clone()
                    };
                    debug!("Found IMIR badge in repository: {}", repo_info);
                    discovered.push(repo_info);
                    pb.set_message(format!(
                        "Found {} repositories with badge (page {}/{})...",
                        discovered.len(),
                        page,
                        config.max_pages
                    ));
                }
            }
        }

        if items_count == 0 || page >= config.max_pages {
            break;
        }

        page += 1;
    }

    pb.finish_with_message(format!(
        "Stargazer discovery complete: {} repositories found",
        discovered.len()
    ));
    info!(
        "Stargazer discovery complete: {} repositories found",
        discovered.len()
    );
    Ok(discovered)
}

/// Extracts repository owner and name from README content.
///
/// Searches for IMIR badge and metrics link pattern, extracting the repository
/// name from the metrics SVG path.
///
/// # Arguments
///
/// * `readme_content` - Raw README file content
///
/// # Returns
///
/// Repository name if both badge and metrics link are found, None otherwise.
///
/// # Example
///
/// ```
/// use imir::extract_repo_from_readme;
///
/// let readme = r#"
/// [![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
/// ![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/my-repo.svg)
/// "#;
/// let repo = extract_repo_from_readme(readme);
/// assert_eq!(repo, Some("my-repo".to_string()));
/// ```
pub fn extract_repo_from_readme(readme_content: &str) -> Option<String> {
    if !readme_content.contains(BADGE_SVG_FILENAME) {
        return None;
    }

    for line in readme_content.lines() {
        if !line.contains(".svg") {
            continue;
        }

        for pattern in ["./metrics/", "metrics/", "/metrics/"] {
            if let Some(metrics_idx) = line.find(pattern) {
                let after_metrics = &line[metrics_idx + pattern.len()..];
                if let Some(svg_idx) = after_metrics.find(".svg") {
                    let repo_name = &after_metrics[..svg_idx];
                    if !repo_name.is_empty() && !repo_name.contains('/') {
                        return Some(repo_name.to_string());
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_repo_from_readme_finds_valid_pattern() {
        let readme = r#"
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/test-repo.svg)
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, Some("test-repo".to_string()));
    }

    #[test]
    fn extract_repo_from_readme_returns_none_without_badge() {
        let readme = r#"
![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/test-repo.svg)
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, None);
    }

    #[test]
    fn extract_repo_from_readme_returns_none_without_metrics_link() {
        let readme = r#"
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
Some other content
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, None);
    }

    #[test]
    fn extract_repo_from_readme_handles_multiline_content() {
        let readme = r#"
# My Project

[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]

Some description here.

![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/my-project.svg)

More content.
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, Some("my-project".to_string()));
    }

    #[test]
    fn extract_repo_from_readme_rejects_invalid_repo_names() {
        let readme = r#"
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/owner/repo.svg)
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, None);
    }

    #[test]
    fn extract_repo_from_readme_handles_empty_content() {
        let readme = "";
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, None);
    }

    #[test]
    fn extract_repo_from_readme_finds_first_valid_match() {
        let readme = r#"
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/first-repo.svg)
![Metrics](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/second-repo.svg)
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, Some("first-repo".to_string()));
    }

    #[test]
    fn extract_repo_from_readme_handles_relative_path_dot_slash() {
        let readme = r#"
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
![Metrics](./metrics/relative-repo.svg)
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, Some("relative-repo".to_string()));
    }

    #[test]
    fn extract_repo_from_readme_handles_relative_path_no_prefix() {
        let readme = r#"
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
![Metrics](metrics/no-prefix-repo.svg)
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, Some("no-prefix-repo".to_string()));
    }

    #[test]
    fn extract_repo_from_readme_prefers_dot_slash_over_others() {
        let readme = r#"
[![IMIR](https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/badge.svg)]
![Metrics](./metrics/dot-slash.svg)
![Metrics](metrics/no-prefix.svg)
"#;
        let result = extract_repo_from_readme(readme);
        assert_eq!(result, Some("dot-slash".to_string()));
    }

    #[test]
    fn discovered_repository_display() {
        let repo = DiscoveredRepository {
            owner:      "testowner".to_string(),
            repository: "testrepo".to_string()
        };
        assert_eq!(repo.to_string(), "testowner/testrepo");
    }

    #[test]
    fn discovered_repository_clone() {
        let repo = DiscoveredRepository {
            owner:      "owner".to_string(),
            repository: "repo".to_string()
        };
        let cloned = repo.clone();
        assert_eq!(repo.owner, cloned.owner);
        assert_eq!(repo.repository, cloned.repository);
    }

    #[tokio::test]
    async fn discover_badge_users_fails_with_invalid_token() {
        let config = DiscoveryConfig::default();
        let result = discover_badge_users("invalid_token", &config).await;
        assert!(result.is_err(), "should fail with invalid token",);
    }

    #[tokio::test]
    async fn discover_stargazer_repositories_fails_with_invalid_token() {
        let config = DiscoveryConfig::default();
        let result = discover_stargazer_repositories("invalid_token", &config).await;
        assert!(result.is_err(), "should fail with invalid token",);
    }

    #[test]
    fn discovery_config_default_values() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.max_pages, 10);
        assert_eq!(config.retry_config.max_attempts, 3);
        assert_eq!(config.retry_config.initial_delay_ms, 1000);
    }

    #[test]
    fn discovery_config_custom_values() {
        let config = DiscoveryConfig {
            max_pages:    5,
            retry_config: RetryConfig {
                max_attempts:     5,
                initial_delay_ms: 500,
                backoff_factor:   1.5
            }
        };
        assert_eq!(config.max_pages, 5);
        assert_eq!(config.retry_config.max_attempts, 5);
        assert_eq!(config.retry_config.initial_delay_ms, 500);
    }

    #[test]
    fn discovery_config_clone_creates_independent_copy() {
        let config1 = DiscoveryConfig {
            max_pages:    7,
            retry_config: RetryConfig::default()
        };
        let config2 = config1.clone();
        assert_eq!(config1.max_pages, config2.max_pages);
    }

    #[test]
    fn discovery_config_debug_format() {
        let config = DiscoveryConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("DiscoveryConfig"));
        assert!(debug_str.contains("max_pages"));
    }

    #[test]
    fn discovered_repository_serialization() {
        let repo = DiscoveredRepository {
            owner:      "testowner".to_string(),
            repository: "testrepo".to_string()
        };
        let json = serde_json::to_string(&repo).expect("serialization failed");
        assert!(json.contains("testowner"));
        assert!(json.contains("testrepo"));

        let deserialized: DiscoveredRepository =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(repo.owner, deserialized.owner);
        assert_eq!(repo.repository, deserialized.repository);
    }

    #[test]
    fn discovered_repository_debug_format() {
        let repo = DiscoveredRepository {
            owner:      "owner".to_string(),
            repository: "repo".to_string()
        };
        let debug_str = format!("{:?}", repo);
        assert!(debug_str.contains("DiscoveredRepository"));
        assert!(debug_str.contains("owner"));
        assert!(debug_str.contains("repository"));
    }
}
