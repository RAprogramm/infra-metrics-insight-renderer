/// Builder for slug strings that can be used for branch names and filenames.
#[derive(Debug, Clone, Copy)]
pub struct SlugStrategy<'input> {
    source: &'input str,
}

impl<'input> SlugStrategy<'input> {
    /// Creates a new slug builder for the provided string slice.
    pub fn builder(source: &'input str) -> Self {
        Self { source }
    }

    /// Builds a slug from the provided source string. The slug contains only
    /// lowercase ASCII alphanumeric characters and single hyphen separators.
    /// Returns `None` when the input does not contain any slug-worthy
    /// characters after normalization.
    pub fn build(self) -> Option<String> {
        let trimmed = self.source.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut slug = String::with_capacity(trimmed.len());
        let mut previous_hyphen = false;

        for candidate in trimmed.chars() {
            match candidate {
                'A'..='Z' => {
                    slug.push(candidate.to_ascii_lowercase());
                    previous_hyphen = false;
                }
                'a'..='z' | '0'..='9' => {
                    slug.push(candidate);
                    previous_hyphen = false;
                }
                '-' | '_' | ' ' | '.' | '/' => {
                    if !previous_hyphen && !slug.is_empty() {
                        slug.push('-');
                        previous_hyphen = true;
                    }
                }
                _ => {
                    if !previous_hyphen && !slug.is_empty() {
                        slug.push('-');
                        previous_hyphen = true;
                    }
                }
            }
        }

        while slug.ends_with('-') {
            slug.pop();
        }

        if slug.is_empty() {
            None
        } else {
            Some(slug)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SlugStrategy;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn slug_contains_only_allowed_characters(input in "[A-Za-z0-9._/ -]{1,48}") {
            let builder = SlugStrategy::builder(&input);
            let slug = builder.build();
            prop_assert!(slug.is_none_or(|value| value.chars().all(|ch| matches!(ch, 'a'..='z' | '0'..='9' | '-'))));
        }
    }

    #[test]
    fn builder_discards_invalid_and_duplicate_separators() {
        let slug = SlugStrategy::builder("  Multi--Separator__Value  ")
            .build()
            .expect("expected slug to be derived");
        assert_eq!(slug, "multi-separator-value");
    }

    #[test]
    fn builder_returns_none_for_empty_input() {
        assert!(SlugStrategy::builder("   ").build().is_none());
        assert!(SlugStrategy::builder("***").build().is_none());
    }

    #[test]
    fn builder_lowercases_uppercase_characters() {
        let slug = SlugStrategy::builder("HelloWorld").build();
        assert_eq!(slug.as_deref(), Some("helloworld"));
    }
}
