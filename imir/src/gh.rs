// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// GitHub CLI operations for PR creation.
///
/// Provides utilities for creating PRs with label handling.
use std::process::Command;

use masterror::AppError;
use serde::{Deserialize, Serialize};

/// Result of PR creation operation.
#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct PrCreateResult
{
    /// Whether a new PR was created.
    pub created:    bool,
    /// PR number (new or existing).
    pub pr_number:  Option<u64,>,
    /// PR URL (if created).
    pub pr_url:     Option<String,>,
    /// Message describing the result.
    pub message:    String,
}

/// Creates a PR idempotently with label handling.
///
/// # Arguments
///
/// * `repo` - Repository in owner/repo format
/// * `head` - Head branch name
/// * `base` - Base branch name
/// * `title` - PR title
/// * `body` - PR body
/// * `labels` - Labels to add
/// * `gh_token` - GitHub token for authentication
///
/// # Returns
///
/// [`PrCreateResult`] containing PR creation status and details.
///
/// # Errors
///
/// Returns [`AppError`] when gh commands fail.
///
/// # Example
///
/// ```no_run
/// use imir::gh_pr_create;
///
/// # fn example() -> Result<(), masterror::AppError> {
/// let result = gh_pr_create(
///     "owner/repo",
///     "feature-branch",
///     "main",
///     "chore(metrics): refresh",
///     "Auto-generated metrics update",
///     &["ci", "metrics"],
///     "ghp_token",
/// )?;
/// if result.created {
///     println!("Created PR: {:?}", result.pr_url);
/// }
/// # Ok(())
/// # }
/// ```
pub fn gh_pr_create(
    repo: &str,
    head: &str,
    base: &str,
    title: &str,
    body: &str,
    labels: &[&str],
    gh_token: &str,
) -> Result<PrCreateResult, AppError,>
{
    let existing_pr = check_existing_pr(repo, head, gh_token,)?;

    if let Some(pr_number,) = existing_pr {
        return Ok(PrCreateResult {
            created:   false,
            pr_number: Some(pr_number,),
            pr_url:    None,
            message:   format!("PR #{pr_number} already open for {repo}:{head} -> {base}"),
        },);
    }

    ensure_labels(repo, labels, gh_token,)?;

    let pr_url = create_pr(repo, head, base, title, body, labels, gh_token,)?;

    Ok(PrCreateResult {
        created:   true,
        pr_number: None,
        pr_url:    Some(pr_url.clone(),),
        message:   format!("Created PR: {pr_url}"),
    },)
}

fn check_existing_pr(repo: &str, head: &str, gh_token: &str,) -> Result<Option<u64,>, AppError,>
{
    let output = Command::new("gh",)
        .env("GH_TOKEN", gh_token,)
        .args([
            "pr",
            "list",
            "-R",
            repo,
            "--head",
            head,
            "--state",
            "open",
            "--json",
            "number",
            "--jq",
            ".[0].number",
        ],)
        .output()
        .map_err(|e| AppError::service(format!("gh pr list failed: {e}"),),)?;

    if !output.status.success() {
        return Ok(None,);
    }

    let stdout = String::from_utf8_lossy(&output.stdout,).trim().to_string();
    if stdout.is_empty() || stdout == "null" {
        return Ok(None,);
    }

    let pr_number = stdout
        .parse::<u64,>()
        .map_err(|e| AppError::validation(format!("invalid PR number: {e}"),),)?;

    Ok(Some(pr_number,),)
}

fn ensure_labels(repo: &str, labels: &[&str], gh_token: &str,) -> Result<(), AppError,>
{
    for label in labels {
        let view_output = Command::new("gh",)
            .env("GH_TOKEN", gh_token,)
            .args(["label", "view", label, "-R", repo],)
            .output()
            .map_err(|e| AppError::service(format!("gh label view failed: {e}"),),)?;

        if !view_output.status.success() {
            let _ = Command::new("gh",)
                .env("GH_TOKEN", gh_token,)
                .args([
                    "label",
                    "create",
                    label,
                    "-R",
                    repo,
                    "--description",
                    "Infrastructure automation",
                ],)
                .output();
        }
    }

    Ok((),)
}

fn create_pr(
    repo: &str,
    head: &str,
    base: &str,
    title: &str,
    body: &str,
    labels: &[&str],
    gh_token: &str,
) -> Result<String, AppError,>
{
    let mut args = vec![
        "pr",
        "create",
        "-R",
        repo,
        "--head",
        head,
        "--base",
        base,
        "--title",
        title,
        "--body",
        body,
    ];

    let label_args: Vec<String,> = labels
        .iter()
        .flat_map(|label| vec!["--label".to_string(), (*label).to_string(),],)
        .collect();

    let label_arg_refs: Vec<&str,> = label_args.iter().map(|s| s.as_str(),).collect();
    args.extend(label_arg_refs,);

    let output = Command::new("gh",)
        .env("GH_TOKEN", gh_token,)
        .args(&args,)
        .output()
        .map_err(|e| AppError::service(format!("gh pr create failed: {e}"),),)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr,);
        return Err(AppError::service(format!("gh pr create failed: {stderr}"),),);
    }

    let pr_url = String::from_utf8_lossy(&output.stdout,).trim().to_string();

    Ok(pr_url,)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn pr_create_result_serialization()
    {
        let result = PrCreateResult {
            created:   true,
            pr_number: None,
            pr_url:    Some("https://github.com/owner/repo/pull/123".to_string(),),
            message:   "Created PR".to_string(),
        };

        let json = serde_json::to_string(&result,).expect("serialization failed",);
        assert!(json.contains("true",));
        assert!(json.contains("pull/123",));
    }

    #[test]
    fn pr_create_result_clone()
    {
        let result = PrCreateResult {
            created:   false,
            pr_number: Some(42,),
            pr_url:    None,
            message:   "PR exists".to_string(),
        };

        let cloned = result.clone();
        assert_eq!(result.created, cloned.created);
        assert_eq!(result.pr_number, cloned.pr_number);
    }
}
