// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Contributor activity tracking for repository metrics.
///
/// Fetches and aggregates contributor statistics from GitHub API,
/// providing last 30 days activity metrics per contributor.
use masterror::AppError;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::retry::{RetryConfig, retry_with_backoff};

/// GitHub API contributor statistics response structure.
#[derive(Debug, Clone, Deserialize)]
struct ContributorStats {
    pub weeks:  Vec<WeeklyStats>,
    pub author: Author
}

/// Weekly contribution statistics.
#[derive(Debug, Clone, Deserialize)]
struct WeeklyStats {
    pub w: i64,
    pub a: u32,
    pub d: u32,
    pub c: u32
}

/// Contributor author information.
#[derive(Debug, Clone, Deserialize)]
struct Author {
    pub login:      String,
    pub avatar_url: String,
    #[serde(rename = "type")]
    pub user_type:  String
}

/// Aggregated contributor activity for last 30 days.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorActivity {
    pub login:      String,
    pub avatar_url: String,
    pub commits:    u32,
    pub additions:  u32,
    pub deletions:  u32,
    pub is_bot:     bool
}

impl std::fmt::Display for ContributorActivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} commits, +{} -{} lines)",
            self.login, self.commits, self.additions, self.deletions
        )
    }
}

/// Fetches contributor activity for the last 30 days from a GitHub repository.
///
/// # Arguments
///
/// * `octocrab` - Authenticated Octocrab client
/// * `owner` - Repository owner
/// * `repo` - Repository name
/// * `retry_config` - Retry configuration for API calls
///
/// # Errors
///
/// Returns [`AppError`] when GitHub API requests fail.
///
/// # Example
///
/// ```no_run
/// use imir::{contributors::fetch_contributor_activity, retry::RetryConfig};
/// use masterror::AppError;
/// use octocrab::Octocrab;
///
/// # async fn example() -> Result<(), AppError> {
/// let octocrab = Octocrab::builder()
///     .personal_token("token")
///     .build()
///     .map_err(|e| AppError::service(format!("failed to build octocrab: {e}")))?;
/// let config = RetryConfig::default();
/// let activity = fetch_contributor_activity(&octocrab, "owner", "repo", &config).await?;
/// for contributor in activity {
///     println!("{}", contributor);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch_contributor_activity(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    retry_config: &RetryConfig
) -> Result<Vec<ContributorActivity>, AppError> {
    debug!("Fetching contributor stats for {}/{}", owner, repo);

    let octocrab_clone = octocrab.clone();
    let owner_str = owner.to_string();
    let repo_str = repo.to_string();

    let stats: Vec<ContributorStats> = retry_with_backoff(
        retry_config,
        &format!("contributor stats for {owner}/{repo}"),
        || {
            let octocrab = octocrab_clone.clone();
            let owner = owner_str.clone();
            let repo = repo_str.clone();
            async move {
                octocrab
                    .get(
                        format!("/repos/{owner}/{repo}/stats/contributors"),
                        None::<&()>
                    )
                    .await
                    .map_err(|e| {
                        AppError::service(format!("failed to fetch contributor stats: {e}"))
                    })
            }
        }
    )
    .await?;

    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AppError::internal(format!("system time error: {e}")))?
            .as_secs()
    )
    .unwrap_or(i64::MAX);

    let thirty_days_ago = now - (30 * 24 * 60 * 60);

    let mut activities = Vec::with_capacity(stats.len());

    for stat in stats {
        let recent_weeks: Vec<&WeeklyStats> = stat
            .weeks
            .iter()
            .filter(|w| w.w >= thirty_days_ago)
            .collect();

        if recent_weeks.is_empty() {
            continue;
        }

        let commits: u32 = recent_weeks.iter().map(|w| w.c).sum();
        let additions: u32 = recent_weeks.iter().map(|w| w.a).sum();
        let deletions: u32 = recent_weeks.iter().map(|w| w.d).sum();

        if commits == 0 {
            continue;
        }

        activities.push(ContributorActivity {
            login: stat.author.login,
            avatar_url: stat.author.avatar_url,
            commits,
            additions,
            deletions,
            is_bot: stat.author.user_type == "Bot"
        });
    }

    activities.sort_by_key(|a| std::cmp::Reverse(a.commits));

    info!(
        "Found {} active contributors in last 30 days for {}/{}",
        activities.len(),
        owner,
        repo
    );

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributor_activity_display_format() {
        let activity = ContributorActivity {
            login:      "testuser".to_string(),
            avatar_url: "https://example.com/avatar.png".to_string(),
            commits:    15,
            additions:  250,
            deletions:  80,
            is_bot:     false
        };

        assert_eq!(
            activity.to_string(),
            "testuser (15 commits, +250 -80 lines)"
        );
    }

    #[test]
    fn contributor_activity_serialization() {
        let activity = ContributorActivity {
            login:      "contributor".to_string(),
            avatar_url: "https://example.com/avatar.png".to_string(),
            commits:    5,
            additions:  100,
            deletions:  20,
            is_bot:     false
        };

        let json = serde_json::to_string(&activity).expect("serialization failed");
        assert!(json.contains("contributor"));
        assert!(json.contains("\"commits\":5"));

        let deserialized: ContributorActivity =
            serde_json::from_str(&json).expect("deserialization failed");
        assert_eq!(activity.login, deserialized.login);
        assert_eq!(activity.commits, deserialized.commits);
    }

    #[test]
    fn contributor_activity_identifies_bots() {
        let bot_activity = ContributorActivity {
            login:      "dependabot[bot]".to_string(),
            avatar_url: "https://example.com/bot.png".to_string(),
            commits:    3,
            additions:  50,
            deletions:  10,
            is_bot:     true
        };

        assert!(bot_activity.is_bot);
    }

    fn fast_retry() -> RetryConfig {
        RetryConfig {
            max_attempts:     1,
            initial_delay_ms: 0,
            backoff_factor:   1.0
        }
    }

    fn mock_octocrab(server: &wiremock::MockServer) -> Octocrab {
        Octocrab::builder()
            .personal_token("test-token")
            .base_uri(server.uri())
            .expect("base_uri")
            .build()
            .expect("octocrab build")
    }

    #[tokio::test]
    async fn fetch_contributor_activity_aggregates_recent_weeks() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path}
        };

        let server = MockServer::start().await;
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs();
        let recent_week = now_secs.saturating_sub(7 * 86400);
        let stale_week = now_secs.saturating_sub(60 * 86400);
        let body = format!(
            r#"[
                {{
                    "author": {{ "login": "alice", "avatar_url": "https://example.com/a.png", "type": "User" }},
                    "weeks": [
                        {{ "w": {recent_week}, "a": 100, "d": 20, "c": 5 }},
                        {{ "w": {stale_week}, "a": 999, "d": 999, "c": 99 }}
                    ]
                }},
                {{
                    "author": {{ "login": "bot[bot]", "avatar_url": "https://example.com/b.png", "type": "Bot" }},
                    "weeks": [
                        {{ "w": {recent_week}, "a": 1, "d": 1, "c": 1 }}
                    ]
                }}
            ]"#
        );
        Mock::given(method("GET"))
            .and(path("/repos/octo/cat/stats/contributors"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&server)
            .await;

        let octocrab = mock_octocrab(&server);
        let activities = fetch_contributor_activity(&octocrab, "octo", "cat", &fast_retry())
            .await
            .expect("fetch should succeed");

        assert_eq!(activities.len(), 2);
        assert_eq!(activities[0].login, "alice");
        assert_eq!(activities[0].commits, 5);
        assert_eq!(activities[0].additions, 100);
        assert!(!activities[0].is_bot);
        assert_eq!(activities[1].login, "bot[bot]");
        assert!(activities[1].is_bot);
    }

    #[tokio::test]
    async fn fetch_contributor_activity_skips_contributors_without_recent_commits() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path}
        };

        let server = MockServer::start().await;
        let stale_week = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .saturating_sub(120 * 86400);
        let body = format!(
            r#"[
                {{
                    "author": {{ "login": "ghost", "avatar_url": "https://example.com/g.png", "type": "User" }},
                    "weeks": [
                        {{ "w": {stale_week}, "a": 1, "d": 1, "c": 7 }}
                    ]
                }}
            ]"#
        );
        Mock::given(method("GET"))
            .and(path("/repos/octo/cat/stats/contributors"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&server)
            .await;

        let octocrab = mock_octocrab(&server);
        let activities = fetch_contributor_activity(&octocrab, "octo", "cat", &fast_retry())
            .await
            .expect("fetch should succeed");
        assert!(activities.is_empty());
    }
}
