// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Badge asset generation utilities.
//!
//! The module materializes lightweight SVG placeholders alongside JSON
//! manifests that describe the normalized render target. The artifacts are
//! deterministic so they can be checked into the repository prior to the first
//! automation run.

use std::{
    borrow::Cow,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf}
};

use serde::Serialize;

use crate::{
    config::TargetKind,
    error::{self, Error},
    normalizer::{BadgeDescriptor, RenderTarget}
};

/// Result of generating badge assets for a render target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadgeAssets {
    /// Absolute path to the rendered SVG badge.
    pub svg_path:      PathBuf,
    /// Absolute path to the JSON manifest describing the badge.
    pub manifest_path: PathBuf
}

/// Generates badge assets for the provided render target inside `output_dir`.
///
/// The function creates the directory hierarchy if it does not exist, writes a
/// deterministic SVG placeholder, and stores a JSON manifest that mirrors the
/// normalized configuration.
///
/// # Errors
///
/// Returns [`Error::BadgeIo`](Error::BadgeIo) when directories or files cannot
/// be created and [`Error::Serialize`](Error::Serialize) if the manifest cannot
/// be encoded.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
///
/// use imir::{generate_badge_assets, load_targets};
///
/// # fn main() -> Result<(), imir::Error> {
/// let document = load_targets(Path::new("targets/targets.yaml"))?;
/// let target = &document.targets[0];
///
/// let assets = generate_badge_assets(target, Path::new("metrics"))?;
/// println!("SVG: {}", assets.svg_path.display());
/// println!("Manifest: {}", assets.manifest_path.display());
/// # Ok(())
/// # }
/// ```
pub fn generate_badge_assets(
    target: &RenderTarget,
    output_dir: &Path
) -> Result<BadgeAssets, Error> {
    fs::create_dir_all(output_dir).map_err(|source| error::badge_io_error(output_dir, source))?;

    let svg_path = output_dir.join(format!("{}.svg", target.slug,));
    let manifest_path = output_dir.join(format!("{}.json", target.slug,));

    write_svg(&svg_path, target)?;
    write_manifest(&manifest_path, target, &svg_path)?;

    Ok(BadgeAssets {
        svg_path,
        manifest_path
    })
}

fn write_svg(path: &Path, target: &RenderTarget) -> Result<(), Error> {
    let contents = build_svg_content(target);
    let file = File::create(path).map_err(|source| error::badge_io_error(path, source))?;
    let mut writer = BufWriter::new(file);
    writer
        .write_all(contents.as_bytes())
        .map_err(|source| error::badge_io_error(path, source))?;
    writer
        .flush()
        .map_err(|source| error::badge_io_error(path, source))
}

fn write_manifest(path: &Path, target: &RenderTarget, svg_path: &Path) -> Result<(), Error> {
    let manifest = BadgeManifest {
        slug:         &target.slug,
        owner:        &target.owner,
        repository:   target.repository.as_deref(),
        kind:         target.kind,
        display_name: &target.display_name,
        target_path:  &target.target_path,
        svg_artifact: path_to_string(svg_path),
        badge:        &target.badge
    };

    let file = File::create(path).map_err(|source| error::badge_io_error(path, source))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &manifest)?;
    writer
        .write_all(b"\n")
        .map_err(|source| error::badge_io_error(path, source))?;
    writer
        .flush()
        .map_err(|source| error::badge_io_error(path, source))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn build_svg_content(target: &RenderTarget) -> String {
    use std::fmt::Write as _;

    let mut buffer = String::with_capacity(256);
    let background = badge_background(target.kind);
    let label = badge_label(target);
    let escaped_label = escape_xml(&label);
    let escaped_display = escape_xml(&target.display_name);

    let _ = writeln!(
        buffer,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-label=\"{escaped_display}\" width=\"440\" height=\"140\" viewBox=\"0 0 440 140\">",
    );
    let _ = writeln!(
        buffer,
        "  <defs>\n    <linearGradient id=\"imir-badge\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"1\">\n      <stop offset=\"0%\" stop-color=\"{}\" stop-opacity=\"0.92\"/>\n      <stop offset=\"100%\" stop-color=\"{}\" stop-opacity=\"1\"/>\n    </linearGradient>\n  </defs>",
        background.primary, background.secondary,
    );
    buffer.push_str("  <rect x=\"8\" y=\"8\" width=\"424\" height=\"124\" rx=\"16\" fill=\"url(#imir-badge)\"/>");
    let _ = writeln!(
        buffer,
        "\n  <text x=\"220\" y=\"60\" text-anchor=\"middle\" font-family=\"'Segoe UI', 'SF Pro Display', sans-serif\" font-size=\"22\" fill=\"#ffffff\">{escaped_label}</text>",
    );
    let _ = writeln!(
        buffer,
        "  <text x=\"220\" y=\"98\" text-anchor=\"middle\" font-family=\"'Segoe UI', 'SF Pro Display', sans-serif\" font-size=\"18\" fill=\"#f6f8fa\">{escaped_display}</text>",
    );
    buffer.push_str("</svg>\n");

    buffer
}

