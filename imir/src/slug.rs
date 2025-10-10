// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
//
// SPDX-License-Identifier: MIT

//! Utilities for deriving stable slugs from user-supplied strings.
//!
//! Slugs produced by this module contain only lowercase ASCII alphanumeric
//! characters separated by single hyphens, making them suitable for branch
//! names and filesystem paths across platforms.

/// Builder for slug strings that can be used for branch names and filenames.
#[derive(Debug, Clone, Copy,)]
pub struct SlugStrategy<'input,>
{
    source: &'input str,
}

impl<'input,> SlugStrategy<'input,>
{
    /// Creates a new slug builder for the provided string slice.
    ///
    /// The builder retains a borrowed view of the source to avoid allocations
    /// until [`build`](Self::build) is invoked.
    pub fn builder(source: &'input str,) -> Self
    {
        Self {
            source,
        }
    }

    /// Builds a slug from the provided source string. The slug contains only
    /// lowercase ASCII alphanumeric characters and single hyphen separators.
    /// Returns `None` when the input does not contain any slug-worthy
    /// characters after normalization.
    ///
    /// # Examples
    ///
    /// ```
    /// use imir::SlugStrategy;
    ///
    /// let slug = SlugStrategy::builder(" Docs/Overview  ",).build();
    /// assert_eq!(slug.as_deref(), Some("docs-overview"));
    /// ```
    pub fn build(self,) -> Option<String,>
    {
        let trimmed = self.source.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut slug = String::with_capacity(trimmed.len(),);
        let mut previous_hyphen = false;

        for candidate in trimmed.chars() {
            match candidate {
                'A'..='Z' => {
                    slug.push(candidate.to_ascii_lowercase(),);
                    previous_hyphen = false;
                }
                'a'..='z' | '0'..='9' => {
                    slug.push(candidate,);
                    previous_hyphen = false;
                }
                '-' | '_' | ' ' | '.' | '/' => {
                    if !previous_hyphen && !slug.is_empty() {
                        slug.push('-',);
                        previous_hyphen = true;
                    }
                }
                _ => {
                    if !previous_hyphen && !slug.is_empty() {
                        slug.push('-',);
                        previous_hyphen = true;
                    }
                }
            }
        }

        while slug.ends_with('-',) {
            slug.pop();
        }

        if slug.is_empty() { None } else { Some(slug,) }
    }
}

#[cfg(test)]
mod tests
{
    use proptest::prelude::*;

    use super::SlugStrategy;

    proptest! {
        #[test]
        fn slug_contains_only_allowed_characters(input in "[A-Za-z0-9._/ -]{1,48}") {
            let builder = SlugStrategy::builder(&input);
            let slug = builder.build();
            prop_assert!(slug.is_none_or(|value| value.chars().all(|ch| matches!(ch, 'a'..='z' | '0'..='9' | '-'))));
        }
    }

    #[test]
    fn builder_discards_invalid_and_duplicate_separators()
    {
        let slug = SlugStrategy::builder("  Multi--Separator__Value  ",)
            .build()
            .expect("expected slug to be derived",);
        assert_eq!(slug, "multi-separator-value");
    }

    #[test]
    fn builder_returns_none_for_empty_input()
    {
        assert!(SlugStrategy::builder("   ").build().is_none());
        assert!(SlugStrategy::builder("***").build().is_none());
    }

    #[test]
    fn builder_lowercases_uppercase_characters()
    {
        let slug = SlugStrategy::builder("HelloWorld",).build();
        assert_eq!(slug.as_deref(), Some("helloworld"));
    }

    #[test]
    fn builder_handles_slashes_and_dots()
    {
        let slug = SlugStrategy::builder("path/to/file.txt",).build();
        assert_eq!(slug.as_deref(), Some("path-to-file-txt"));
    }

    #[test]
    fn builder_removes_trailing_hyphens()
    {
        let slug = SlugStrategy::builder("test---",).build();
        assert_eq!(slug.as_deref(), Some("test"));
    }

    #[test]
    fn builder_handles_leading_separators()
    {
        let slug = SlugStrategy::builder("---test",).build();
        assert_eq!(slug.as_deref(), Some("test"));
    }

    #[test]
    fn builder_handles_numbers()
    {
        let slug = SlugStrategy::builder("test123",).build();
        assert_eq!(slug.as_deref(), Some("test123"));
    }

    #[test]
    fn builder_handles_unicode_characters()
    {
        let slug = SlugStrategy::builder("hello-世界-test",).build();
        assert_eq!(slug.as_deref(), Some("hello-test"));
    }

    #[test]
    fn builder_handles_underscores()
    {
        let slug = SlugStrategy::builder("snake_case_name",).build();
        assert_eq!(slug.as_deref(), Some("snake-case-name"));
    }

    #[test]
    fn builder_handles_spaces()
    {
        let slug = SlugStrategy::builder("word with spaces",).build();
        assert_eq!(slug.as_deref(), Some("word-with-spaces"));
    }

    #[test]
    fn builder_handles_special_characters()
    {
        let slug = SlugStrategy::builder("test!@#$%^&*()",).build();
        assert_eq!(slug.as_deref(), Some("test"));
    }

    #[test]
    fn builder_handles_mixed_case_with_separators()
    {
        let slug = SlugStrategy::builder("My Project/Version 2.0",).build();
        assert_eq!(slug.as_deref(), Some("my-project-version-2-0"));
    }

    #[test]
    fn slug_strategy_copy_trait()
    {
        let builder1 = SlugStrategy::builder("test",);
        let builder2 = builder1;
        assert_eq!(builder1.build(), builder2.build());
    }

    #[test]
    fn slug_strategy_clone_trait()
    {
        let builder = SlugStrategy::builder("clone-test",);
        let cloned = builder;
        assert_eq!(builder.build(), cloned.build());
    }

    #[test]
    fn slug_strategy_debug_format()
    {
        let builder = SlugStrategy::builder("debug",);
        let debug_str = format!("{:?}", builder);
        assert!(debug_str.contains("SlugStrategy"));
        assert!(debug_str.contains("source"));
    }

    #[test]
    fn builder_handles_only_separators()
    {
        let slug = SlugStrategy::builder("---___...///",).build();
        assert!(slug.is_none());
    }

    #[test]
    fn builder_preserves_capacity_optimization()
    {
        let input = "a".repeat(100,);
        let slug = SlugStrategy::builder(&input,).build();
        assert_eq!(slug.as_deref(), Some(input.as_str()));
    }
}
