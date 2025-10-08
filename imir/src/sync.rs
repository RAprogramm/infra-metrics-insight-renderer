// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Synchronizes targets.yaml with discovered repositories.
///
/// Merges newly discovered repositories into the existing targets configuration
/// without duplicating entries or overwriting user customizations.
use std::{collections::HashSet, fs, path::Path};

use masterror::AppError;

use crate::{DiscoveredRepository, TargetConfig, TargetEntry, TargetKind};

/// Synchronizes discovered repositories with the targets configuration file.
///
/// # Arguments
///
/// * `config_path` - Path to the targets.yaml configuration file
/// * `discovered` - List of discovered repositories to add
///
/// # Errors
///
/// Returns [`AppError`] when file operations fail or YAML parsing errors occur.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
///
/// use imir::{DiscoveredRepository, sync_targets};
///
/// # async fn example() -> Result<(), masterror::AppError> {
/// let discovered = vec![DiscoveredRepository {
///     owner:      "user".to_string(),
///     repository: "repo".to_string(),
/// }];
/// sync_targets(Path::new("targets/targets.yaml",), &discovered,)?;
/// # Ok(())
/// # }
/// ```
pub fn sync_targets(
    config_path: &Path,
    discovered: &[DiscoveredRepository],
) -> Result<usize, AppError,>
{
    let yaml_content = fs::read_to_string(config_path,).map_err(|e| {
        AppError::service(format!("failed to read config at {}: {e}", config_path.display(),),)
    },)?;

    let mut config: TargetConfig = serde_yaml::from_str(&yaml_content,)
        .map_err(|e| AppError::validation(format!("failed to parse targets config: {e}"),),)?;

    let existing_repos: HashSet<(String, Option<String,>,),> =
        config.targets.iter().map(|t| (t.owner.clone(), t.repository.clone(),),).collect();

    let mut added_count = 0;

    for repo in discovered {
        let key = (repo.owner.clone(), Some(repo.repository.clone(),),);

        if !existing_repos.contains(&key,) {
            let new_entry = TargetEntry {
                owner:               repo.owner.clone(),
                repository:          Some(repo.repository.clone(),),
                target_type:         TargetKind::OpenSource,
                branch_name:         None,
                contributors_branch: None,
                target_path:         None,
                temp_artifact:       None,
                time_zone:           None,
                slug:                None,
                display_name:        None,
                include_private:     None,
                badge:               None,
            };

            config.targets.push(new_entry,);
            added_count += 1;
        }
    }

    if added_count > 0 {
        config.targets.sort_by(|a, b| {
            a.owner
                .cmp(&b.owner,)
                .then_with(|| a.repository.as_deref().cmp(&b.repository.as_deref(),),)
        },);

        let updated_yaml = serde_yaml::to_string(&config,)
            .map_err(|e| AppError::service(format!("failed to serialize updated config: {e}"),),)?;

        fs::write(config_path, updated_yaml,).map_err(|e| {
            AppError::service(format!("failed to write config to {}: {e}", config_path.display()),)
        },)?;
    }

    Ok(added_count,)
}

#[cfg(test)]
mod tests
{
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn sync_targets_adds_new_repositories()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let initial_yaml = r#"
targets:
  - owner: existing
    repository: repo
    type: open_source
"#;
        fs::write(&config_path, initial_yaml,).expect("failed to write config",);

        let discovered = vec![DiscoveredRepository {
            owner:      "newuser".to_string(),
            repository: "newrepo".to_string(),
        }];

        let added = sync_targets(&config_path, &discovered,).expect("sync failed",);
        assert_eq!(added, 1);

        let updated = fs::read_to_string(&config_path,).expect("failed to read updated config",);
        assert!(updated.contains("newuser"));
        assert!(updated.contains("newrepo"));
    }

    #[test]
    fn sync_targets_skips_duplicates()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let initial_yaml = r#"
targets:
  - owner: existing
    repository: repo
    type: open_source
"#;
        fs::write(&config_path, initial_yaml,).expect("failed to write config",);

        let discovered = vec![DiscoveredRepository {
            owner:      "existing".to_string(),
            repository: "repo".to_string(),
        }];

        let added = sync_targets(&config_path, &discovered,).expect("sync failed",);
        assert_eq!(added, 0);
    }
}
