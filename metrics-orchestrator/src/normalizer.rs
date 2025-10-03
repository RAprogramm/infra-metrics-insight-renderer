use std::{collections::HashSet, fs, path::Path};

use serde::Serialize;

use crate::{
    config::{TargetConfig, TargetEntry, TargetKind},
    error::{self, Error},
};

const DEFAULT_BRANCH_PREFIX: &str = "ci/metrics-refresh-";
const DEFAULT_OUTPUT_DIR: &str = "metrics";
const DEFAULT_TEMP_DIR: &str = ".metrics-tmp";
const DEFAULT_EXTENSION: &str = "svg";
const DEFAULT_TIME_ZONE: &str = "Asia/Ho_Chi_Minh";

/// Normalized representation of a metrics target used by automation workflows.
#[derive(Debug, Serialize, Clone, PartialEq, Eq,)]
pub struct RenderTarget
{
    /// Unique slug derived from the configuration entry.
    pub slug:          String,
    /// Account that owns the repository or profile.
    pub owner:         String,
    /// Optional repository associated with the target.
    pub repository:    Option<String,>,
    /// Target category.
    pub kind:          TargetKind,
    /// Branch name used for storing refreshed metrics commits.
    pub branch_name:   String,
    /// Final destination path for the generated SVG artifact.
    pub target_path:   String,
    /// Temporary artifact produced by the metrics renderer.
    pub temp_artifact: String,
    /// Time zone passed to the renderer.
    pub time_zone:     String,
    /// Display name used in commit messages and logs.
    pub display_name:  String,
}

/// Document containing all normalized targets.
#[derive(Debug, Serialize, Clone, PartialEq, Eq,)]
pub struct TargetsDocument
{
    /// Aggregated targets derived from the configuration.
    pub targets: Vec<RenderTarget,>,
}

/// Loads targets from the provided YAML configuration file path.
pub fn load_targets(path: &Path,) -> Result<TargetsDocument, Error,>
{
    let contents = fs::read_to_string(path,).map_err(|source| error::io_error(path, source,),)?;
    parse_targets(&contents,)
}

/// Parses targets from the provided YAML document string.
pub fn parse_targets(contents: &str,) -> Result<TargetsDocument, Error,>
{
    let config: TargetConfig = serde_yaml::from_str(contents,)?;
    if config.targets.is_empty() {
        return Err(Error::validation("configuration must include at least one target",),);
    }

    normalize_targets(&config.targets,)
}

fn normalize_targets(entries: &[TargetEntry],) -> Result<TargetsDocument, Error,>
{
    let mut normalized = Vec::with_capacity(entries.len(),);
    let mut seen_slugs = HashSet::with_capacity(entries.len(),);
    let mut seen_paths = HashSet::with_capacity(entries.len(),);
    let mut seen_temp = HashSet::with_capacity(entries.len(),);
    let mut seen_branches = HashSet::with_capacity(entries.len(),);

    for entry in entries {
        let target = normalize_entry(entry,)?;

        if !seen_slugs.insert(target.slug.clone(),) {
            return Err(Error::validation(format!("duplicate slug '{}'", target.slug),),);
        }
        if !seen_paths.insert(target.target_path.clone(),) {
            return Err(Error::validation(format!(
                "duplicate target_path '{}'",
                target.target_path
            ),),);
        }
        if !seen_temp.insert(target.temp_artifact.clone(),) {
            return Err(Error::validation(format!(
                "duplicate temp_artifact '{}'",
                target.temp_artifact
            ),),);
        }
        if !seen_branches.insert(target.branch_name.clone(),) {
            return Err(Error::validation(format!(
                "duplicate branch_name '{}'",
                target.branch_name
            ),),);
        }

        normalized.push(target,);
    }

    Ok(TargetsDocument {
        targets: normalized,
    },)
}

