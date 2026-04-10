#![allow(non_shorthand_field_patterns)]
#![doc = "Error handling primitives shared across the orchestrator crate."]
// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
//
// SPDX-License-Identifier: MIT

//! The derive emitted by [`masterror::Error`] expands pattern matches that
//! trigger the `non_shorthand_field_patterns` lint. The lint is disabled for
//! the module to keep the generated implementations warning-free while still
//! exposing a thoroughly documented error surface for library consumers.

use std::path::{Path, PathBuf};

/// Unified error type returned by the configuration loader and CLI.
///
/// Each variant captures sufficient context for diagnostics while avoiding
/// accidental exposure of sensitive data. Instances are typically constructed
/// through the [`io_error`] helper or by converting from serde error types via
/// the provided `From` implementations.
#[derive(Debug, masterror::Error)]
pub enum Error {
    /// Wraps I/O errors that occur while reading configuration files.
    #[error("failed to read configuration from {path:?}: {source}")]
    Io {
        /// Location of the configuration file.
        path:   PathBuf,
        /// Underlying I/O error.
        source: std::io::Error
    },
    /// Wraps YAML decoding errors.
    #[error("failed to parse configuration: {source}")]
    Parse {
        /// Source decoding error from serde_yaml.
        source: serde_yaml::Error
    },
    /// Returned when the configuration violates invariants.
    #[error("invalid configuration: {message}")]
    Validation {
        /// Human readable message describing the validation problem.
        message: String
    },
    /// Wraps serialization errors when writing normalized output.
    #[error("failed to serialize targets: {source}")]
    Serialize {
        /// Underlying serialization error.
        source: serde_json::Error
    },
    /// Wraps I/O errors that occur while writing badge artifacts.
    #[error("failed to write badge artifact at {path:?}: {source}")]
    BadgeIo {
        /// Location of the artifact being produced.
        path:   PathBuf,
        /// Underlying I/O error reported by the operating system.
        source: std::io::Error
    },
    /// Service errors when interacting with external APIs.
    #[error("service error: {message}")]
    Service {
        /// Human readable message describing the service error.
        message: String
    },
    /// Wraps I/O errors that occur while processing SVG files.
    #[error("failed to process SVG at {path:?}: {source}")]
    SvgIo {
        /// Location of the SVG file being processed.
        path:   PathBuf,
        /// Underlying I/O error reported by the operating system.
        source: std::io::Error
    },
    /// Wraps parsing errors when analyzing SVG structure.
    #[error("failed to parse SVG: {message}")]
    SvgParse {
        /// Human readable message describing the parse failure.
        message: String
    }
}

impl Error {
    /// Constructs a validation error from the provided displayable value.
    ///
    /// # Parameters
    ///
    /// * `message` - Human-readable description of the validation failure.
    pub fn validation<M>(message: M) -> Self
    where
        M: Into<String>
    {
        Self::Validation {
            message: message.into()
        }
    }

    /// Constructs a service error from the provided displayable value.
    ///
    /// # Parameters
    ///
    /// * `message` - Human-readable description of the service error.
    pub fn service<M>(message: M) -> Self
    where
        M: Into<String>
    {
        Self::Service {
            message: message.into()
        }
    }

    /// Formats the error for diagnostics without the variant name.
    ///
    /// This method is primarily intended for CLI contexts where the variant
    /// name does not add value to end users. The returned string matches the
    /// [`std::fmt::Display`] implementation.
    pub fn to_display_string(&self) -> String {
        format!("{self}")
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(source: serde_yaml::Error) -> Self {
        Self::Parse {
            source
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Self::Serialize {
            source
        }
    }
}

impl From<masterror::AppError> for Error {
    fn from(error: masterror::AppError) -> Self {
        Self::Service {
            message: error.to_string()
        }
    }
}

/// Creates an [`Error::Io`] variant capturing the failing path and source.
///
/// # Parameters
///
/// * `path` - Location of the configuration file that triggered the error.
/// * `source` - I/O error reported by the operating system.
pub fn io_error(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source
    }
}

/// Creates an [`Error::BadgeIo`] variant capturing the failing path and source.
///
/// # Parameters
///
/// * `path` - Location of the badge artifact that triggered the error.
/// * `source` - I/O error reported by the operating system.
pub fn badge_io_error(path: &Path, source: std::io::Error) -> Error {
    Error::BadgeIo {
        path: path.to_path_buf(),
        source
    }
}

/// Creates an [`Error::SvgIo`] variant capturing the failing path and source.
///
/// # Parameters
///
/// * `path` - Location of the SVG file that triggered the error.
/// * `source` - I/O error reported by the operating system.
pub fn svg_io_error(path: &Path, source: std::io::Error) -> Error {
    Error::SvgIo {
        path: path.to_path_buf(),
        source
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn validation_constructor_populates_message() {
        let error = Error::validation("something went wrong");
        match error {
            Error::Validation {
                ref message
            } => {
                assert_eq!(message, "something went wrong");
            }
            other => panic!("expected validation error, got {other:?}")
        }
    }

    #[test]
    fn to_display_string_matches_display() {
        let error = Error::validation("display me");
        assert_eq!(error.to_string(), error.to_display_string());
    }

    #[test]
    fn io_error_helper_wraps_path_and_source() {
        let path = std::path::Path::new("/tmp/example.yaml");
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let error = super::io_error(path, io_error);

        match error {
            Error::Io {
                path: ref stored_path,
                ref source
            } => {
                assert_eq!(stored_path, path);
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("expected io error, got {other:?}")
        }
    }

    #[test]
    fn serde_yaml_conversion_maps_to_parse_variant() {
        let error = serde_yaml::from_str::<usize>("not-a-number").unwrap_err();
        let mapped: Error = error.into();
        assert!(matches!(mapped, Error::Parse { .. }));
    }

    #[test]
    fn serde_json_conversion_maps_to_serialize_variant() {
        let invalid = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        let mapped: Error = invalid.into();
        assert!(matches!(mapped, Error::Serialize { .. }));
    }

    #[test]
    fn badge_io_error_helper_wraps_path_and_source() {
        let path = std::path::Path::new("/tmp/badge.svg");
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let error = super::badge_io_error(path, io_error);

        match error {
            Error::BadgeIo {
                path: ref stored_path,
                ref source
            } => {
                assert_eq!(stored_path, path);
                assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            }
            other => panic!("expected badge io error, got {other:?}")
        }
    }
}
