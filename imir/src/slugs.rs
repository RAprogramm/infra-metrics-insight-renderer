// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Detects impacted badge slugs from git changes.
///
/// Analyzes git diff between base and head refs to identify which badge slugs
/// need regeneration based on changes to README.md or targets.yaml.
use std::process::Command;

use masterror::AppError;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Result of slug detection containing list of impacted slugs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlugDetectionResult {
    /// List of slugs that need regeneration.
    pub slugs:   Vec<String>,
    /// Whether any slugs were detected.
    pub has_any: bool
}

/// Detects impacted slugs based on git diff.
///
/// # Arguments
///
/// * `base_ref` - Base git reference (commit, branch, tag)
/// * `head_ref` - Head git reference to compare against
/// * `files` - Files to check for changes (e.g., README.md, targets.yaml)
/// * `all_slugs` - All available slugs from targets
///
/// # Returns
///
/// [`SlugDetectionResult`] containing list of impacted slugs.
///
/// # Errors
///
/// Returns [`AppError`] when git commands fail or references are invalid.
///
/// # Example
///
/// ```no_run
/// use imir::detect_impacted_slugs;
///
/// # fn example() -> Result<(), masterror::AppError> {
/// let all_slugs = vec![
///     "profile".to_string(),
///     "masterror".to_string(),
///     "telegram-webapp-sdk".to_string(),
/// ];
/// let result = detect_impacted_slugs(
///     "main",
///     "HEAD",
///     &["README.md", "targets/targets.yaml"],
///     &all_slugs
/// )?;
/// println!("Impacted slugs: {:?}", result.slugs);
/// # Ok(())
/// # }
/// ```
pub fn detect_impacted_slugs(
    base_ref: &str,
    head_ref: &str,
    files: &[&str],
    all_slugs: &[String]
) -> Result<SlugDetectionResult, AppError> {
    if base_ref.is_empty() {
        return Ok(SlugDetectionResult {
            slugs:   all_slugs.to_vec(),
            has_any: !all_slugs.is_empty()
        });
    }

    let base_exists = Command::new("git")
        .args(["rev-parse", "--verify", base_ref])
        .output()
        .is_ok_and(|output| output.status.success());

    if !base_exists {
        let fetch_result = Command::new("git")
            .args([
                "fetch",
                "--no-tags",
                "--prune",
                "--depth=1",
                "origin",
                &format!("+{base_ref}:{base_ref}")
            ])
            .output();

        let fetch_failed = match fetch_result {
            Ok(output) => !output.status.success(),
            Err(_) => true
        };

        if fetch_failed {
            return Ok(SlugDetectionResult {
                slugs:   all_slugs.to_vec(),
                has_any: !all_slugs.is_empty()
            });
        }
    }

    let diff_output = if base_ref.is_empty() {
        Command::new("git")
            .args(["show", head_ref, "--"])
            .args(files)
            .output()
            .map_err(|e| AppError::service(format!("git show failed: {e}")))?
    } else {
        Command::new("git")
            .args(["diff", "--unified=0", base_ref, head_ref, "--"])
            .args(files)
            .output()
            .map_err(|e| AppError::service(format!("git diff failed: {e}")))?
    };

    if !diff_output.status.success() {
        return Ok(SlugDetectionResult {
            slugs:   Vec::new(),
            has_any: false
        });
    }

    let diff_text = String::from_utf8_lossy(&diff_output.stdout);
    let pattern = Regex::new(r"metrics/([A-Za-z0-9_.-]+)\.svg")
        .map_err(|e| AppError::validation(format!("invalid regex: {e}")))?;

    let mut slugs = Vec::new();
    for cap in pattern.captures_iter(&diff_text) {
        if let Some(slug) = cap.get(1) {
            let slug_str = slug.as_str().to_string();
            if all_slugs.contains(&slug_str) && !slugs.contains(&slug_str) {
                slugs.push(slug_str);
            }
        }
    }

    slugs.sort();

    Ok(SlugDetectionResult {
        has_any: !slugs.is_empty(),
        slugs
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_detection_result_serialization() {
        let result = SlugDetectionResult {
            slugs:   vec!["profile".to_string(), "masterror".to_string()],
            has_any: true
        };

        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(json.contains("profile",));
        assert!(json.contains("masterror",));
        assert!(json.contains("true",));
    }

    #[test]
    fn slug_detection_result_empty() {
        let result = SlugDetectionResult {
            slugs:   Vec::new(),
            has_any: false
        };

        assert!(!result.has_any);
        assert!(result.slugs.is_empty());
    }

    #[test]
    fn slug_detection_result_clone() {
        let result = SlugDetectionResult {
            slugs:   vec!["test".to_string()],
            has_any: true
        };

        let cloned = result.clone();
        assert_eq!(result.slugs, cloned.slugs);
        assert_eq!(result.has_any, cloned.has_any);
    }

    #[test]
    fn empty_base_ref_returns_all_slugs() {
        let all_slugs = vec!["profile".to_string(), "masterror".to_string()];
        let result = detect_impacted_slugs("", "HEAD", &["README.md"], &all_slugs)
            .expect("empty base ref should short-circuit successfully");
        assert!(result.has_any);
        assert_eq!(result.slugs, all_slugs);
    }

    #[test]
    fn empty_base_ref_with_no_slugs_reports_none() {
        let result = detect_impacted_slugs("", "HEAD", &["README.md"], &[])
            .expect("short-circuit must succeed even with empty slug set");
        assert!(!result.has_any);
        assert!(result.slugs.is_empty());
    }

    fn init_repo_with_two_commits() -> tempfile::TempDir {
        use std::process::Command;

        let dir = tempfile::tempdir().expect("tempdir");
        for args in [
            ["init", "--quiet", "--initial-branch=main"].as_slice(),
            ["config", "user.name", "Test"].as_slice(),
            ["config", "user.email", "test@example.com"].as_slice(),
            ["config", "commit.gpgsign", "false"].as_slice()
        ] {
            Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .status()
                .expect("git init/config");
        }
        std::fs::create_dir_all(dir.path().join("metrics")).expect("mkdir metrics");
        std::fs::write(dir.path().join("README.md"), "initial\n").expect("write readme");
        for args in [
            ["add", "."].as_slice(),
            ["commit", "--quiet", "-m", "init"].as_slice()
        ] {
            Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .status()
                .expect("git add/commit init");
        }
        std::fs::write(dir.path().join("metrics/profile.svg"), "<svg/>\n").expect("write svg");
        std::fs::write(
            dir.path().join("README.md"),
            "updated link metrics/profile.svg\n"
        )
        .expect("update readme");
        for args in [
            ["add", "."].as_slice(),
            ["commit", "--quiet", "-m", "add profile"].as_slice()
        ] {
            Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .status()
                .expect("git add/commit add profile");
        }
        dir
    }

    #[test]
    #[serial_test::serial]
    fn detects_slug_referenced_between_two_commits() {
        let repo = init_repo_with_two_commits();
        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(repo.path()).expect("cd repo");

        let all_slugs = vec!["profile".to_string(), "masterror".to_string()];
        let result = detect_impacted_slugs("HEAD~1", "HEAD", &["README.md"], &all_slugs);

        std::env::set_current_dir(&prev_cwd).expect("restore cwd");
        let result = result.expect("detection should succeed");
        assert!(result.has_any);
        assert_eq!(result.slugs, vec!["profile".to_string()]);
    }

    #[test]
    #[serial_test::serial]
    fn missing_base_ref_with_unreachable_remote_falls_back_to_all_slugs() {
        let repo = init_repo_with_two_commits();
        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(repo.path()).expect("cd repo");

        let all_slugs = vec!["profile".to_string()];
        let result =
            detect_impacted_slugs("nonexistent-ref-zzzz", "HEAD", &["README.md"], &all_slugs);

        std::env::set_current_dir(&prev_cwd).expect("restore cwd");
        let result = result.expect("missing base must fall back to all slugs");
        assert!(result.has_any);
        assert_eq!(result.slugs, all_slugs);
    }
}
