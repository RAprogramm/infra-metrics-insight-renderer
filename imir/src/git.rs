// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Git operations for committing and pushing metrics updates.
///
/// Provides utilities for branch management, commits, and force-with-lease pushes.
use std::process::Command;

use masterror::AppError;
use serde::{Deserialize, Serialize};

/// Result of git commit and push operation.
#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct GitPushResult
{
    /// Whether changes were pushed to remote.
    pub pushed:       bool,
    /// Default base branch for PR creation.
    pub default_base: String,
}

/// Commits and pushes changes to a branch with retry logic.
///
/// # Arguments
///
/// * `branch_name` - Target branch name
/// * `file_path` - Path to file to add and commit
/// * `commit_message` - Commit message
///
/// # Returns
///
/// [`GitPushResult`] containing push status and default base branch.
///
/// # Errors
///
/// Returns [`AppError`] when git operations fail after all retries.
///
/// # Example
///
/// ```no_run
/// use imir::git_commit_push;
///
/// # fn example() -> Result<(), masterror::AppError> {
/// let result = git_commit_push(
///     "ci/metrics-refresh-profile",
///     "metrics/profile.svg",
///     "chore(metrics): refresh profile",
/// )?;
/// if result.pushed {
///     println!("Pushed to branch, base: {}", result.default_base);
/// }
/// # Ok(())
/// # }
/// ```
pub fn git_commit_push(
    branch_name: &str,
    file_path: &str,
    commit_message: &str,
) -> Result<GitPushResult, AppError,>
{
    configure_git()?;

    let default_ref = get_default_ref()?;
    checkout_or_create_branch(branch_name, &default_ref,)?;

    let upstream_before = get_upstream_sha(branch_name,)?;

    add_file(file_path,)?;

    if !has_changes()? {
        return Ok(GitPushResult {
            pushed:       false,
            default_base: get_default_base(&default_ref,)?,
        },);
    }

    commit_changes(commit_message,)?;

    let pushed = push_with_retry(branch_name, &upstream_before,)?;

    Ok(GitPushResult {
        pushed,
        default_base: get_default_base(&default_ref,)?,
    },)
}

fn configure_git() -> Result<(), AppError,>
{
    run_git(&["config", "user.name", "github-actions[bot]"],)?;
    run_git(&["config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com"],)?;
    run_git(&["config", "pull.rebase", "true"],)?;
    Ok((),)
}

fn get_default_ref() -> Result<String, AppError,>
{
    let output = Command::new("git",)
        .args(["symbolic-ref", "--quiet", "--short", "HEAD"],)
        .output()
        .map_err(|e| AppError::service(format!("git symbolic-ref failed: {e}"),),)?;

    if output.status.success() {
        let ref_name = String::from_utf8_lossy(&output.stdout,).trim().to_string();
        if !ref_name.is_empty() {
            return Ok(ref_name,);
        }
    }

    Ok("main".to_string(),)
}

fn checkout_or_create_branch(branch_name: &str, default_ref: &str,) -> Result<(), AppError,>
{
    let remote_exists = Command::new("git",)
        .args(["ls-remote", "--exit-code", "--heads", "origin", branch_name],)
        .output()
        .map(|o| o.status.success(),)
        .unwrap_or(false,);

    if remote_exists {
        run_git(&[
            "fetch",
            "--no-tags",
            "--prune",
            "--depth=1",
            "origin",
            &format!("+refs/heads/{branch_name}:refs/remotes/origin/{branch_name}"),
        ],)?;
        run_git(&["checkout", "-B", branch_name, &format!("origin/{branch_name}"),],)?;
    } else {
        let fetch_result = Command::new("git",)
            .args([
                "fetch",
                "--no-tags",
                "--prune",
                "--depth=1",
                "origin",
                &format!("+refs/heads/{default_ref}:refs/remotes/origin/{default_ref}"),
            ],)
            .output();

        if fetch_result.is_ok() && fetch_result.unwrap().status.success() {
            run_git(&["checkout", "-B", branch_name, &format!("origin/{default_ref}"),],)?;
        } else {
            run_git(&["checkout", "-B", branch_name, default_ref],)?;
        }
    }

    Ok((),)
}

