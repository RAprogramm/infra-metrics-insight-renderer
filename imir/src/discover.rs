// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Discovers repositories using IMIR badges through GitHub Code Search API.
///
/// Searches for repositories referencing badge URLs from the configured
/// metrics repository and returns their owner/repository identifiers.
use std::collections::HashSet;

use indicatif::{ProgressBar, ProgressStyle};
use masterror::AppError;
use octocrab::{Octocrab, models::Code};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

const BADGE_URL_PATTERN: &str = "RAprogramm/infra-metrics-insight-renderer";
const METRICS_PATH_PATTERN: &str = "/metrics/";
const IMIR_REPO_OWNER: &str = "RAprogramm";
const IMIR_REPO_NAME: &str = "infra-metrics-insight-renderer";

/// Configuration for repository discovery operations.
#[derive(Debug, Clone,)]
pub struct DiscoveryConfig
{
    /// Maximum number of pages to fetch from GitHub API (default: 10).
    pub max_pages:            u32,
    /// Badge URL pattern to search for (default:
    /// RAprogramm/infra-metrics-insight-renderer).
    pub badge_url_pattern:    String,
    /// Metrics path pattern to search for (default: /metrics/).
    pub metrics_path_pattern: String,
}

impl Default for DiscoveryConfig
{
    fn default() -> Self
    {
        Self {
            max_pages:            10,
            badge_url_pattern:    BADGE_URL_PATTERN.to_string(),
            metrics_path_pattern: METRICS_PATH_PATTERN.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct DiscoveredRepository
{
    pub owner:      String,
    pub repository: String,
}

impl std::fmt::Display for DiscoveredRepository
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_,>,) -> std::fmt::Result
    {
        write!(f, "{}/{}", self.owner, self.repository)
    }
}

/// Discovers repositories using IMIR badges via GitHub Code Search API.
///
/// # Arguments
///
/// * `token` - GitHub personal access token for API authentication
/// * `config` - Discovery configuration (max pages, search patterns)
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
/// let token = std::env::var("GITHUB_TOKEN",).unwrap();
/// let config = DiscoveryConfig::default();
/// let repos = discover_badge_users(&token, &config,).await?;
/// for repo in repos {
///     println!("Found: {}", repo);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn discover_badge_users(
    token: &str,
    config: &DiscoveryConfig,
) -> Result<Vec<DiscoveredRepository,>, AppError,>
{
    debug!("Initializing GitHub client for badge discovery");
    let octocrab = Octocrab::builder().personal_token(token,).build().map_err(|e| {
        AppError::unauthorized(format!("failed to initialize GitHub client: {e}"),)
    },)?;

    let query = format!("{} {}", config.badge_url_pattern, config.metrics_path_pattern);
    info!("Searching for repositories using badge pattern: {}", query);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}",)
            .expect("valid template",),
    );
    pb.set_message("Searching for badge users...",);

    let mut discovered = Vec::with_capacity(100,);
    let mut seen = HashSet::with_capacity(100,);
    let mut page = 1u32;

    loop {
        pb.set_message(format!("Searching page {}/{}...", page, config.max_pages),);
        debug!("Fetching page {} of search results", page);
        let search_result = octocrab
            .search()
            .code(&query,)
            .page(page,)
            .send()
            .await
            .map_err(|e| AppError::service(format!("GitHub code search failed: {e}"),),)?;

        let items_count = search_result.items.len();
        debug!("Found {} items on page {}", items_count, page);

        for item in &search_result.items {
            if let Some(repo_info,) = extract_repository_info(item,) {
                let key = (repo_info.owner.clone(), repo_info.repository.clone(),);
                if seen.insert(key,) {
                    debug!("Discovered new repository: {}", repo_info);
                    discovered.push(repo_info,);
                    pb.set_message(format!(
                        "Found {} repositories (page {}/{})...",
                        discovered.len(),
                        page,
                        config.max_pages
                    ),);
                }
            }
        }

        if items_count == 0 || page >= config.max_pages {
            break;
        }

        page += 1;
    }

    pb.finish_with_message(format!(
        "Badge discovery complete: {} repositories found",
        discovered.len()
    ),);
    info!("Badge discovery complete: {} repositories found", discovered.len());
    Ok(discovered,)
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
/// let token = std::env::var("GITHUB_TOKEN",).unwrap();
/// let config = DiscoveryConfig::default();
/// let repos = discover_stargazer_repositories(&token, &config,).await?;
/// for repo in repos {
///     println!("Found: {}", repo);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn discover_stargazer_repositories(
    token: &str,
    config: &DiscoveryConfig,
) -> Result<Vec<DiscoveredRepository,>, AppError,>
{
    debug!("Initializing GitHub client for stargazer discovery");
    let octocrab = Octocrab::builder().personal_token(token,).build().map_err(|e| {
        AppError::unauthorized(format!("failed to initialize GitHub client: {e}"),)
    },)?;

    info!("Discovering repositories from stargazers of {}/{}", IMIR_REPO_OWNER, IMIR_REPO_NAME);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} [{elapsed_precise}] {msg}",)
            .expect("valid template",),
    );
    pb.set_message("Fetching stargazers...",);

    let mut discovered = Vec::with_capacity(500,);
    let mut seen = HashSet::with_capacity(500,);
    let mut page = 1u32;

    loop {
        pb.set_message(format!("Fetching stargazers page {}/{}...", page, config.max_pages),);
        debug!("Fetching page {} of stargazers", page);
        let stargazers = octocrab
            .repos(IMIR_REPO_OWNER, IMIR_REPO_NAME,)
            .list_stargazers()
            .per_page(100,)
            .page(page,)
            .send()
            .await
            .map_err(|e| AppError::service(format!("failed to fetch stargazers: {e}"),),)?;

        let items_count = stargazers.items.len();
        debug!("Processing {} stargazers on page {}", items_count, page);

        for (idx, stargazer,) in stargazers.items.iter().enumerate() {
            let user = match &stargazer.user {
                Some(u,) => u,
                None => continue,
            };
            let username = &user.login;
            pb.set_message(format!(
                "Processing stargazer {}/{} on page {}...",
                idx + 1,
                items_count,
                page
            ),);
            debug!("Fetching repositories for user: {}", username);

            let user_repos = octocrab
                .users(username,)
                .repos()
                .per_page(100,)
                .page(1u32,)
                .send()
                .await
                .map_err(|e| {
                    AppError::service(format!("failed to fetch repos for {username}: {e}"),)
                },)?;

            for repo in &user_repos.items {
                if repo.fork.unwrap_or(false,) {
                    continue;
                }

                let repo_info = DiscoveredRepository {
                    owner:      username.to_string(),
                    repository: repo.name.clone(),
                };

                let key = (repo_info.owner.clone(), repo_info.repository.clone(),);
                if seen.insert(key,) {
                    debug!("Discovered repository: {}", repo_info);
                    discovered.push(repo_info,);
                    pb.set_message(format!(
                        "Found {} repositories (processing page {}/{})...",
                        discovered.len(),
                        page,
                        config.max_pages
                    ),);
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
    ),);
    info!("Stargazer discovery complete: {} repositories found", discovered.len());
    Ok(discovered,)
}

fn extract_repository_info(code: &Code,) -> Option<DiscoveredRepository,>
{
    let repo_url = code.repository.html_url.as_ref()?;
    let parts: Vec<&str,> = repo_url.path_segments()?.collect();

    if parts.len() >= 2 {
        Some(DiscoveredRepository {
            owner:      parts[0].to_string(),
            repository: parts[1].to_string(),
        },)
    } else {
        None
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn discovered_repository_display()
    {
        let repo = DiscoveredRepository {
            owner:      "testowner".to_string(),
            repository: "testrepo".to_string(),
        };
        assert_eq!(repo.to_string(), "testowner/testrepo");
    }

    #[test]
    fn discovered_repository_clone()
    {
        let repo = DiscoveredRepository {
            owner:      "owner".to_string(),
            repository: "repo".to_string(),
        };
        let cloned = repo.clone();
        assert_eq!(repo.owner, cloned.owner);
        assert_eq!(repo.repository, cloned.repository);
    }

    #[tokio::test]
    async fn discover_badge_users_fails_with_invalid_token()
    {
        let config = DiscoveryConfig::default();
        let result = discover_badge_users("invalid_token", &config,).await;
        assert!(result.is_err(), "should fail with invalid token",);
    }

    #[tokio::test]
    async fn discover_stargazer_repositories_fails_with_invalid_token()
    {
        let config = DiscoveryConfig::default();
        let result = discover_stargazer_repositories("invalid_token", &config,).await;
        assert!(result.is_err(), "should fail with invalid token",);
    }

    #[test]
    fn discovery_config_default_values()
    {
        let config = DiscoveryConfig::default();
        assert_eq!(config.max_pages, 10);
        assert_eq!(config.badge_url_pattern, "RAprogramm/infra-metrics-insight-renderer");
        assert_eq!(config.metrics_path_pattern, "/metrics/");
    }

    #[test]
    fn discovery_config_custom_values()
    {
        let config = DiscoveryConfig {
            max_pages:            5,
            badge_url_pattern:    "custom/repo".to_string(),
            metrics_path_pattern: "/custom/".to_string(),
        };
        assert_eq!(config.max_pages, 5);
        assert_eq!(config.badge_url_pattern, "custom/repo");
        assert_eq!(config.metrics_path_pattern, "/custom/");
    }
}
