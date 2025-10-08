// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Discovers repositories using IMIR badges through GitHub Code Search API.
///
/// Searches for repositories referencing badge URLs from the configured
/// metrics repository and returns their owner/repository identifiers.
use std::collections::HashSet;

use masterror::AppError;
use octocrab::{Octocrab, models::Code};
use serde::{Deserialize, Serialize};

const BADGE_URL_PATTERN: &str = "RAprogramm/infra-metrics-insight-renderer";
const METRICS_PATH_PATTERN: &str = "/metrics/";
const IMIR_REPO_OWNER: &str = "RAprogramm";
const IMIR_REPO_NAME: &str = "infra-metrics-insight-renderer";

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
///
/// # Errors
///
/// Returns [`AppError`] when GitHub API requests fail or authentication fails.
///
/// # Example
///
/// ```no_run
/// use imir::discover_badge_users;
///
/// # async fn example() -> Result<(), masterror::AppError> {
/// let token = std::env::var("GITHUB_TOKEN",).unwrap();
/// let repos = discover_badge_users(&token,).await?;
/// for repo in repos {
///     println!("Found: {}", repo);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn discover_badge_users(token: &str,) -> Result<Vec<DiscoveredRepository,>, AppError,>
{
    let octocrab = Octocrab::builder().personal_token(token,).build().map_err(|e| {
        AppError::unauthorized(format!("failed to initialize GitHub client: {e}"),)
    },)?;

    let query = format!("{BADGE_URL_PATTERN} {METRICS_PATH_PATTERN}");

    let mut discovered = Vec::with_capacity(100,);
    let mut seen = HashSet::with_capacity(100,);
    let mut page = 1u32;
    const MAX_PAGES: u32 = 10;

    loop {
        let search_result = octocrab
            .search()
            .code(&query,)
            .page(page,)
            .send()
            .await
            .map_err(|e| AppError::service(format!("GitHub code search failed: {e}"),),)?;

        let items_count = search_result.items.len();

        for item in &search_result.items {
            if let Some(repo_info,) = extract_repository_info(item,) {
                let key = (repo_info.owner.clone(), repo_info.repository.clone(),);
                if seen.insert(key,) {
                    discovered.push(repo_info,);
                }
            }
        }

        if items_count == 0 || page >= MAX_PAGES {
            break;
        }

        page += 1;
    }

    Ok(discovered,)
}

/// Discovers repositories from users who starred the IMIR repository.
///
/// # Arguments
///
/// * `token` - GitHub personal access token for API authentication
///
/// # Errors
///
/// Returns [`AppError`] when GitHub API requests fail or authentication fails.
///
/// # Example
///
/// ```no_run
/// use imir::discover_stargazer_repositories;
///
/// # async fn example() -> Result<(), masterror::AppError> {
/// let token = std::env::var("GITHUB_TOKEN",).unwrap();
/// let repos = discover_stargazer_repositories(&token,).await?;
/// for repo in repos {
///     println!("Found: {}", repo);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn discover_stargazer_repositories(
    token: &str,
) -> Result<Vec<DiscoveredRepository,>, AppError,>
{
    let octocrab = Octocrab::builder().personal_token(token,).build().map_err(|e| {
        AppError::unauthorized(format!("failed to initialize GitHub client: {e}"),)
    },)?;

    let mut discovered = Vec::with_capacity(500,);
    let mut seen = HashSet::with_capacity(500,);
    let mut page = 1u32;
    const MAX_PAGES: u32 = 10;

    loop {
        let stargazers = octocrab
            .repos(IMIR_REPO_OWNER, IMIR_REPO_NAME,)
            .list_stargazers()
            .per_page(100,)
            .page(page,)
            .send()
            .await
            .map_err(|e| AppError::service(format!("failed to fetch stargazers: {e}"),),)?;

        let items_count = stargazers.items.len();

        for stargazer in &stargazers.items {
            let user = match &stargazer.user {
                Some(u,) => u,
                None => continue,
            };
            let username = &user.login;

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
                    discovered.push(repo_info,);
                }
            }
        }

        if items_count == 0 || page >= MAX_PAGES {
            break;
        }

        page += 1;
    }

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
        let result = discover_badge_users("invalid_token",).await;
        assert!(result.is_err(), "should fail with invalid token",);
    }

    #[tokio::test]
    async fn discover_stargazer_repositories_fails_with_invalid_token()
    {
        let result = discover_stargazer_repositories("invalid_token",).await;
        assert!(result.is_err(), "should fail with invalid token",);
    }
}
