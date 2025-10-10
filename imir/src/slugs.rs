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
#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct SlugDetectionResult
{
    /// List of slugs that need regeneration.
    pub slugs:   Vec<String,>,
    /// Whether any slugs were detected.
    pub has_any: bool,
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
///     &["README.md", "targets/targets.yaml",],
///     &all_slugs,
/// )?;
/// println!("Impacted slugs: {:?}", result.slugs);
/// # Ok(())
/// # }
/// ```
pub fn detect_impacted_slugs(
    base_ref: &str,
    head_ref: &str,
    files: &[&str],
    all_slugs: &[String],
) -> Result<SlugDetectionResult, AppError,>
{
    if base_ref.is_empty() {
        return Ok(SlugDetectionResult {
            slugs:   all_slugs.to_vec(),
            has_any: !all_slugs.is_empty(),
        },);
    }

    let base_exists = Command::new("git",)
        .args(["rev-parse", "--verify", base_ref,],)
        .output()
        .map(|output| output.status.success(),)
        .unwrap_or(false,);

    if !base_exists {
        let fetch_result = Command::new("git",)
            .args([
                "fetch",
                "--no-tags",
                "--prune",
                "--depth=1",
                "origin",
                &format!("+{}:{}", base_ref, base_ref),
            ],)
            .output();

        if fetch_result.is_err() || !fetch_result.unwrap().status.success() {
            return Ok(SlugDetectionResult {
                slugs:   all_slugs.to_vec(),
                has_any: !all_slugs.is_empty(),
            },);
        }
    }

    let diff_output = if !base_ref.is_empty() {
        Command::new("git",)
            .args(["diff", "--unified=0", base_ref, head_ref, "--",],)
            .args(files,)
            .output()
            .map_err(|e| AppError::service(format!("git diff failed: {e}"),),)?
    } else {
        Command::new("git",)
            .args(["show", head_ref, "--",],)
            .args(files,)
            .output()
            .map_err(|e| AppError::service(format!("git show failed: {e}"),),)?
    };

    if !diff_output.status.success() {
        return Ok(SlugDetectionResult {
            slugs: Vec::new(), has_any: false,
        },);
    }

    let diff_text = String::from_utf8_lossy(&diff_output.stdout,);
    let pattern = Regex::new(r"metrics/([A-Za-z0-9_.-]+)\.svg",)
        .map_err(|e| AppError::validation(format!("invalid regex: {e}"),),)?;

    let mut slugs = Vec::new();
    for cap in pattern.captures_iter(&diff_text,) {
        if let Some(slug,) = cap.get(1,) {
            let slug_str = slug.as_str().to_string();
            if !slugs.contains(&slug_str,) {
                slugs.push(slug_str,);
            }
        }
    }

    slugs.sort();

    Ok(SlugDetectionResult {
        has_any: !slugs.is_empty(),
        slugs,
    },)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn slug_detection_result_serialization()
    {
        let result = SlugDetectionResult {
            slugs:   vec!["profile".to_string(), "masterror".to_string()],
            has_any: true,
        };

        let json = serde_json::to_string(&result,).expect("serialization failed",);
        assert!(json.contains("profile",));
        assert!(json.contains("masterror",));
        assert!(json.contains("true",));
    }

    #[test]
    fn slug_detection_result_empty()
    {
        let result = SlugDetectionResult {
            slugs: Vec::new(), has_any: false,
        };

        assert!(!result.has_any);
        assert!(result.slugs.is_empty());
    }

    #[test]
    fn slug_detection_result_clone()
    {
        let result = SlugDetectionResult {
            slugs: vec!["test".to_string()], has_any: true,
        };

        let cloned = result.clone();
        assert_eq!(result.slugs, cloned.slugs);
        assert_eq!(result.has_any, cloned.has_any);
    }
}
