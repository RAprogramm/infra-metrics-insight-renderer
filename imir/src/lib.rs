//! Utilities for generating rendering instructions for metrics dashboards.
//!
//! The library exposes helpers that load YAML configuration files describing
//! metrics targets and transform them into normalized descriptors suitable for
//! driving GitHub Actions matrices. All public APIs are documented with
//! invariants, error semantics, and minimal examples to facilitate integration
//! in automation tooling.
//!
//! # Examples
//!
//! Parse a configuration document that customizes the badge widget and inspect
//! the resulting normalized descriptor:
//!
//! ```
//! use imir::{parse_targets, BadgeStyle, Error};
//!
//! # fn main() -> Result<(), Error> {
//! let yaml = r#"
//! targets:
//!   - owner: octocat
//!     repo: metrics
//!     type: open_source
//!     badge:
//!       style: flat
//!       widget:
//!         columns: 2
//!         alignment: center
//! "#;
//!
//! let document = parse_targets(yaml)?;
//! assert_eq!(document.targets[0].badge.style, BadgeStyle::Flat);
//! assert_eq!(document.targets[0].badge.widget.columns, 2);
//! # Ok(())
//! # }
//! ```

mod badge;
mod config;
mod error;
mod normalizer;
mod open_source;
mod slug;

pub use badge::{BadgeAssets, generate_badge_assets};
pub use config::{
    BadgeOptions, BadgeStyle, BadgeWidgetAlignment, BadgeWidgetOptions, TargetConfig, TargetEntry,
    TargetKind,
};
pub use error::{Error, io_error};
pub use normalizer::{
    BadgeDescriptor, BadgeWidgetDescriptor, RenderTarget, TargetsDocument, load_targets,
    parse_targets,
};
pub use open_source::resolve_open_source_repositories;
pub use slug::SlugStrategy;
