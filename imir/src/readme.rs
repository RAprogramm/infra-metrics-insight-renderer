// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Updates README.md with automatically generated badge tables.
///
/// Scans targets.yaml and generates HTML tables for each category:
/// - Open-source repositories (🟩)
/// - Private projects (🟦)
/// - Profile badges (🟪)
use std::{fs, path::Path};

use masterror::AppError;
use tracing::{debug, info};

use crate::{RenderTarget, TargetKind, TargetsDocument};

const OPEN_SOURCE_START_MARKER: &str =
    "<h4 align=\"center\" id=\"open-source-badges\">🟩 Open-source badges</h4>";
const PRIVATE_START_MARKER: &str =
    "<h4 align=\"center\" id=\"private-project-badges\">🟦 Private project badges</h4>";
const PROFILE_START_MARKER: &str =
    "<h4 align=\"center\" id=\"profile-badges\">🟪 Profile badges</h4>";
const SECTION_END_MARKER: &str =
    "<p align=\"right\"><em><a href=\"#top\">Back to top</a></em></p>";

/// Updates README.md badge tables based on targets configuration.
///
/// # Arguments
///
/// * `readme_path` - Path to README.md file
/// * `document` - Parsed targets configuration
///
/// # Errors
///
/// Returns [`AppError`] when file operations fail or markers are not found.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
///
/// use imir::{load_targets, update_readme};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let document = load_targets(Path::new("targets/targets.yaml"))?;
/// update_readme(Path::new("README.md"), &document)?;
/// # Ok(())
/// # }
/// ```
pub fn update_readme(readme_path: &Path, document: &TargetsDocument) -> Result<(), AppError> {
    info!("Reading README from {}", readme_path.display());
    let content = fs::read_to_string(readme_path).map_err(|e| {
        AppError::service(format!(
            "failed to read README at {}: {e}",
            readme_path.display()
        ))
    })?;

    debug!("Grouping targets by kind");
    let open_source: Vec<&RenderTarget> = document
        .targets
        .iter()
        .filter(|t| t.kind == TargetKind::OpenSource)
        .collect();
    let private: Vec<&RenderTarget> = document
        .targets
        .iter()
        .filter(|t| t.kind == TargetKind::PrivateProject)
        .collect();
    let profiles: Vec<&RenderTarget> = document
        .targets
        .iter()
        .filter(|t| t.kind == TargetKind::Profile)
        .collect();

    info!(
        "Found {} open-source, {} private, {} profile targets",
        open_source.len(),
        private.len(),
        profiles.len()
    );

    let mut updated = content.clone();

    updated = replace_section(
        &updated,
        OPEN_SOURCE_START_MARKER,
        &generate_repository_table(&open_source)
    )?;

    updated = replace_section(
        &updated,
        PRIVATE_START_MARKER,
        &generate_private_section(&private)
    )?;

    updated = replace_section(
        &updated,
        PROFILE_START_MARKER,
        &generate_profile_table(&profiles)
    )?;

    if updated != content {
        info!("Writing updated README to {}", readme_path.display());
        fs::write(readme_path, updated).map_err(|e| {
            AppError::service(format!(
                "failed to write README to {}: {e}",
                readme_path.display()
            ))
        })?;
        info!("README updated successfully");
    } else {
        info!("No changes to README");
    }

    Ok(())
}

fn replace_section(
    content: &str,
    start_marker: &str,
    new_content: &str
) -> Result<String, AppError> {
    let start_idx = content
        .find(start_marker)
        .ok_or_else(|| AppError::validation(format!("start marker not found: {start_marker}")))?;

    let search_from = start_idx + start_marker.len();
    let end_idx = content[search_from..]
        .find(SECTION_END_MARKER)
        .ok_or_else(|| AppError::validation("section end marker not found".to_string()))?
        + search_from;

    let mut result = String::with_capacity(content.len());
    result.push_str(&content[..start_idx + start_marker.len()]);
    result.push_str("\n\n");
    result.push_str(new_content);
    result.push_str("\n\n");
    result.push_str(&content[end_idx..]);

    Ok(result)
}

fn generate_repository_table(targets: &[&RenderTarget]) -> String {
    if targets.is_empty() {
        return "<p>\n  No open-source repositories registered yet.\n</p>".to_string();
    }

    let mut table = String::from(
        "<table>\n  <thead>\n    <tr><th>Repository</th><th>Badge</th></tr>\n  </thead>\n  <tbody>"
    );

    for target in targets {
        let repo_name = target.repository.as_ref().map_or("", |r| r.as_str());
        let full_name = format!("{}/{}", target.owner, repo_name);
        let metrics_url = format!(
            "https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/{}.svg",
            target.slug
        );

        table.push_str(&format!(
            "\n    <tr>\n      <td><code>{}</code></td>\n      <td><img alt=\"{} metrics\" src=\"{}\" /></td>\n    </tr>",
            escape_html(&full_name),
            escape_html(repo_name),
            escape_html(&metrics_url)
        ));
    }

    table.push_str("\n  </tbody>\n</table>");
    table
}

