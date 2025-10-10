// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Input normalization for render actions.
///
/// Provides utilities for validating and normalizing action inputs.
use masterror::AppError;
use serde::{Deserialize, Serialize};

/// Normalized profile render inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInputs {
    pub target_user: String,
    pub branch_name: String,
    pub target_path: String,
    pub temp_artifact: String,
    pub time_zone: String,
    pub display_name: String,
    pub include_private: String,
    pub repositories_affiliations: String,
    pub plugin_repositories_affiliations: String,
    pub plugin_activity_visibility: String,
    pub plugin_code_visibility: String,
    pub plugin_achievements_secrets: String
}

/// Normalized repository render inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInputs {
    pub target_owner:        String,
    pub target_repo:         String,
    pub target_path:         String,
    pub temp_artifact:       String,
    pub branch_name:         String,
    pub contributors_branch: String,
    pub time_zone:           String
}

/// Normalizes profile render inputs with defaults and validation.
///
/// # Arguments
///
/// * `target_user` - GitHub user or organization
/// * `branch_name` - Branch for commits (optional)
/// * `target_path` - Destination path (optional)
/// * `temp_artifact` - Temp artifact filename (optional)
/// * `time_zone` - Time zone string (optional)
/// * `display_name` - Display name for commits (optional)
/// * `include_private` - Include private repos (optional)
///
/// # Returns
///
/// [`ProfileInputs`] with normalized values.
///
/// # Errors
///
/// Returns [`AppError`] when target_user is empty or include_private is
/// invalid.
pub fn normalize_profile_inputs(
    target_user: &str,
    branch_name: Option<&str>,
    target_path: Option<&str>,
    temp_artifact: Option<&str>,
    time_zone: Option<&str>,
    display_name: Option<&str>,
    include_private: Option<&str>
) -> Result<ProfileInputs, AppError> {
    if target_user.is_empty() {
        return Err(AppError::validation("target_user must be provided"));
    }

    let branch = branch_name
        .filter(|s| !s.is_empty())
        .unwrap_or("ci/metrics-refresh-profile");

    let path = target_path
        .filter(|s| !s.is_empty())
        .unwrap_or("metrics/profile.svg");

    let artifact = temp_artifact
        .filter(|s| !s.is_empty())
        .unwrap_or(".metrics-tmp/profile.svg");

    let tz = time_zone
        .filter(|s| !s.is_empty())
        .unwrap_or("Asia/Ho_Chi_Minh");

    let name = display_name.filter(|s| !s.is_empty()).unwrap_or("profile");

    let private_normalized = include_private.unwrap_or("").to_lowercase();

    let (
        include_priv,
        repos_affil,
        plugin_repos_affil,
        plugin_activity_vis,
        plugin_code_vis,
        plugin_achievements_sec
    ) = match private_normalized.as_str() {
        "" | "false" | "0" | "no" => (
            "false",
            "owner, organization_member",
            "owner, organization_member",
            "public",
            "public",
            "no"
        ),
        "true" | "1" | "yes" => (
            "true",
            "owner, collaborator, organization_member",
            "owner, collaborator, organization_member",
            "all",
            "all",
            "yes"
        ),
        _ => {
            return Err(AppError::validation(
                "include_private must be a boolean value"
            ));
        }
    };

    Ok(ProfileInputs {
        target_user: target_user.to_string(),
        branch_name: branch.to_string(),
        target_path: path.to_string(),
        temp_artifact: artifact.to_string(),
        time_zone: tz.to_string(),
        display_name: name.to_string(),
        include_private: include_priv.to_string(),
        repositories_affiliations: repos_affil.to_string(),
        plugin_repositories_affiliations: plugin_repos_affil.to_string(),
        plugin_activity_visibility: plugin_activity_vis.to_string(),
        plugin_code_visibility: plugin_code_vis.to_string(),
        plugin_achievements_secrets: plugin_achievements_sec.to_string()
    })
}

