use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::config::{TargetConfig, TargetEntry, TargetKind};
use crate::error::{self, Error};

const DEFAULT_BRANCH_PREFIX: &str = "ci/metrics-refresh-";
const DEFAULT_OUTPUT_DIR: &str = "metrics";
const DEFAULT_TEMP_DIR: &str = ".metrics-tmp";
const DEFAULT_EXTENSION: &str = "svg";
const DEFAULT_TIME_ZONE: &str = "Asia/Ho_Chi_Minh";

/// Normalized representation of a metrics target used by automation workflows.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct RenderTarget {
    /// Unique slug derived from the configuration entry.
    pub slug: String,
    /// Account that owns the repository or profile.
    pub owner: String,
    /// Optional repository associated with the target.
    pub repository: Option<String>,
    /// Target category.
    pub kind: TargetKind,
    /// Branch name used for storing refreshed metrics commits.
    pub branch_name: String,
    /// Final destination path for the generated SVG artifact.
    pub target_path: String,
    /// Temporary artifact produced by the metrics renderer.
    pub temp_artifact: String,
    /// Time zone passed to the renderer.
    pub time_zone: String,
    /// Display name used in commit messages and logs.
    pub display_name: String,
}

/// Document containing all normalized targets.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TargetsDocument {
    /// Aggregated targets derived from the configuration.
    pub targets: Vec<RenderTarget>,
}

/// Loads targets from the provided YAML configuration file path.
pub fn load_targets(path: &Path) -> Result<TargetsDocument, Error> {
    let contents = fs::read_to_string(path).map_err(|source| error::io_error(path, source))?;
    parse_targets(&contents)
}

/// Parses targets from the provided YAML document string.
pub fn parse_targets(contents: &str) -> Result<TargetsDocument, Error> {
    let config: TargetConfig = serde_yaml::from_str(contents)?;
    if config.targets.is_empty() {
        return Err(Error::validation(
            "configuration must include at least one target",
        ));
    }

    normalize_targets(&config.targets)
}

fn normalize_targets(entries: &[TargetEntry]) -> Result<TargetsDocument, Error> {
    let mut normalized = Vec::with_capacity(entries.len());
    let mut seen_slugs = HashSet::with_capacity(entries.len());
    let mut seen_paths = HashSet::with_capacity(entries.len());
    let mut seen_temp = HashSet::with_capacity(entries.len());
    let mut seen_branches = HashSet::with_capacity(entries.len());

    for entry in entries {
        let target = normalize_entry(entry)?;

        if !seen_slugs.insert(target.slug.clone()) {
            return Err(Error::validation(format!(
                "duplicate slug '{}'",
                target.slug
            )));
        }
        if !seen_paths.insert(target.target_path.clone()) {
            return Err(Error::validation(format!(
                "duplicate target_path '{}'",
                target.target_path
            )));
        }
        if !seen_temp.insert(target.temp_artifact.clone()) {
            return Err(Error::validation(format!(
                "duplicate temp_artifact '{}'",
                target.temp_artifact
            )));
        }
        if !seen_branches.insert(target.branch_name.clone()) {
            return Err(Error::validation(format!(
                "duplicate branch_name '{}'",
                target.branch_name
            )));
        }

        normalized.push(target);
    }

    Ok(TargetsDocument {
        targets: normalized,
    })
}

fn normalize_entry(entry: &TargetEntry) -> Result<RenderTarget, Error> {
    let owner = normalize_identifier(&entry.owner, "owner")?;

    let repository = match entry.target_type {
        TargetKind::Profile => None,
        TargetKind::OpenSource | TargetKind::PrivateProject => {
            let repo_name = entry.repository.as_ref().ok_or_else(|| {
                Error::validation("repository is required for repository targets")
            })?;
            Some(normalize_identifier(repo_name, "repository")?)
        }
    };

    let slug = entry
        .resolved_slug()
        .ok_or_else(|| Error::validation("unable to derive slug for target"))?;

    let branch_name = match entry.branch_name.as_ref() {
        Some(custom) => normalize_path_like(custom, "branch_name")?,
        None => format!("{DEFAULT_BRANCH_PREFIX}{slug}"),
    };

    let target_path = match entry.target_path.as_ref() {
        Some(custom) => normalize_path_like(custom, "target_path")?,
        None => format!("{DEFAULT_OUTPUT_DIR}/{slug}.{DEFAULT_EXTENSION}"),
    };

    let temp_artifact = match entry.temp_artifact.as_ref() {
        Some(custom) => normalize_path_like(custom, "temp_artifact")?,
        None => format!("{DEFAULT_TEMP_DIR}/{slug}.{DEFAULT_EXTENSION}"),
    };

    let time_zone = entry
        .time_zone
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map_or_else(|| DEFAULT_TIME_ZONE.to_owned(), |value| value.to_owned());

    let display_name = entry
        .resolved_display_name()
        .ok_or_else(|| Error::validation("unable to derive display name for target"))?;

    Ok(RenderTarget {
        slug,
        owner,
        repository,
        kind: entry.target_type,
        branch_name,
        target_path,
        temp_artifact,
        time_zone,
        display_name,
    })
}

fn normalize_identifier(input: &str, field: &str) -> Result<String, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::validation(format!("{field} cannot be empty")));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(Error::validation(format!(
            "{field} cannot contain whitespace"
        )));
    }
    Ok(trimmed.to_owned())
}

fn normalize_path_like(input: &str, field: &str) -> Result<String, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::validation(format!(
            "{field} override cannot be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TargetKind;

    #[test]
    fn normalizes_repository_entry() {
        let entry = TargetEntry {
            owner: "RAprogramm".to_owned(),
            repository: Some("metrics".to_owned()),
            target_type: TargetKind::OpenSource,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: None,
        };

        let target = normalize_entry(&entry).expect("expected normalization success");
        assert_eq!(target.slug, "metrics");
        assert_eq!(target.branch_name, "ci/metrics-refresh-metrics");
        assert_eq!(target.target_path, "metrics/metrics.svg");
        assert_eq!(target.temp_artifact, ".metrics-tmp/metrics.svg");
        assert_eq!(target.display_name, "metrics");
    }

    #[test]
    fn rejects_missing_repository_for_repository_target() {
        let entry = TargetEntry {
            owner: "user".to_owned(),
            repository: None,
            target_type: TargetKind::OpenSource,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: None,
        };

        let result = normalize_entry(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn prevents_duplicate_slugs() {
        let entries = vec![
            TargetEntry {
                owner: "user".to_owned(),
                repository: Some("repo".to_owned()),
                target_type: TargetKind::OpenSource,
                slug: None,
                branch_name: None,
                target_path: None,
                temp_artifact: None,
                time_zone: None,
                display_name: None,
            },
            TargetEntry {
                owner: "user".to_owned(),
                repository: Some("repo".to_owned()),
                target_type: TargetKind::PrivateProject,
                slug: None,
                branch_name: None,
                target_path: None,
                temp_artifact: None,
                time_zone: None,
                display_name: None,
            },
        ];

        let result = normalize_targets(&entries);
        assert!(result.is_err());
    }
}