fn generate_private_section(targets: &[&RenderTarget]) -> String {
    if targets.is_empty() {
        return "<p>\n  Private dashboards follow the same embedding rules. Publish badges from this section once private projects are registered.\n</p>".to_string();
    }

    let mut table = String::from(
        "<table>\n  <thead>\n    <tr><th>Repository</th><th>Badge</th></tr>\n  </thead>\n  <tbody>"
    );

    for target in targets {
        let repo_name = target.repository.as_ref().map_or("", |r| r.as_str());
        let full_name = format!("{}/{}", target.owner, repo_name);
        let metrics_url = format!(
            "https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/{}.svg",
            target.slug
        );

        table.push_str(&format!(
            "\n    <tr>\n      <td><code>{}</code></td>\n      <td><img alt=\"{} metrics\" src=\"{}\" /></td>\n    </tr>",
            escape_html(&full_name),
            escape_html(repo_name),
            escape_html(&metrics_url)
        ));
    }

    table.push_str("\n  </tbody>\n</table>");
    table
}

fn generate_profile_table(targets: &[&RenderTarget]) -> String {
    if targets.is_empty() {
        return "<p>\n  No profile badges registered yet.\n</p>".to_string();
    }

    let mut table = String::from(
        "<table>\n  <thead>\n    <tr><th>Account</th><th>Badge</th></tr>\n  </thead>\n  <tbody>"
    );

    for target in targets {
        let metrics_url = format!(
            "https://raw.githubusercontent.com/RAprogramm/infra-metrics-insight-renderer/main/metrics/{}.svg",
            target.slug
        );

        table.push_str(&format!(
            "\n    <tr>\n      <td><code>{}</code></td>\n      <td><img alt=\"{} profile metrics\" src=\"{}\" /></td>\n    </tr>",
            escape_html(&target.owner),
            escape_html(&target.owner),
            escape_html(&metrics_url)
        ));
    }

    table.push_str("\n  </tbody>\n</table>");
    table
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::{BadgeStyle, BadgeWidgetAlignment},
        normalizer::{BadgeDescriptor, BadgeWidgetDescriptor}
    };

    fn sample_target(
        owner: &str,
        repo: Option<&str>,
        kind: TargetKind,
        slug: &str
    ) -> RenderTarget {
        RenderTarget {
            slug: slug.to_owned(),
            owner: owner.to_owned(),
            repository: repo.map(String::from),
            kind,
            branch_name: "main".to_owned(),
            target_path: format!("metrics/{}.svg", slug),
            temp_artifact: format!(".metrics-tmp/{}.svg", slug),
            time_zone: "UTC".to_owned(),
            display_name: slug.to_owned(),
            contributors_branch: "main".to_owned(),
            include_private: false,
            badge: BadgeDescriptor {
                style:  BadgeStyle::Classic,
                widget: BadgeWidgetDescriptor {
                    columns:       2,
                    alignment:     BadgeWidgetAlignment::Center,
                    border_radius: 6
                }
            }
        }
    }

    #[test]
    fn generate_repository_table_creates_valid_html() {
        let target1 = sample_target("user1", Some("repo1"), TargetKind::OpenSource, "repo1");
        let target2 = sample_target("user2", Some("repo2"), TargetKind::OpenSource, "repo2");
        let targets = vec![&target1, &target2];

        let table = generate_repository_table(&targets);
        assert!(table.contains("<table>"));
        assert!(table.contains("user1/repo1"));
        assert!(table.contains("user2/repo2"));
        assert!(table.contains("</table>"));
    }

    #[test]
    fn generate_repository_table_handles_empty_list() {
        let targets: Vec<&RenderTarget> = vec![];
        let result = generate_repository_table(&targets);
        assert!(result.contains("No open-source repositories"));
    }

    #[test]
    fn generate_profile_table_creates_valid_html() {
        let target = sample_target("user1", None, TargetKind::Profile, "profile");
        let targets = vec![&target];

        let table = generate_profile_table(&targets);
        assert!(table.contains("<table>"));
        assert!(table.contains("user1"));
        assert!(table.contains("profile metrics"));
    }

    #[test]
    fn escape_html_handles_special_characters() {
        let input = "<script>alert('test')</script>";
        let escaped = escape_html(input);
        assert_eq!(
            escaped,
            "&lt;script&gt;alert(&#x27;test&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn update_readme_replaces_sections() {
        let temp = tempdir().expect("failed to create tempdir");
        let readme_path = temp.path().join("README.md");

        let initial_content = format!(
            r#"# Test README

{}

Old content here

{}

## Next section

{}

Old private content

{}

## Another section

{}

Old profile content

{}
"#,
            OPEN_SOURCE_START_MARKER,
            SECTION_END_MARKER,
            PRIVATE_START_MARKER,
            SECTION_END_MARKER,
            PROFILE_START_MARKER,
            SECTION_END_MARKER
        );

        fs::write(&readme_path, initial_content).expect("failed to write README");

        let document = TargetsDocument {
            targets: vec![sample_target(
                "testuser",
                Some("testrepo"),
                TargetKind::OpenSource,
                "testrepo"
            )]
        };

        update_readme(&readme_path, &document).expect("update failed");

        let updated = fs::read_to_string(&readme_path).expect("failed to read updated README");
        assert!(updated.contains("testuser/testrepo"));
        assert!(!updated.contains("Old content here"));
    }
}
