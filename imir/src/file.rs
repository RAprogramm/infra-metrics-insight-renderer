// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// File operations for metrics artifacts.
///
/// Provides utilities for moving generated artifacts into repository workspace.
use std::path::{Path, PathBuf};

use masterror::AppError;
use serde::{Deserialize, Serialize};

/// Result of file move operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMoveResult {
    /// Destination path where file was moved.
    pub destination: PathBuf,
    /// Whether the operation succeeded.
    pub success:     bool
}

/// Moves a file from source to destination, creating parent directories.
///
/// # Arguments
///
/// * `source` - Source file path
/// * `destination` - Destination file path
///
/// # Returns
///
/// [`FileMoveResult`] containing the destination path.
///
/// # Errors
///
/// Returns [`AppError`] when source doesn't exist, destination parent cannot
/// be created, or move operation fails.
///
/// # Example
///
/// ```no_run
/// use imir::move_file;
///
/// # fn example() -> Result<(), masterror::AppError> {
/// let result = move_file("/tmp/artifact.svg", "metrics/profile.svg")?;
/// println!("Moved to: {}", result.destination.display());
/// # Ok(())
/// # }
/// ```
pub fn move_file(source: &str, destination: &str) -> Result<FileMoveResult, AppError> {
    let source_path = Path::new(source);
    let dest_path = Path::new(destination);

    if !source_path.exists() {
        return Err(AppError::validation(format!(
            "source file not found: {source}"
        )));
    }

    if !source_path.is_file() {
        return Err(AppError::validation(format!(
            "source is not a file: {source}"
        )));
    }

    if let Some(parent) = dest_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::service(format!(
                "failed to create parent directories for {}: {e}",
                dest_path.display()
            ))
        })?;
    }

    // Use copy+remove instead of rename to support cross-filesystem moves
    std::fs::copy(source_path, dest_path).map_err(|e| {
        AppError::service(format!("failed to copy {} to {}: {e}", source, destination))
    })?;

    std::fs::remove_file(source_path)
        .map_err(|e| AppError::service(format!("failed to remove source file {}: {e}", source)))?;

    Ok(FileMoveResult {
        destination: dest_path.to_path_buf(),
        success:     true
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn file_move_result_serialization() {
        let result = FileMoveResult {
            destination: PathBuf::from("metrics/profile.svg"),
            success:     true
        };

        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(json.contains("profile.svg",));
        assert!(json.contains("true",));
    }

    #[test]
    fn file_move_result_clone() {
        let result = FileMoveResult {
            destination: PathBuf::from("/test/path.svg"),
            success:     true
        };

        let cloned = result.clone();
        assert_eq!(result.destination, cloned.destination);
        assert_eq!(result.success, cloned.success);
    }

    #[test]
    fn move_file_rejects_nonexistent_source() {
        let result = move_file("/nonexistent/file.svg", "/tmp/dest.svg");
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err(),);
        assert!(error_msg.contains("source file not found"),);
    }

    #[test]
    fn move_file_rejects_directory_source() {
        let dir = tempdir().expect("failed to create tempdir");
        let result = move_file(dir.path().to_str().unwrap(), "/tmp/dest.svg");
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err(),);
        assert!(error_msg.contains("not a file"),);
    }

    #[test]
    fn move_file_creates_parent_directories() {
        let dir = tempdir().expect("failed to create tempdir");
        let source = dir.path().join("source.svg");
        std::fs::write(&source, "test").expect("failed to write source");

        let dest = dir.path().join("nested/dir/dest.svg");
        let result = move_file(source.to_str().unwrap(), dest.to_str().unwrap());

        assert!(result.is_ok());
        assert!(dest.exists());
        assert!(!source.exists());
    }

    #[test]
    fn move_file_succeeds_with_valid_inputs() {
        let dir = tempdir().expect("failed to create tempdir");
        let source = dir.path().join("source.svg");
        std::fs::write(&source, "test content").expect("failed to write source");

        let dest = dir.path().join("dest.svg");
        let result =
            move_file(source.to_str().unwrap(), dest.to_str().unwrap()).expect("move_file failed");

        assert!(result.success);
        assert_eq!(result.destination, dest);
        assert!(dest.exists());
        assert!(!source.exists());

        let content = std::fs::read_to_string(&dest).expect("failed to read dest");
        assert_eq!(content, "test content");
    }
}
