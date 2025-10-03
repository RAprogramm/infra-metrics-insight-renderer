//! Utilities for generating rendering instructions for metrics dashboards.
//!
//! The library exposes helpers that load YAML configuration files describing
//! metrics targets and transform them into normalized descriptors suitable for
//! driving GitHub Actions matrices. All public APIs are documented with
//! invariants, error semantics, and minimal examples to facilitate integration
//! in automation tooling.

mod config;
mod error;
mod normalizer;
mod open_source;
mod slug;

pub use config::{TargetConfig, TargetEntry, TargetKind};
pub use error::{io_error, Error};
pub use normalizer::{load_targets, parse_targets, RenderTarget, TargetsDocument};
pub use open_source::resolve_open_source_repositories;
pub use slug::SlugStrategy;
