use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;

/// Unified error type returned by the configuration loader and CLI.
#[derive(Debug, Error)]
pub enum Error {
    /// Wraps I/O errors that occur while reading configuration files.
    #[error("failed to read configuration from {path}: {source}")]
    Io {
        /// Location of the configuration file.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Wraps YAML decoding errors.
    #[error("failed to parse configuration: {source}")]
    Parse {
        /// Source decoding error from serde_yaml.
        source: serde_yaml::Error,
    },
    /// Returned when the configuration violates invariants.
    #[error("invalid configuration: {message}")]
    Validation {
        /// Human readable message describing the validation problem.
        message: String,
    },
    /// Wraps serialization errors when writing normalized output.
    #[error("failed to serialize targets: {source}")]
    Serialize {
        /// Underlying serialization error.
        source: serde_json::Error,
    },
}

impl Error {
    /// Constructs a validation error from the provided displayable value.
    pub fn validation<M>(message: M) -> Self
    where
        M: Into<String>,
    {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Formats the error for diagnostics without the variant name to ease CLI reporting.
    pub fn to_display_string(&self) -> String {
        format!("{self}")
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(source: serde_yaml::Error) -> Self {
        Self::Parse { source }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Self::Serialize { source }
    }
}

/// Helper function to create an [`Error::Io`] variant.
pub fn io_error(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}
