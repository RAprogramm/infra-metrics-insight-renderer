//! Utilities for generating rendering instructions for metrics dashboards.
//!
//! The library exposes helpers that load YAML configuration files describing
//! metrics targets and transform them into normalized descriptors suitable for
//! driving GitHub Actions matrices.

mod config;
mod error;
mod normalizer;
mod slug;

pub use config::{TargetConfig, TargetEntry, TargetKind};
pub use error::Error;
pub use normalizer::{load_targets, parse_targets, RenderTarget, TargetsDocument};
