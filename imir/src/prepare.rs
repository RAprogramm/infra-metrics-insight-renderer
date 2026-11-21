// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Preparation of the lowlighter/metrics Docker image.
//!
//! Replaces the legacy shell script with a Rust implementation that fetches the
//! latest metrics release, pulls the prebuilt image, and tags it for local use.

use std::process::Command;

use octocrab::Octocrab;
use serde::Serialize;

use crate::error::Error;

/// Status of GitHub API rate limits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RateLimitStatus {
    /// Core API limit.
    pub core_limit:        usize,
    /// Remaining core API requests.
    pub core_remaining:    usize,
    /// Core API reset timestamp (unix seconds).
    pub core_reset:        u64,
    /// Search API limit.
    pub search_limit:      usize,
    /// Remaining search API requests.
    pub search_remaining:  usize,
    /// Search API reset timestamp (unix seconds).
    pub search_reset:      u64,
    /// GraphQL API limit (if available).
    pub graphql_limit:     Option<usize>,
    /// Remaining GraphQL API requests (if available).
    pub graphql_remaining: Option<usize>,
    /// GraphQL API reset timestamp (if available).
    pub graphql_reset:     Option<u64>
}

/// Checks the current GitHub API rate limits.
///
/// # Errors
///
/// Returns an error when the rate limit cannot be fetched.
pub async fn check_rate_limit(token: Option<&str>) -> Result<RateLimitStatus, Error> {
    let octocrab = if let Some(token) = token {
        Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|error| Error::service(format!("failed to build GitHub client: {error}")))?
    } else {
        Octocrab::builder()
            .build()
            .map_err(|error| Error::service(format!("failed to build GitHub client: {error}")))?
    };

    let rate = octocrab
        .ratelimit()
        .get()
        .await
        .map_err(|error| Error::service(format!("failed to fetch rate limit: {error}")))?;

    Ok(RateLimitStatus {
        core_limit:        rate.resources.core.limit,
        core_remaining:    rate.resources.core.remaining,
        core_reset:        rate.resources.core.reset,
        search_limit:      rate.resources.search.limit,
        search_remaining:  rate.resources.search.remaining,
        search_reset:      rate.resources.search.reset,
        graphql_limit:     rate.resources.graphql.as_ref().map(|value| value.limit),
        graphql_remaining: rate.resources.graphql.as_ref().map(|value| value.remaining),
        graphql_reset:     rate.resources.graphql.as_ref().map(|value| value.reset)
    })
}

/// Result of preparing the metrics Docker image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrepareMetricsImageResult {
    /// Remote image that was pulled.
    pub remote_image: String,
    /// Local image tag created for the action.
    pub local_image:  String,
    /// Whether the remote image was pulled successfully.
    pub pulled:       bool,
    /// Whether the remote image was tagged locally.
    pub tagged:       bool
}

/// Prepares the lowlighter/metrics Docker image.
///
/// # Errors
///
/// Returns an error when the latest release cannot be fetched.
pub async fn prepare_metrics_image(
    token: Option<&str>,
    actor: Option<&str>
) -> Result<PrepareMetricsImageResult, Error> {
    let octocrab = if let Some(token) = token {
        Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|error| Error::service(format!("failed to build GitHub client: {error}")))?
    } else {
        Octocrab::builder()
            .build()
            .map_err(|error| Error::service(format!("failed to build GitHub client: {error}")))?
    };

    let release = octocrab
        .repos("lowlighter", "metrics")
        .releases()
        .get_latest()
        .await
        .map_err(|error| {
            Error::service(format!("failed to fetch latest metrics release: {error}"))
        })?;

    let tag = release.tag_name;
    let version = tag.trim_start_matches('v');
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return Err(Error::service(format!("unexpected metrics version: {tag}")));
    }

    let major = parts[0];
    let minor = parts[1];
    let patch = parts.get(2).copied().unwrap_or("0");

    let prebuilt_tag = format!("v{major}.{minor}");
    let remote_image = format!("ghcr.io/lowlighter/metrics:{prebuilt_tag}");
    let local_image = format!("metrics:forked-{major}.{minor}.{patch}");

    let _ = Command::new("docker").args(["rmi", &local_image]).output();

    if let Some(token) = token {
        let actor = actor.unwrap_or("github-actions[bot]");
        let mut command = Command::new("docker")
            .args(["login", "ghcr.io", "-u", actor, "--password-stdin"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|source| Error::service(format!("failed to spawn docker: {source}")))?;

        if let Some(mut stdin) = command.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(token.as_bytes());
        }
        let _ = command.wait();
    }

    let pulled = Command::new("docker")
        .args(["pull", &remote_image])
        .status()
        .is_ok_and(|status| status.success());

    if !pulled {
        return Ok(PrepareMetricsImageResult {
            remote_image,
            local_image,
            pulled: false,
            tagged: false
        });
    }

    let tagged = Command::new("docker")
        .args(["tag", &remote_image, &local_image])
        .status()
        .is_ok_and(|status| status.success());

    Ok(PrepareMetricsImageResult {
        remote_image,
        local_image,
        pulled,
        tagged
    })
}