/// Normalizes repository render inputs with defaults and validation.
///
/// # Arguments
///
/// * `target_repo` - Repository name
/// * `target_owner` - Repository owner (optional, uses GITHUB_REPOSITORY owner
///   if empty)
/// * `github_repo` - GITHUB_REPOSITORY env var (owner/repo format)
/// * `target_path` - Destination path (optional)
/// * `temp_artifact` - Temp artifact filename (optional)
/// * `branch_name` - Branch for commits (optional)
/// * `contributors_branch` - Branch for contributors plugin (optional)
/// * `time_zone` - Time zone string (optional)
///
/// # Returns
///
/// [`RepositoryInputs`] with normalized values.
///
/// # Errors
///
/// Returns [`AppError`] when target_repo is empty or contributors_branch is
/// invalid.
#[allow(clippy::too_many_arguments)]
pub fn normalize_repository_inputs(
    target_repo: &str,
    target_owner: Option<&str>,
    github_repo: &str,
    target_path: Option<&str>,
    temp_artifact: Option<&str>,
    branch_name: Option<&str>,
    contributors_branch: Option<&str>,
    time_zone: Option<&str>
) -> Result<RepositoryInputs, AppError> {
    if target_repo.is_empty() {
        return Err(AppError::validation("target_repo must be provided"));
    }

    let owner = if let Some(o) = target_owner.filter(|s| !s.is_empty()) {
        o.to_string()
    } else {
        github_repo
            .split('/')
            .next()
            .ok_or_else(|| AppError::validation("invalid GITHUB_REPOSITORY format"))?
            .to_string()
    };

    let path = if let Some(p) = target_path.filter(|s| !s.is_empty()) {
        p.to_string()
    } else {
        format!("metrics/{target_repo}.svg")
    };

    let artifact = if let Some(a) = temp_artifact.filter(|s| !s.is_empty()) {
        a.to_string()
    } else {
        format!(".metrics-tmp/{target_repo}.svg")
    };

    let branch = if let Some(b) = branch_name.filter(|s| !s.is_empty()) {
        b.to_string()
    } else {
        format!("ci/metrics-refresh-{target_repo}")
    };

    let contrib_branch = contributors_branch
        .filter(|s| !s.is_empty())
        .map(|s| s.trim())
        .unwrap_or("main");

    if contrib_branch.is_empty() {
        return Err(AppError::validation("contributors_branch cannot be empty"));
    }

    if contrib_branch.contains(char::is_whitespace) {
        return Err(AppError::validation(
            "contributors_branch cannot contain whitespace"
        ));
    }

    let tz = time_zone
        .filter(|s| !s.is_empty())
        .unwrap_or("Asia/Ho_Chi_Minh");

    Ok(RepositoryInputs {
        target_owner:        owner,
        target_repo:         target_repo.to_string(),
        target_path:         path,
        temp_artifact:       artifact,
        branch_name:         branch,
        contributors_branch: contrib_branch.to_string(),
        time_zone:           tz.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_profile_inputs_with_defaults() {
        let result =
            normalize_profile_inputs("octocat", None, None, None, None, None, None).unwrap();

        assert_eq!(result.target_user, "octocat");
        assert_eq!(result.branch_name, "ci/metrics-refresh-profile");
        assert_eq!(result.target_path, "metrics/profile.svg");
        assert_eq!(result.temp_artifact, ".metrics-tmp/profile.svg");
        assert_eq!(result.time_zone, "Asia/Ho_Chi_Minh");
        assert_eq!(result.display_name, "profile");
        assert_eq!(result.include_private, "false");
    }

    #[test]
    fn normalize_profile_inputs_with_custom_values() {
        let result = normalize_profile_inputs(
            "custom-user",
            Some("custom-branch"),
            Some("custom/path.svg"),
            Some("custom-tmp.svg"),
            Some("UTC"),
            Some("custom"),
            Some("true")
        )
        .unwrap();

        assert_eq!(result.target_user, "custom-user");
        assert_eq!(result.branch_name, "custom-branch");
        assert_eq!(result.include_private, "true");
        assert_eq!(
            result.repositories_affiliations,
            "owner, collaborator, organization_member"
        );
    }

    #[test]
    fn normalize_profile_inputs_rejects_empty_target_user() {
        let result = normalize_profile_inputs("", None, None, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn normalize_profile_inputs_rejects_invalid_include_private() {
        let result =
            normalize_profile_inputs("octocat", None, None, None, None, None, Some("invalid"));
        assert!(result.is_err());
    }

    #[test]
    fn normalize_repository_inputs_with_defaults() {
        let result = normalize_repository_inputs(
            "my-repo",
            None,
            "owner/repo",
            None,
            None,
            None,
            None,
            None
        )
        .unwrap();

        assert_eq!(result.target_owner, "owner");
        assert_eq!(result.target_repo, "my-repo");
        assert_eq!(result.target_path, "metrics/my-repo.svg");
        assert_eq!(result.branch_name, "ci/metrics-refresh-my-repo");
        assert_eq!(result.contributors_branch, "main");
    }

    #[test]
    fn normalize_repository_inputs_with_custom_values() {
        let result = normalize_repository_inputs(
            "test-repo",
            Some("custom-owner"),
            "ignored/repo",
            Some("custom/path.svg"),
            Some("custom-tmp.svg"),
            Some("custom-branch"),
            Some("develop"),
            Some("UTC")
        )
        .unwrap();

        assert_eq!(result.target_owner, "custom-owner");
        assert_eq!(result.contributors_branch, "develop");
    }

    #[test]
    fn normalize_repository_inputs_rejects_empty_target_repo() {
        let result =
            normalize_repository_inputs("", None, "owner/repo", None, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn normalize_repository_inputs_rejects_whitespace_in_contributors_branch() {
        let result = normalize_repository_inputs(
            "test-repo",
            None,
            "owner/repo",
            None,
            None,
            None,
            Some("branch with spaces"),
            None
        );
        assert!(result.is_err());
    }
}