fn badge_label(target: &RenderTarget) -> Cow<'_, str> {
    match target.repository.as_deref() {
        Some(repository) => {
            let mut owned = String::with_capacity(target.owner.len() + repository.len() + 1);
            owned.push_str(target.owner.as_str());
            owned.push('/');
            owned.push_str(repository);
            Cow::Owned(owned)
        }
        None => Cow::Borrowed(target.owner.as_str())
    }
}

fn escape_xml(value: &str) -> Cow<'_, str> {
    if value
        .chars()
        .any(|character| matches!(character, '&' | '<' | '>' | '\"' | '\''))
    {
        let mut escaped = String::with_capacity(value.len());
        for character in value.chars() {
            match character {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '\"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&apos;"),
                other => escaped.push(other)
            }
        }
        Cow::Owned(escaped)
    } else {
        Cow::Borrowed(value)
    }
}

struct BadgeGradient {
    primary:   &'static str,
    secondary: &'static str
}

fn badge_background(kind: TargetKind) -> BadgeGradient {
    match kind {
        TargetKind::Profile => BadgeGradient {
            primary:   "#6f42c1",
            secondary: "#8648d1"
        },
        TargetKind::OpenSource => BadgeGradient {
            primary:   "#1f883d",
            secondary: "#2ea043"
        },
        TargetKind::PrivateProject => BadgeGradient {
            primary:   "#0a3069",
            secondary: "#1b4b91"
        }
    }
}

