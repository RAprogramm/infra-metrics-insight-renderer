//! Helpers for resolving open-source repository inputs supplied to workflows.
//!
//! The functions in this module sanitize and validate user-supplied JSON
//! arrays before converting them into normalized repository descriptors. Each
//! descriptor captures the repository name and the contributors branch so the
//! renderer can display accurate contributor insights while remaining resilient
//! to malformed input.

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Default repositories used when the workflow input is omitted.
const DEFAULT_REPOSITORIES: &[&str] = &["masterror", "telegram-webapp-sdk"];
const DEFAULT_CONTRIBUTORS_BRANCH: &str = "main";

/// Normalized descriptor for an open-source repository entry.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct OpenSourceRepository {
    /// Repository name resolved from workflow input.
    pub repository: String,
    /// Branch analyzed by the contributors plugin.
    pub contributors_branch: String,
}

/// Resolves repository descriptors for the open-source workflow input.
///
/// The input accepts a JSON array containing either bare repository names or
/// objects with `repository` and optional `contributors_branch` fields. Leading
/// and trailing whitespace around individual entries is trimmed. When no input
/// is provided the default repositories are returned.
///
/// # Errors
///
/// Returns [`Error::Validation`](Error::Validation) when the input is not a
/// valid JSON array, contains empty entries, or expands to an empty list.
///
/// # Examples
///
/// ```
/// use imir::{resolve_open_source_targets, OpenSourceRepository};
///
/// let targets = resolve_open_source_targets(Some("[{\"repository\":\"repo\"}]",),)?;
/// assert_eq!(targets, vec![OpenSourceRepository {
///     repository: "repo".to_owned(),
///     contributors_branch: "main".to_owned(),
/// }]);
/// # Ok::<(), imir::Error>(())
/// ```
pub fn resolve_open_source_targets(
    raw_input: Option<&str>,
) -> Result<Vec<OpenSourceRepository>, Error> {
    match raw_input.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }) {
        Some(value) => parse_user_supplied_repositories(value),
        None => Ok(default_repositories()),
    }
}

/// Resolves repository names without contributor metadata for compatibility.
///
/// This helper preserves the previous behaviour for callers that only require
/// repository names.
pub fn resolve_open_source_repositories(raw_input: Option<&str>) -> Result<Vec<String>, Error> {
    let targets = resolve_open_source_targets(raw_input)?;
    Ok(targets
        .into_iter()
        .map(|target| target.repository)
        .collect())
}

/// Parses and validates repository descriptors supplied as a JSON array.
///
/// # Errors
///
/// Returns [`Error::Validation`](Error::Validation) when the JSON is invalid,
/// expands to an empty array, or contains blank entries.
fn parse_user_supplied_repositories(input: &str) -> Result<Vec<OpenSourceRepository>, Error> {
    let parsed: Vec<RepositoryInput> = serde_json::from_str(input)
        .map_err(|error| Error::validation(format!("invalid repositories JSON: {error}")))?;

    if parsed.is_empty() {
        return Err(Error::validation(
            "repositories input must be a non-empty JSON array of repository names",
        ));
    }

    let mut normalized = Vec::with_capacity(parsed.len());
    for repository in parsed {
        let descriptor = match repository {
            RepositoryInput::Name(name) => OpenSourceRepository {
                repository: normalize_repository(&name)?,
                contributors_branch: DEFAULT_CONTRIBUTORS_BRANCH.to_owned(),
            },
            RepositoryInput::Descriptor(descriptor) => {
                let repository = normalize_repository(&descriptor.repository)?;
                let contributors_branch = descriptor
                    .contributors_branch
                    .as_deref()
                    .map(normalize_contributors_branch)
                    .transpose()?
                    .unwrap_or_else(|| DEFAULT_CONTRIBUTORS_BRANCH.to_owned());

                OpenSourceRepository {
                    repository,
                    contributors_branch,
                }
            }
        };

        normalized.push(descriptor);
    }

    Ok(normalized)
}

