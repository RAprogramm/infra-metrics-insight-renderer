// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Locates generated metrics artifacts in expected paths.
///
/// Searches multiple candidate locations where lowlighter/metrics may have
/// written the generated SVG artifact.
use std::path::{Path, PathBuf};

use masterror::AppError;
use serde::{Deserialize, Serialize};

/// Result of artifact location containing the found path.
#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct ArtifactLocation
{
    /// Absolute path to the located artifact.
    pub path: PathBuf,
}

/// Locates a metrics artifact by searching expected paths.
///
/// # Arguments
///
/// * `temp_artifact` - Expected filename or relative path
/// * `workspace` - GitHub workspace directory (usually GITHUB_WORKSPACE)
///
/// # Returns
///
/// [`ArtifactLocation`] containing the absolute path to the artifact.
///
/// # Errors
///
/// Returns [`AppError`] when artifact cannot be found in any candidate path.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use imir::locate_artifact;
///
/// # fn example() -> Result<(), masterror::AppError> {
/// let location = locate_artifact(
///     ".metrics-tmp/profile.svg",
///     "/github/workspace",
/// )?;
/// println!("Found artifact at: {}", location.path.display());
/// # Ok(())
/// # }
/// ```
pub fn locate_artifact(
    temp_artifact: &str,
    workspace: &str,
) -> Result<ArtifactLocation, AppError,>
{
    if temp_artifact.is_empty() {
        return Err(AppError::validation("temp_artifact cannot be empty",),);
    }

    let workspace_path = Path::new(workspace,);
    let temp_path = Path::new(temp_artifact,);
    let basename = temp_path
        .file_name()
        .ok_or_else(|| AppError::validation("temp_artifact has no filename",),)?;

    let candidates = vec![
        workspace_path.join(temp_artifact,),
        PathBuf::from("/metrics_renders",).join(temp_artifact,),
        PathBuf::from("/metrics_renders",).join(basename,),
    ];

    for candidate in &candidates {
        if candidate.exists() && candidate.is_file() {
            return Ok(ArtifactLocation {
                path: candidate.clone(),
            },);
        }
    }

    let mut error_msg = format!(
        "Unable to locate metrics artifact. Searched paths:\n{}",
        candidates
            .iter()
            .map(|p| format!("  - {}", p.display()),)
            .collect::<Vec<_>>()
            .join("\n"),
    );

    let metrics_renders = Path::new("/metrics_renders",);
    if metrics_renders.exists() && metrics_renders.is_dir() {
        error_msg.push_str("\n\nDiscovered files under /metrics_renders:\n",);
        if let Ok(entries,) = std::fs::read_dir(metrics_renders,) {
            for entry in entries.flatten() {
                if let Ok(path,) = entry.path().canonicalize() {
                    error_msg.push_str(&format!("  - {}\n", path.display()),);
                }
            }
        }
    }

    Err(AppError::service(error_msg,),)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn artifact_location_serialization()
    {
        let location = ArtifactLocation {
            path: PathBuf::from("/metrics_renders/profile.svg",),
        };

        let json = serde_json::to_string(&location,).expect("serialization failed",);
        assert!(json.contains("profile.svg",));
    }

    #[test]
    fn artifact_location_clone()
    {
        let location = ArtifactLocation {
            path: PathBuf::from("/test/path.svg",),
        };

        let cloned = location.clone();
        assert_eq!(location.path, cloned.path);
    }

    #[test]
    fn locate_artifact_rejects_empty_temp_artifact()
    {
        let result = locate_artifact("", "/workspace",);
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err(),);
        assert!(error_msg.contains("temp_artifact"),);
    }

    #[test]
    fn locate_artifact_rejects_invalid_filename()
    {
        let result = locate_artifact("/", "/workspace",);
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err(),);
        assert!(error_msg.contains("filename"),);
    }
}