#[derive(Serialize)]
struct BadgeManifest<'a> {
    slug:         &'a str,
    owner:        &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository:   Option<&'a str>,
    kind:         TargetKind,
    display_name: &'a str,
    target_path:  &'a str,
    svg_artifact: String,
    badge:        &'a BadgeDescriptor
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::{BadgeStyle, BadgeWidgetAlignment},
        normalizer::BadgeWidgetDescriptor
    };

    fn sample_target(kind: TargetKind) -> RenderTarget {
        RenderTarget {
            slug: "sample".to_owned(),
            owner: "octocat".to_owned(),
            repository: Some("example".to_owned()),
            kind,
            branch_name: "branch".to_owned(),
            target_path: "metrics/sample.svg".to_owned(),
            temp_artifact: "tmp/sample.svg".to_owned(),
            time_zone: "UTC".to_owned(),
            display_name: "Example Dashboard".to_owned(),
            contributors_branch: "main".to_owned(),
            include_private: false,
            badge: BadgeDescriptor {
                style:  BadgeStyle::Classic,
                widget: BadgeWidgetDescriptor {
                    columns:       2,
                    alignment:     BadgeWidgetAlignment::Center,
                    border_radius: 6
                }
            }
        }
    }

    #[test]
    fn generate_badge_assets_writes_svg_and_manifest() {
        let target = sample_target(TargetKind::OpenSource);
        let directory = tempdir().expect("failed to create temp dir");
        let output_dir = directory.path().join("out");

        let assets = generate_badge_assets(&target, &output_dir)
            .expect("expected badge generation to succeed");

        assert!(assets.svg_path.exists());
        assert!(assets.manifest_path.exists());

        let svg = fs::read_to_string(&assets.svg_path).expect("expected svg to be readable");
        assert!(svg.contains("octocat/example"));
        assert!(svg.contains("Example Dashboard"));
        assert!(svg.contains("#2ea043"));

        let manifest =
            fs::read_to_string(&assets.manifest_path).expect("expected manifest to be readable");
        let value: Value =
            serde_json::from_str(&manifest).expect("expected manifest to be valid JSON");
        assert_eq!(value["slug"], "sample");
        assert_eq!(value["owner"], "octocat");
        assert_eq!(value["repository"], "example");
        assert_eq!(value["kind"], "open_source");
        assert_eq!(value["target_path"], "metrics/sample.svg");
        assert!(value["svg_artifact"].as_str().is_some());
    }

    #[test]
    fn generate_badge_assets_propagates_directory_errors() {
        let target = sample_target(TargetKind::Profile);
        let directory = tempdir().expect("failed to create temp dir");
        let file_path = directory.path().join("blocked");
        File::create(&file_path).expect("failed to create placeholder file");

        let error = generate_badge_assets(&target, &file_path).expect_err("expected io failure");

        match error {
            Error::BadgeIo {
                path, ..
            } => {
                assert_eq!(path, file_path);
            }
            other => panic!("unexpected error variant: {other:?}")
        }
    }

    #[test]
    fn svg_renderer_escapes_dynamic_content() {
        let mut target = sample_target(TargetKind::PrivateProject);
        target.display_name = "ACME & <Partners>".to_owned();
        target.repository = None;
        target.owner = "Org > Team".to_owned();

        let svg = build_svg_content(&target);
        assert!(svg.contains("Org &gt; Team"));
        assert!(svg.contains("ACME &amp; &lt;Partners&gt;"));
    }

    #[test]
    fn escape_xml_handles_all_special_characters() {
        let input = "&<>\"'normal";
        let result = escape_xml(input);
        assert_eq!(result, "&amp;&lt;&gt;&quot;&apos;normal");
    }

    #[test]
    fn escape_xml_returns_borrowed_when_no_escaping_needed() {
        let input = "no special characters";
        let result = escape_xml(input);
        match result {
            Cow::Borrowed(s) => assert_eq!(s, input),
            Cow::Owned(_) => panic!("expected borrowed variant",)
        }
    }

    #[test]
    fn badge_label_formats_repository_correctly() {
        let target = sample_target(TargetKind::OpenSource);
        let label = badge_label(&target);
        assert_eq!(label, "octocat/example");
    }

    #[test]
    fn badge_label_uses_owner_when_no_repository() {
        let mut target = sample_target(TargetKind::Profile);
        target.repository = None;
        let label = badge_label(&target);
        assert_eq!(label, "octocat");
    }

    #[test]
    fn badge_background_returns_correct_gradient_for_profile() {
        let gradient = badge_background(TargetKind::Profile);
        assert_eq!(gradient.primary, "#6f42c1");
        assert_eq!(gradient.secondary, "#8648d1");
    }

    #[test]
    fn badge_background_returns_correct_gradient_for_open_source() {
        let gradient = badge_background(TargetKind::OpenSource);
        assert_eq!(gradient.primary, "#1f883d");
        assert_eq!(gradient.secondary, "#2ea043");
    }

    #[test]
    fn badge_background_returns_correct_gradient_for_private() {
        let gradient = badge_background(TargetKind::PrivateProject);
        assert_eq!(gradient.primary, "#0a3069");
        assert_eq!(gradient.secondary, "#1b4b91");
    }

    #[test]
    fn path_to_string_converts_path_correctly() {
        let path = Path::new("/tmp/test.svg");
        let result = path_to_string(path);
        assert_eq!(result, "/tmp/test.svg");
    }

    #[test]
    fn badge_assets_equality() {
        let assets1 = BadgeAssets {
            svg_path:      PathBuf::from("/tmp/a.svg"),
            manifest_path: PathBuf::from("/tmp/a.json")
        };
        let assets2 = BadgeAssets {
            svg_path:      PathBuf::from("/tmp/a.svg"),
            manifest_path: PathBuf::from("/tmp/a.json")
        };
        assert_eq!(assets1, assets2);
    }

    #[test]
    fn badge_assets_clone() {
        let assets = BadgeAssets {
            svg_path:      PathBuf::from("/tmp/test.svg"),
            manifest_path: PathBuf::from("/tmp/test.json")
        };
        let cloned = assets.clone();
        assert_eq!(assets.svg_path, cloned.svg_path);
        assert_eq!(assets.manifest_path, cloned.manifest_path);
    }

    #[test]
    fn badge_assets_debug_format() {
        let assets = BadgeAssets {
            svg_path:      PathBuf::from("/tmp/debug.svg"),
            manifest_path: PathBuf::from("/tmp/debug.json")
        };
        let debug_str = format!("{:?}", assets);
        assert!(debug_str.contains("BadgeAssets"));
        assert!(debug_str.contains("svg_path"));
    }

    #[test]
    fn write_svg_creates_valid_file() {
        let target = sample_target(TargetKind::OpenSource);
        let directory = tempdir().expect("failed to create temp dir");
        let svg_path = directory.path().join("test.svg");

        write_svg(&svg_path, &target).expect("write should succeed");

        assert!(svg_path.exists());
        let contents = fs::read_to_string(&svg_path).expect("should read svg");
        assert!(contents.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(contents.contains("octocat/example"));
    }

    #[test]
    fn write_manifest_creates_valid_json() {
        let target = sample_target(TargetKind::Profile);
        let directory = tempdir().expect("failed to create temp dir");
        let manifest_path = directory.path().join("test.json");
        let svg_path = PathBuf::from("/tmp/test.svg");

        write_manifest(&manifest_path, &target, &svg_path).expect("write should succeed");

        assert!(manifest_path.exists());
        let contents = fs::read_to_string(&manifest_path).expect("should read manifest");
        let value: Value = serde_json::from_str(&contents).expect("should parse json");
        assert_eq!(value["slug"], "sample");
        assert_eq!(value["kind"], "profile");
    }

    #[test]
    fn svg_content_includes_gradient_definition() {
        let target = sample_target(TargetKind::PrivateProject);
        let svg = build_svg_content(&target);
        assert!(svg.contains("<linearGradient id=\"imir-badge\""));
        assert!(svg.contains("#0a3069"));
        assert!(svg.contains("#1b4b91"));
    }

    #[test]
    fn svg_content_includes_text_elements() {
        let target = sample_target(TargetKind::OpenSource);
        let svg = build_svg_content(&target);
        assert!(svg.contains("<text"));
        assert!(svg.contains("octocat/example"));
        assert!(svg.contains("Example Dashboard"));
    }
}
