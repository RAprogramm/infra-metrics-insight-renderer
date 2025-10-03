use serde::Deserialize;

use crate::slug::SlugStrategy;

/// Root configuration document describing all targets that should be rendered.
#[derive(Debug, Deserialize)]
pub struct TargetConfig {
    /// Collection of metrics targets to render.
    #[serde(default)]
    pub targets: Vec<TargetEntry>,
}

/// Raw configuration entry describing a single metrics target before normalization.
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
    #[serde(default)]
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
    /// Returns the slug that should be used for this target, normalizing
    /// optional user-provided overrides when present.
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
