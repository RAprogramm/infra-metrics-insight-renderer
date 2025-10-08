// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Synchronizes targets.yaml with discovered repositories.
///
/// Merges newly discovered repositories into the existing targets configuration
/// without duplicating entries or overwriting user customizations.
use std::{collections::HashSet, fs, path::Path};

use indicatif::{ProgressBar, ProgressStyle};
use masterror::AppError;
use tracing::{debug, info};

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
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.yellow} [{elapsed_precise}] {msg}",)
            .expect("valid template",),
    );

    pb.set_message(format!("Reading config from {}...", config_path.display()),);
    debug!("Reading config from {}", config_path.display());
    let yaml_content = fs::read_to_string(config_path,).map_err(|e| {
        AppError::service(format!("failed to read config at {}: {e}", config_path.display(),),)
    },)?;

    pb.set_message("Parsing YAML configuration...",);
    debug!("Parsing YAML configuration");
    let mut config: TargetConfig = serde_yaml::from_str(&yaml_content,)
        .map_err(|e| AppError::validation(format!("failed to parse targets config: {e}"),),)?;

    pb.set_message(format!("Building index of {} existing targets...", config.targets.len()),);
    debug!("Building index of {} existing targets", config.targets.len());
    let existing_repos: HashSet<(String, Option<String,>,),> =
        config.targets.iter().map(|t| (t.owner.clone(), t.repository.clone(),),).collect();

    let mut added_count = 0;

    pb.set_message(format!("Processing {} discovered repositories...", discovered.len()),);
    info!("Processing {} discovered repositories", discovered.len());
    for repo in discovered {
        let key = (repo.owner.clone(), Some(repo.repository.clone(),),);

        if !existing_repos.contains(&key,) {
            debug!("Adding new repository: {}", repo);
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
            pb.set_message(format!("Added {} new repositories...", added_count),);
        } else {
            debug!("Skipping existing repository: {}", repo);
        }
    }

    if added_count > 0 {
        pb.set_message(format!(
            "Sorting {} total targets alphabetically...",
            config.targets.len()
        ),);
        info!("Sorting {} total targets alphabetically", config.targets.len());
        config.targets.sort_by(|a, b| {
            a.owner
                .cmp(&b.owner,)
                .then_with(|| a.repository.as_deref().cmp(&b.repository.as_deref(),),)
        },);

        pb.set_message("Serializing updated configuration...",);
        debug!("Serializing updated configuration");
        let updated_yaml = serde_yaml::to_string(&config,)
            .map_err(|e| AppError::service(format!("failed to serialize updated config: {e}"),),)?;

        pb.set_message(format!("Writing updated config to {}...", config_path.display()),);
        info!("Writing updated config to {}", config_path.display());
        fs::write(config_path, updated_yaml,).map_err(|e| {
            AppError::service(format!("failed to write config to {}: {e}", config_path.display()),)
        },)?;

        pb.finish_with_message(format!("Sync complete: {} new repositories added", added_count),);
    } else {
        pb.finish_with_message("Sync complete: no new repositories to add",);
        debug!("No new repositories to add");
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
        let initial_yaml = r"
targets:
  - owner: existing
    repository: repo
    type: open_source
";
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
        let initial_yaml = r"
targets:
  - owner: existing
    repository: repo
    type: open_source
";
        fs::write(&config_path, initial_yaml,).expect("failed to write config",);

        let discovered = vec![DiscoveredRepository {
            owner:      "existing".to_string(),
            repository: "repo".to_string(),
        }];

        let added = sync_targets(&config_path, &discovered,).expect("sync failed",);
        assert_eq!(added, 0);
    }

    #[test]
    fn sync_targets_adds_multiple_repositories()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let initial_yaml = r"
targets:
  - owner: existing
    repository: repo
    type: open_source
";
        fs::write(&config_path, initial_yaml,).expect("failed to write config",);

        let discovered = vec![
            DiscoveredRepository {
                owner:      "user1".to_string(),
                repository: "repo1".to_string(),
            },
            DiscoveredRepository {
                owner:      "user2".to_string(),
                repository: "repo2".to_string(),
            },
            DiscoveredRepository {
                owner:      "user1".to_string(),
                repository: "repo3".to_string(),
            },
        ];

        let added = sync_targets(&config_path, &discovered,).expect("sync failed",);
        assert_eq!(added, 3);

        let updated = fs::read_to_string(&config_path,).expect("failed to read updated config",);
        assert!(updated.contains("user1"));
        assert!(updated.contains("repo1"));
        assert!(updated.contains("user2"));
        assert!(updated.contains("repo2"));
        assert!(updated.contains("repo3"));
    }

    #[test]
    fn sync_targets_preserves_existing_customizations()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let initial_yaml = r"
targets:
  - owner: existing
    repository: repo
    type: open_source
    slug: custom-slug
    display_name: Custom Name
";
        fs::write(&config_path, initial_yaml,).expect("failed to write config",);

        let discovered = vec![DiscoveredRepository {
            owner:      "newuser".to_string(),
            repository: "newrepo".to_string(),
        }];

        sync_targets(&config_path, &discovered,).expect("sync failed",);

        let updated = fs::read_to_string(&config_path,).expect("failed to read updated config",);
        assert!(updated.contains("custom-slug"));
        assert!(updated.contains("Custom Name"));
    }

    #[test]
    fn sync_targets_sorts_alphabetically()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let initial_yaml = r"
targets:
  - owner: zebra
    repository: repo
    type: open_source
";
        fs::write(&config_path, initial_yaml,).expect("failed to write config",);

        let discovered = vec![DiscoveredRepository {
            owner:      "alpha".to_string(),
            repository: "repo".to_string(),
        }];

        sync_targets(&config_path, &discovered,).expect("sync failed",);

        let updated = fs::read_to_string(&config_path,).expect("failed to read updated config",);
        let alpha_pos = updated.find("alpha",).expect("alpha not found",);
        let zebra_pos = updated.find("zebra",).expect("zebra not found",);
        assert!(alpha_pos < zebra_pos, "entries should be sorted alphabetically",);
    }

    #[test]
    fn sync_targets_returns_error_for_invalid_yaml()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        fs::write(&config_path, "invalid: [yaml: structure",).expect("failed to write config",);

        let discovered = vec![DiscoveredRepository {
            owner:      "user".to_string(),
            repository: "repo".to_string(),
        }];

        let result = sync_targets(&config_path, &discovered,);
        assert!(result.is_err(), "should fail on invalid YAML",);
    }

    #[test]
    fn sync_targets_returns_error_for_missing_file()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("nonexistent.yaml",);

        let discovered = vec![DiscoveredRepository {
            owner:      "user".to_string(),
            repository: "repo".to_string(),
        }];

        let result = sync_targets(&config_path, &discovered,);
        assert!(result.is_err(), "should fail when file doesn't exist",);
    }

    #[test]
    fn sync_targets_handles_empty_discovered_list()
    {
        let temp = tempdir().expect("failed to create tempdir",);
        let config_path = temp.path().join("targets.yaml",);
        let initial_yaml = r"
targets:
  - owner: existing
    repository: repo
    type: open_source
";
        fs::write(&config_path, initial_yaml,).expect("failed to write config",);

        let discovered = vec![];

        let added = sync_targets(&config_path, &discovered,).expect("sync failed",);
        assert_eq!(added, 0);
    }
}