fn get_upstream_sha(branch_name: &str,) -> Result<Option<String,>, AppError,>
{
    let output = Command::new("git",)
        .args(["rev-parse", "--verify", &format!("origin/{branch_name}"),],)
        .output()
        .map_err(|e| AppError::service(format!("git rev-parse failed: {e}"),),)?;

    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout,).trim().to_string();
        if !sha.is_empty() {
            return Ok(Some(sha,),);
        }
    }

    Ok(None,)
}

fn add_file(file_path: &str,) -> Result<(), AppError,>
{
    run_git(&["add", file_path],)
}

fn has_changes() -> Result<bool, AppError,>
{
    let output = Command::new("git",)
        .args(["diff", "--cached", "--quiet"],)
        .output()
        .map_err(|e| AppError::service(format!("git diff failed: {e}"),),)?;

    Ok(!output.status.success(),)
}

fn commit_changes(message: &str,) -> Result<(), AppError,>
{
    run_git(&["commit", "-m", message],)
}

fn push_with_retry(
    branch_name: &str,
    upstream_before: &Option<String,>,
) -> Result<bool, AppError,>
{
    for attempt in 1..=3 {
        if try_push(branch_name,)? {
            return Ok(true,);
        }

        let _ = run_git(&[
            "fetch",
            "--no-tags",
            "--prune",
            "--depth=1",
            "origin",
            &format!("+refs/heads/{branch_name}:refs/remotes/origin/{branch_name}"),
        ]);

        let remote_after = get_upstream_sha(branch_name,)?;

        if upstream_before.is_some() && remote_after != *upstream_before {
            continue;
        }

        if upstream_before.is_none() && remote_after.is_some() {
            continue;
        }

        if try_force_push(branch_name, upstream_before,)? {
            return Ok(true,);
        }

        if attempt == 3 {
            return Err(AppError::service("unable to push after 3 attempts",),);
        }
    }

    Ok(false,)
}

fn try_push(branch_name: &str,) -> Result<bool, AppError,>
{
    let output = Command::new("git",)
        .args(["push", "origin", branch_name],)
        .output()
        .map_err(|e| AppError::service(format!("git push failed: {e}"),),)?;

    Ok(output.status.success(),)
}

fn try_force_push(
    branch_name: &str,
    upstream_before: &Option<String,>,
) -> Result<bool, AppError,>
{
    let force_arg = if let Some(sha,) = upstream_before {
        format!("--force-with-lease=refs/heads/{branch_name}:{sha}")
    } else {
        "--force-with-lease".to_string()
    };

    let output = Command::new("git",)
        .args(["push", &force_arg, "origin", branch_name],)
        .output()
        .map_err(|e| AppError::service(format!("git push force failed: {e}"),),)?;

    Ok(output.status.success(),)
}

fn get_default_base(default_ref: &str,) -> Result<String, AppError,>
{
    let output = Command::new("git",)
        .args(["symbolic-ref", "--quiet", "--short", "refs/remotes/origin/HEAD"],)
        .output()
        .map_err(|e| AppError::service(format!("git symbolic-ref failed: {e}"),),)?;

    if output.status.success() {
        let remote_head = String::from_utf8_lossy(&output.stdout,).trim().to_string();
        if let Some(base,) = remote_head.strip_prefix("origin/",)
            && !base.is_empty()
        {
            return Ok(base.to_string(),);
        }
    }

    Ok(default_ref.to_string(),)
}

fn run_git(args: &[&str],) -> Result<(), AppError,>
{
    let output = Command::new("git",)
        .args(args,)
        .output()
        .map_err(|e| AppError::service(format!("git command failed: {e}"),),)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr,);
        return Err(AppError::service(format!("git {} failed: {stderr}", args.join(" "),),),);
    }

    Ok((),)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn git_push_result_serialization()
    {
        let result = GitPushResult {
            pushed:       true,
            default_base: "main".to_string(),
        };

        let json = serde_json::to_string(&result,).expect("serialization failed",);
        assert!(json.contains("true",));
        assert!(json.contains("main",));
    }

    #[test]
    fn git_push_result_clone()
    {
        let result = GitPushResult {
            pushed:       false,
            default_base: "develop".to_string(),
        };

        let cloned = result.clone();
        assert_eq!(result.pushed, cloned.pushed);
        assert_eq!(result.default_base, cloned.default_base);
    }
}
