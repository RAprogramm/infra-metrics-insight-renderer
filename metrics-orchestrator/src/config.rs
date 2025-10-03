//! Configuration document types describing metrics renderer targets.
//!
//! The types in this module mirror the structure of the YAML documents
//! consumed by the orchestrator CLI. They intentionally keep optional values
//! flexible to allow user-supplied overrides, and provide helper methods for
//! deriving normalized values that satisfy downstream invariants.

use serde::Deserialize;

use crate::slug::SlugStrategy;

/// Root configuration document describing all targets that should be rendered.
///
/// # Examples
///
/// ```
/// use metrics_orchestrator::TargetConfig;
///
/// let yaml = r#"
/// targets:
///   - owner: octocat
///     repo: hello-world
///     type: open_source
/// "#;
/// let config: TargetConfig = serde_yaml::from_str(yaml,).expect("valid configuration",);
/// assert_eq!(config.targets.len(), 1);
/// ```
#[derive(Debug, Deserialize)]
pub struct TargetConfig {
    /// Collection of metrics targets to render.
    #[serde(default)]
    pub targets: Vec<TargetEntry>,
}

/// Raw configuration entry describing a single metrics target before
/// normalization.
///
/// Instances are typically created by deserializing YAML documents. Helper
/// methods are provided to derive slugs and display names in a consistent
/// manner.
#[derive(Debug, Deserialize, Clone)]
pub struct TargetEntry {
    /// GitHub account that owns the repository or profile.
    #[serde(alias = "user")]
    pub owner: String,

    /// Optional repository name associated with the target.
    #[serde(default, alias = "repo")]
    pub repository: Option<String>,

    /// Logical target type that influences renderer presets.
    #[serde(rename = "type")]
    pub target_type: TargetKind,

    /// Optional slug override used for filenames and branch names.
    #[serde(default)]
    pub slug: Option<String>,

    /// Optional branch name override for commits with refreshed metrics.
    #[serde(default, alias = "branch", alias = "branch-name", alias = "branchName")]
    pub branch_name: Option<String>,

    /// Optional destination path override for the generated SVG artifact.
    #[serde(default)]
    pub target_path: Option<String>,

    /// Optional temporary artifact override produced by the renderer.
    #[serde(default)]
    pub temp_artifact: Option<String>,

    /// Optional timezone override for renderer inputs.
    #[serde(default)]
    pub time_zone: Option<String>,

    /// Optional display name override used in commit messages.
    #[serde(default)]
    pub display_name: Option<String>,
}

impl TargetEntry {
    /// Returns the slug that should be used for this target.
    ///
    /// Custom overrides are normalized through [`SlugStrategy`] while
    /// fallbacks are derived based on the target type.
    ///
    /// # Examples
    ///
    /// ```
    /// use metrics_orchestrator::{TargetEntry, TargetKind};
    ///
    /// let entry = TargetEntry {
    ///     owner: "octocat".to_owned(),
    ///     repository: Some("metrics".to_owned(),),
    ///     target_type: TargetKind::OpenSource,
    ///     slug: None,
    ///     branch_name: None,
    ///     target_path: None,
    ///     temp_artifact: None,
    ///     time_zone: None,
    ///     display_name: None,
    /// };
    /// assert_eq!(entry.resolved_slug().as_deref(), Some("metrics"));
    /// ```
    pub fn resolved_slug(&self) -> Option<String> {
        if let Some(custom) = self.slug.as_ref() {
            return SlugStrategy::builder(custom).build();
        }

        match self.target_type {
            TargetKind::Profile => {
                let derived = format!("{}-profile", self.owner);
                SlugStrategy::builder(&derived).build()
            }
            TargetKind::OpenSource | TargetKind::PrivateProject => self
                .repository
                .as_ref()
                .and_then(|name| SlugStrategy::builder(name).build()),
        }
    }

    /// Provides the display name used for commit messages and logging.
    ///
    /// Leading and trailing whitespace is trimmed. When no override is
    /// supplied, the value falls back to "profile" or the repository name.
    pub fn resolved_display_name(&self) -> Option<String> {
        if let Some(name) = self.display_name.as_ref() {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return None;
            }
            return Some(trimmed.to_owned());
        }

        match self.target_type {
            TargetKind::Profile => Some("profile".to_owned()),
            TargetKind::OpenSource | TargetKind::PrivateProject => {
                self.repository.as_ref().map(|repo| repo.trim().to_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TargetEntry, TargetKind};

    #[test]
    fn resolved_slug_prefers_custom_value() {
        let entry = TargetEntry {
            owner: "octocat".to_owned(),
            repository: Some("Hello-World".to_owned()),
            target_type: TargetKind::OpenSource,
            slug: Some("  Custom Slug  ".to_owned()),
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: None,
        };

        let slug = entry
            .resolved_slug()
            .expect("expected slug to be derived from override");
        assert_eq!(slug, "custom-slug");
    }

    #[test]
    fn resolved_slug_falls_back_to_profile_default() {
        let entry = TargetEntry {
            owner: "octocat".to_owned(),
            repository: None,
            target_type: TargetKind::Profile,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: None,
        };

        let slug = entry
            .resolved_slug()
            .expect("expected slug to be derived from owner");
        assert_eq!(slug, "octocat-profile");
    }

    #[test]
    fn resolved_slug_falls_back_to_repository_name() {
        let entry = TargetEntry {
            owner: "octocat".to_owned(),
            repository: Some("Example Repo".to_owned()),
            target_type: TargetKind::PrivateProject,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: None,
        };

        let slug = entry
            .resolved_slug()
            .expect("expected slug to be derived from repository");
        assert_eq!(slug, "example-repo");
    }

    #[test]
    fn resolved_slug_returns_none_when_unable_to_derive() {
        let entry = TargetEntry {
            owner: "octocat".to_owned(),
            repository: Some("***".to_owned()),
            target_type: TargetKind::OpenSource,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: None,
        };

        assert!(entry.resolved_slug().is_none());
    }

    #[test]
    fn resolved_display_name_prefers_override() {
        let entry = TargetEntry {
            owner: "octocat".to_owned(),
            repository: Some("repo".to_owned()),
            target_type: TargetKind::OpenSource,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: Some("  Friendly Name  ".to_owned()),
        };

        let display = entry
            .resolved_display_name()
            .expect("expected display name to be derived");
        assert_eq!(display, "Friendly Name");
    }

    #[test]
    fn resolved_display_name_uses_repository_name() {
        let entry = TargetEntry {
            owner: "octocat".to_owned(),
            repository: Some(" Repo With Spaces ".to_owned()),
            target_type: TargetKind::OpenSource,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: None,
        };

        let display = entry
            .resolved_display_name()
            .expect("expected repository name to be used");
        assert_eq!(display, "Repo With Spaces");
    }

    #[test]
    fn resolved_display_name_returns_none_when_override_blank() {
        let entry = TargetEntry {
            owner: "octocat".to_owned(),
            repository: None,
            target_type: TargetKind::Profile,
            slug: None,
            branch_name: None,
            target_path: None,
            temp_artifact: None,
            time_zone: None,
            display_name: Some("   ".to_owned()),
        };

        assert!(entry.resolved_display_name().is_none());
    }
}

/// Supported categories of metrics targets.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    /// Render a GitHub profile dashboard.
    Profile,
    /// Render an open-source repository dashboard.
    OpenSource,
    /// Render a private repository dashboard.
    PrivateProject,
}