fn normalize_entry(entry: &TargetEntry,) -> Result<RenderTarget, Error,>
{
    let owner = normalize_identifier(&entry.owner, "owner",)?;

    let repository = match entry.target_type {
        TargetKind::Profile => None,
        TargetKind::OpenSource | TargetKind::PrivateProject => {
            let repo_name = entry.repository.as_ref().ok_or_else(|| {
                Error::validation("repository is required for repository targets",)
            },)?;
            Some(normalize_identifier(repo_name, "repository",)?,)
        }
    };

    let slug = entry
        .resolved_slug()
        .ok_or_else(|| Error::validation("unable to derive slug for target",),)?;

    let branch_name = match entry.branch_name.as_ref() {
        Some(custom,) => normalize_path_like(custom, "branch_name",)?,
        None => format!("{DEFAULT_BRANCH_PREFIX}{slug}"),
    };

    let target_path = match entry.target_path.as_ref() {
        Some(custom,) => normalize_path_like(custom, "target_path",)?,
        None => format!("{DEFAULT_OUTPUT_DIR}/{slug}.{DEFAULT_EXTENSION}"),
    };

    let temp_artifact = match entry.temp_artifact.as_ref() {
        Some(custom,) => normalize_path_like(custom, "temp_artifact",)?,
        None => format!("{DEFAULT_TEMP_DIR}/{slug}.{DEFAULT_EXTENSION}"),
    };

    let time_zone = entry
        .time_zone
        .as_ref()
        .map(|value| value.trim(),)
        .filter(|value| !value.is_empty(),)
        .map_or_else(|| DEFAULT_TIME_ZONE.to_owned(), |value| value.to_owned(),);

    let display_name = entry
        .resolved_display_name()
        .ok_or_else(|| Error::validation("unable to derive display name for target",),)?;

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
    },)
}

fn normalize_identifier(input: &str, field: &str,) -> Result<String, Error,>
{
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::validation(format!("{field} cannot be empty"),),);
    }
    if trimmed.chars().any(char::is_whitespace,) {
        return Err(Error::validation(format!("{field} cannot contain whitespace"),),);
    }
    Ok(trimmed.to_owned(),)
}

fn normalize_path_like(input: &str, field: &str,) -> Result<String, Error,>
{
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::validation(format!("{field} override cannot be empty"),),);
    }
    Ok(trimmed.to_owned(),)
}

#[cfg(test)]
mod tests
{
    use std::io::Write;

    use super::{
        load_targets, normalize_entry, normalize_identifier, normalize_path_like,
        normalize_targets, parse_targets, Error,
    };
    use crate::config::{TargetEntry, TargetKind};

    fn repository_entry() -> TargetEntry
    {
        TargetEntry {
            owner:         "RAprogramm".to_owned(),
            repository:    Some("metrics".to_owned(),),
            target_type:   TargetKind::OpenSource,
            slug:          None,
            branch_name:   None,
            target_path:   None,
            temp_artifact: None,
            time_zone:     None,
            display_name:  None,
        }
    }

    #[test]
    fn normalizes_repository_entry()
    {
        let entry = repository_entry();

        let target = normalize_entry(&entry,).expect("expected normalization success",);
        assert_eq!(target.slug, "metrics");
        assert_eq!(target.branch_name, "ci/metrics-refresh-metrics");
        assert_eq!(target.target_path, "metrics/metrics.svg");
        assert_eq!(target.temp_artifact, ".metrics-tmp/metrics.svg");
        assert_eq!(target.display_name, "metrics");
    }

    #[test]
    fn normalizes_profile_entry_with_overrides()
    {
        let entry = TargetEntry {
            owner:         " Octocat ".to_owned(),
            repository:    None,
            target_type:   TargetKind::Profile,
            slug:          Some(" Custom.Profile ".to_owned(),),
            branch_name:   Some("  feature/metrics  ".to_owned(),),
            target_path:   Some("  dashboards/profile.svg  ".to_owned(),),
            temp_artifact: Some("  tmp/profile.svg  ".to_owned(),),
            time_zone:     Some("  UTC  ".to_owned(),),
            display_name:  Some("  Profile Name  ".to_owned(),),
        };

        let target = normalize_entry(&entry,).expect("expected overrides to be honored",);
        assert_eq!(target.slug, "custom-profile");
        assert_eq!(target.branch_name, "feature/metrics");
        assert_eq!(target.target_path, "dashboards/profile.svg");
        assert_eq!(target.temp_artifact, "tmp/profile.svg");
        assert_eq!(target.time_zone, "UTC");
        assert_eq!(target.display_name, "Profile Name");
    }

    #[test]
    fn rejects_missing_repository_for_repository_target()
    {
        let entry = TargetEntry {
            repository: None,
            ..repository_entry()
        };

        let result = normalize_entry(&entry,);
        assert!(result.is_err());
    }

    #[test]
    fn prevents_duplicate_slugs()
    {
        let entries = vec![repository_entry(), repository_entry()];

        let result = normalize_targets(&entries,);
        assert!(result.is_err());
    }

