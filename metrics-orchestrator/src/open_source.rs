//! Helpers for resolving open-source repository inputs supplied to workflows.
//!
//! The functions in this module sanitize and validate user-supplied JSON
//! arrays before converting them into normalized repository name lists. The
//! sanitization ensures downstream automation cannot be tricked into
//! referencing empty or malformed entries.

use crate::error::Error;

/// Default repositories used when the workflow input is omitted.
const DEFAULT_REPOSITORIES: &[&str] = &["masterror", "telegram-webapp-sdk"];

/// Resolves the repository list for the open-source workflow input.
///
/// The input must be a JSON array of repository names. Leading and trailing
/// whitespace around individual entries is trimmed. When no input is provided
/// the default repositories are returned.
///
/// # Errors
///
/// Returns [`Error::Validation`](Error::Validation) when the input is not a
/// valid JSON array of strings, contains empty entries, or expands to an empty
/// list.
///
/// # Examples
///
/// ```
/// use metrics_orchestrator::resolve_open_source_repositories;
///
/// let repositories = resolve_open_source_repositories(Some("[\"repo\"]",),)?;
/// assert_eq!(repositories, vec!["repo".to_owned()]);
/// # Ok::<(), metrics_orchestrator::Error>(())
/// ```
pub fn resolve_open_source_repositories(raw_input: Option<&str>) -> Result<Vec<String>, Error> {
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

/// Parses and validates repository names supplied as a JSON array.
///
/// # Errors
///
/// Returns [`Error::Validation`](Error::Validation) when the JSON is invalid,
/// expands to an empty array, or contains blank entries.
fn parse_user_supplied_repositories(input: &str) -> Result<Vec<String>, Error> {
    let parsed: Vec<String> = serde_json::from_str(input)
        .map_err(|error| Error::validation(format!("invalid repositories JSON: {error}")))?;

    if parsed.is_empty() {
        return Err(Error::validation(
            "repositories input must be a non-empty JSON array of repository names",
        ));
    }

    let mut normalized = Vec::with_capacity(parsed.len());
    for repository in parsed {
        let trimmed = repository.trim();
        if trimmed.is_empty() {
            return Err(Error::validation(
                "repository names cannot be empty strings",
            ));
        }
        normalized.push(trimmed.to_owned());
    }

    Ok(normalized)
}

/// Returns the default repository list as owned strings.
fn default_repositories() -> Vec<String> {
    let mut defaults = Vec::with_capacity(DEFAULT_REPOSITORIES.len());
    for repository in DEFAULT_REPOSITORIES {
        defaults.push((*repository).to_owned());
    }
    defaults
}

#[cfg(test)]
mod tests {
    use super::resolve_open_source_repositories;

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
}