/// Returns the default repository descriptors when no input is supplied.
fn default_repositories() -> Vec<OpenSourceRepository> {
    let mut defaults = Vec::with_capacity(DEFAULT_REPOSITORIES.len());
    for repository in DEFAULT_REPOSITORIES {
        defaults.push(OpenSourceRepository {
            repository: (*repository).to_owned(),
            contributors_branch: DEFAULT_CONTRIBUTORS_BRANCH.to_owned(),
        });
    }
    defaults
}

fn normalize_repository(input: &str) -> Result<String, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::validation(
            "repository names cannot be empty strings",
        ));
    }

    Ok(trimmed.to_owned())
}

fn normalize_contributors_branch(input: &str) -> Result<String, Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Error::validation("contributors_branch cannot be empty"));
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(Error::validation(
            "contributors_branch cannot contain whitespace",
        ));
    }

    Ok(trimmed.to_owned())
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RepositoryInput {
    Name(String),
    Descriptor(RepositoryDescriptor),
}

#[derive(Debug, Deserialize)]
struct RepositoryDescriptor {
    repository: String,
    #[serde(default)]
    contributors_branch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_CONTRIBUTORS_BRANCH, OpenSourceRepository, resolve_open_source_repositories,
        resolve_open_source_targets,
    };

    #[test]
    fn falls_back_to_defaults_when_input_missing() {
        let repositories = resolve_open_source_repositories(None).expect("expected defaults");
        assert_eq!(
            repositories,
            vec!["masterror".to_owned(), "telegram-webapp-sdk".to_owned()]
        );
    }

    #[test]
    fn trims_and_normalizes_entries() {
        let repositories = resolve_open_source_repositories(Some("[\" repo \", \"another\"]"))
            .expect("expected normalization");
        assert_eq!(repositories, vec!["repo".to_owned(), "another".to_owned()]);
    }

    #[test]
    fn rejects_empty_array() {
        let error = resolve_open_source_repositories(Some("[]")).unwrap_err();
        match error {
            crate::Error::Validation { message } => {
                assert_eq!(
                    message,
                    "repositories input must be a non-empty JSON array of repository names"
                );
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_json() {
        let error = resolve_open_source_repositories(Some("not-json")).unwrap_err();
        match error {
            crate::Error::Validation { message } => {
                assert!(message.starts_with("invalid repositories JSON:"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_empty_entries() {
        let error = resolve_open_source_repositories(Some("[\"\", \"repo\"]")).unwrap_err();
        match error {
            crate::Error::Validation { message } => {
                assert_eq!(message, "repository names cannot be empty strings");
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn treats_whitespace_input_as_missing() {
        let repositories = resolve_open_source_repositories(Some("   "))
            .expect("expected defaults when input whitespace");
        assert_eq!(
            repositories,
            vec!["masterror".to_owned(), "telegram-webapp-sdk".to_owned()]
        );
    }

    #[test]
    fn resolves_descriptors_with_default_branch() {
        let targets = resolve_open_source_targets(Some("[\"repo\"]"))
            .expect("expected descriptor resolution");

        assert_eq!(
            targets,
            vec![OpenSourceRepository {
                repository: "repo".to_owned(),
                contributors_branch: DEFAULT_CONTRIBUTORS_BRANCH.to_owned(),
            }]
        );
    }

    #[test]
    fn resolves_branch_override_with_trimming() {
        let targets = resolve_open_source_targets(Some(
            "[{\"repository\":\"repo\",\"contributors_branch\":\" feature/main \"}]",
        ))
        .expect("expected branch override");

        assert_eq!(
            targets,
            vec![OpenSourceRepository {
                repository: "repo".to_owned(),
                contributors_branch: "feature/main".to_owned(),
            }]
        );
    }

    #[test]
    fn rejects_branch_with_internal_whitespace() {
        let error = resolve_open_source_targets(Some(
            "[{\"repository\":\"repo\",\"contributors_branch\":\"feature main\"}]",
        ))
        .expect_err("expected branch validation error");

        match error {
            crate::Error::Validation { message } => {
                assert_eq!(message, "contributors_branch cannot contain whitespace");
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }
}