    #[test]
    fn prevents_duplicate_target_paths()
    {
        let mut a = repository_entry();
        a.target_path = Some("custom/path.svg".to_owned(),);
        let mut b = repository_entry();
        b.slug = Some("other".to_owned(),);
        b.target_path = Some("custom/path.svg".to_owned(),);

        let result = normalize_targets(&[a, b,],);
        assert!(result.is_err());
    }

    #[test]
    fn prevents_duplicate_temp_artifacts()
    {
        let mut a = repository_entry();
        a.temp_artifact = Some("tmp/output.svg".to_owned(),);
        let mut b = repository_entry();
        b.slug = Some("other".to_owned(),);
        b.temp_artifact = Some("tmp/output.svg".to_owned(),);

        let result = normalize_targets(&[a, b,],);
        assert!(result.is_err());
    }

    #[test]
    fn prevents_duplicate_branch_names()
    {
        let mut a = repository_entry();
        a.branch_name = Some("ci/branch".to_owned(),);
        let mut b = repository_entry();
        b.slug = Some("other".to_owned(),);
        b.branch_name = Some("ci/branch".to_owned(),);

        let result = normalize_targets(&[a, b,],);
        assert!(result.is_err());
    }

    #[test]
    fn normalize_identifier_rejects_whitespace()
    {
        let error = normalize_identifier("bad value", "field",).unwrap_err();
        match error {
            Error::Validation {
                message,
            } => {
                assert_eq!(message, "field cannot contain whitespace");
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn normalize_identifier_rejects_empty()
    {
        let error = normalize_identifier("   ", "field",).unwrap_err();
        match error {
            Error::Validation {
                message,
            } => {
                assert_eq!(message, "field cannot be empty");
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn normalize_path_like_trims_values()
    {
        let normalized = normalize_path_like("  path/value  ", "field",)
            .expect("expected normalization success",);
        assert_eq!(normalized, "path/value");
    }

    #[test]
    fn normalize_path_like_rejects_empty()
    {
        let error = normalize_path_like("   ", "field",).unwrap_err();
        match error {
            Error::Validation {
                message,
            } => {
                assert_eq!(message, "field override cannot be empty");
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn parse_targets_rejects_empty_configuration()
    {
        let result = parse_targets("targets: []",);
        assert!(result.is_err());
    }

    #[test]
    fn parse_targets_handles_valid_document()
    {
        let yaml = r#"
            targets:
              - owner: octocat
                repo: metrics
                type: open_source
        "#;

        let document = parse_targets(yaml,).expect("expected parse success",);
        assert_eq!(document.targets.len(), 1);
    }

    #[test]
    fn parse_targets_supports_branch_alias()
    {
        let yaml = r#"
            targets:
              - owner: octocat
                repository: metrics
                type: open_source
                branch:  feature/metrics 
        "#;

        let document = parse_targets(yaml,).expect("expected parse success",);
        assert_eq!(document.targets.len(), 1);
        assert_eq!(document.targets[0].branch_name, "feature/metrics");
    }

    #[test]
    fn parse_targets_propagates_decode_errors()
    {
        let result = parse_targets("targets: invalid",);
        assert!(matches!(result, Err(Error::Parse { .. })));
    }

    #[test]
    fn normalized_document_preserves_order()
    {
        let mut first = repository_entry();
        first.slug = Some("first".to_owned(),);
        let mut second = repository_entry();
        second.slug = Some("second".to_owned(),);

        let document =
            normalize_targets(&[first, second,],).expect("expected normalization success",);
        let slugs: Vec<_,> = document.targets.iter().map(|target| target.slug.as_str(),).collect();
        assert_eq!(slugs, ["first", "second"]);
    }

    #[test]
    fn render_target_equality_covers_all_fields()
    {
        let base = normalize_entry(&repository_entry(),).expect("expected success",);
        let mut clone = base.clone();
        assert_eq!(base, clone);
        clone.branch_name.push_str("-extra",);
        assert_ne!(base, clone);
    }

    #[test]
    fn load_targets_reads_configuration_from_disk()
    {
        let mut file = tempfile::NamedTempFile::new().expect("expected temp file",);
        write!(file, "targets:\n  - owner: octocat\n    repo: metrics\n    type: open_source\n")
            .expect("expected write to succeed",);

        let document = load_targets(file.path(),).expect("expected load to succeed",);
        assert_eq!(document.targets.len(), 1);
        assert_eq!(document.targets[0].owner, "octocat");
    }

    #[test]
    fn load_targets_reports_io_errors()
    {
        let path = std::path::Path::new("/nonexistent/config.yaml",);
        let error = load_targets(path,).expect_err("expected io error",);
        assert!(matches!(error, Error::Io { .. }));
    }
}
