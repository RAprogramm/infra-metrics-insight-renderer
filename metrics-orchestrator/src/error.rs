#![allow(non_shorthand_field_patterns)]
// The derive emitted by `masterror::Error` expands pattern matches that trip
// `non_shorthand_field_patterns`, so we disable the lint for this module.

use std::path::{Path, PathBuf};

/// Unified error type returned by the configuration loader and CLI.
#[derive(Debug, masterror::Error,)]
pub enum Error
{
    /// Wraps I/O errors that occur while reading configuration files.
    #[error("failed to read configuration from {path:?}: {source}")]
    Io
    {
        /// Location of the configuration file.
        path:   PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Wraps YAML decoding errors.
    #[error("failed to parse configuration: {source}")]
    Parse
    {
        /// Source decoding error from serde_yaml.
        source: serde_yaml::Error,
    },
    /// Returned when the configuration violates invariants.
    #[error("invalid configuration: {message}")]
    Validation
    {
        /// Human readable message describing the validation problem.
        message: String,
    },
    /// Wraps serialization errors when writing normalized output.
    #[error("failed to serialize targets: {source}")]
    Serialize
    {
        /// Underlying serialization error.
        source: serde_json::Error,
    },
}

impl Error
{
    /// Constructs a validation error from the provided displayable value.
    pub fn validation<M,>(message: M,) -> Self
    where
        M: Into<String,>,
    {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Formats the error for diagnostics without the variant name to ease CLI
    /// reporting.
    pub fn to_display_string(&self,) -> String
    {
        format!("{self}")
    }
}

impl From<serde_yaml::Error,> for Error
{
    fn from(source: serde_yaml::Error,) -> Self
    {
        Self::Parse {
            source,
        }
    }
}

impl From<serde_json::Error,> for Error
{
    fn from(source: serde_json::Error,) -> Self
    {
        Self::Serialize {
            source,
        }
    }
}

/// Helper function to create an [`Error::Io`] variant.
pub fn io_error(path: &Path, source: std::io::Error,) -> Error
{
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests
{
    use super::Error;

    #[test]
    fn validation_constructor_populates_message()
    {
        let error = Error::validation("something went wrong",);
        match error {
            Error::Validation {
                ref message,
            } => {
                assert_eq!(message, "something went wrong");
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn to_display_string_matches_display()
    {
        let error = Error::validation("display me",);
        assert_eq!(error.to_string(), error.to_display_string());
    }

    #[test]
    fn io_error_helper_wraps_path_and_source()
    {
        let path = std::path::Path::new("/tmp/example.yaml",);
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing",);
        let error = super::io_error(path, io_error,);

        match error {
            Error::Io {
                path: ref stored_path,
                ref source,
            } => {
                assert_eq!(stored_path, path);
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("expected io error, got {other:?}"),
        }
    }

    #[test]
    fn serde_yaml_conversion_maps_to_parse_variant()
    {
        let error = serde_yaml::from_str::<usize,>("not-a-number",).unwrap_err();
        let mapped: Error = error.into();
        assert!(matches!(mapped, Error::Parse { .. }));
    }

    #[test]
    fn serde_json_conversion_maps_to_serialize_variant()
    {
        let invalid = serde_json::from_str::<serde_json::Value,>("not-json",).unwrap_err();
        let mapped: Error = invalid.into();
        assert!(matches!(mapped, Error::Serialize { .. }));
    }
}
