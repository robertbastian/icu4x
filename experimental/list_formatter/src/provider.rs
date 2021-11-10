// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Data provider struct definitions for this ICU4X component.
//!
//! Read more about data providers: [`icu_provider`]

use crate::options::Width;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use icu_provider::yoke::{self, *};

pub mod key {
    //! Resource keys for [`list_formatter`](crate).
    use icu_provider::{resource_key, ResourceKey};

    // Resource key: symbols used for list formatting.
    pub const LIST_FORMAT_AND_V1: ResourceKey = resource_key!(ListFormatter, "list/and", 1);
    pub const LIST_FORMAT_OR_V1: ResourceKey = resource_key!(ListFormatter, "list/or", 1);
    pub const LIST_FORMAT_UNIT_V1: ResourceKey = resource_key!(ListFormatter, "list/unit", 1);
}

/// Symbols and metadata required for [`ListFormatter`](crate::ListFormatter).
#[icu_provider::data_struct]
#[derive(Debug, PartialEq, Clone)]
#[yoke(cloning_zcf)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ListFormatterPatternsV1<'data> {
    // All patterns required by this instance
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    patterns: Vec<ConditionalListJoinerPattern<'data>>,

    // Indices into `patterns`. Each u8 fits two indices (4 bits each)
    start_middle: u8,
    end_pair: u8,

    short_start_middle: u8,
    short_end_pair: u8,

    narrow_start_middle: u8,
    narrow_end_pair: u8,
}

impl<'data> ListFormatterPatternsV1<'data> {
    #[cfg(any(test, feature = "provider_transform_internals"))]
    #[allow(clippy::too_many_arguments)] // same as constructor
    pub fn new(
        start: ConditionalListJoinerPattern<'data>,
        middle: ConditionalListJoinerPattern<'data>,
        end: ConditionalListJoinerPattern<'data>,
        pair: ConditionalListJoinerPattern<'data>,
        short_start: ConditionalListJoinerPattern<'data>,
        short_middle: ConditionalListJoinerPattern<'data>,
        short_end: ConditionalListJoinerPattern<'data>,
        short_pair: ConditionalListJoinerPattern<'data>,
        narrow_start: ConditionalListJoinerPattern<'data>,
        narrow_middle: ConditionalListJoinerPattern<'data>,
        narrow_end: ConditionalListJoinerPattern<'data>,
        narrow_pair: ConditionalListJoinerPattern<'data>,
    ) -> Self {
        let mut patterns = std::collections::HashSet::new();
        patterns.insert(start.clone());
        patterns.insert(middle.clone());
        patterns.insert(end.clone());
        patterns.insert(pair.clone());
        patterns.insert(short_start.clone());
        patterns.insert(short_middle.clone());
        patterns.insert(short_middle.clone());
        patterns.insert(short_pair.clone());
        patterns.insert(narrow_start.clone());
        patterns.insert(narrow_middle.clone());
        patterns.insert(narrow_end.clone());
        patterns.insert(narrow_pair.clone());
        patterns.remove(COMMA);

        let patterns: Vec<ConditionalListJoinerPattern<'data>> = patterns.into_iter().collect();

        let index = |pattern| {
            // Indices are increased by 1, with 0 meaning COMMA
            patterns
                .iter()
                .position(|p| p == pattern)
                .map(|index| index + 1)
                .unwrap_or(0) as u8
        };

        Self {
            start_middle: index(&start) << 4 | index(&middle),
            end_pair: index(&end) << 4 | index(&pair),
            short_start_middle: index(&short_start) << 4 | index(&short_middle),
            short_end_pair: index(&short_end) << 4 | index(&short_pair),
            narrow_start_middle: index(&narrow_start) << 4 | index(&narrow_middle),
            narrow_end_pair: index(&narrow_end) << 4 | index(&narrow_pair),
            patterns,
        }
    }

    fn get_pattern(&self, index: u8) -> &ConditionalListJoinerPattern<'data> {
        if index == 0 {
            COMMA
        } else {
            &self.patterns[(index - 1) as usize]
        }
    }

    pub fn start(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get_pattern(match width {
            Width::Wide => self.start_middle >> 4,
            Width::Short => self.short_start_middle >> 4,
            Width::Narrow => self.narrow_start_middle >> 4,
        })
    }

    pub fn middle(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get_pattern(match width {
            Width::Wide => self.start_middle & 0xF,
            Width::Short => self.short_start_middle & 0xF,
            Width::Narrow => self.narrow_start_middle & 0xF,
        })
    }

    pub fn end(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get_pattern(match width {
            Width::Wide => self.end_pair >> 4,
            Width::Short => self.short_end_pair >> 4,
            Width::Narrow => self.narrow_end_pair >> 4,
        })
    }

    pub fn pair(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get_pattern(match width {
            Width::Wide => self.end_pair & 0xF,
            Width::Short => self.short_end_pair & 0xF,
            Width::Narrow => self.narrow_end_pair & 0xF,
        })
    }

    #[cfg(any(test, feature = "provider_transform_internals"))]
    pub fn replace_patterns(
        &mut self,
        old: &ConditionalListJoinerPattern<'data>,
        new: &ConditionalListJoinerPattern<'data>,
    ) {
        for pattern in self.patterns.as_mut_slice() {
            if pattern == old {
                *pattern = new.clone();
            }
        }
    }
}

const COMMA: &ConditionalListJoinerPattern = &ConditionalListJoinerPattern {
    default: ListJoinerPattern {
        string: Cow::Borrowed(", "),
        index_1: 2,
    },
    special_case: None,
};

/// A pattern that can behave conditionally on the next element.
#[derive(Debug, PartialEq, Eq, Clone, Yokeable, ZeroCopyFrom, Hash)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ConditionalListJoinerPattern<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    default: ListJoinerPattern<'data>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    special_case: Option<SpecialCasePattern<'data>>,
}

/// A pattern that can behave conditionally on the next element.
#[derive(Debug, PartialEq, Eq, Clone, Yokeable, ZeroCopyFrom, Hash)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
struct SpecialCasePattern<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    condition: Cow<'data, str>, // TODO: Serialize a compiled regex instead
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    pattern: ListJoinerPattern<'data>,
}

/// A pattern containing two numeric placeholders ("{0}, and {1}.")
#[derive(Debug, PartialEq, Eq, Clone, Yokeable, ZeroCopyFrom, Hash)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
struct ListJoinerPattern<'data> {
    /// The pattern string without the placeholders
    string: Cow<'data, str>,
    /// The index of the second placeholder
    index_1: u8,
}

pub type PatternParts<'a> = (&'a str, &'a str);

impl<'data> ListJoinerPattern<'data> {
    fn borrow_tuple(&'data self) -> PatternParts<'data> {
        (
            &self.string[0..self.index_1 as usize],
            &self.string[self.index_1 as usize..],
        )
    }
}

impl<'a> ConditionalListJoinerPattern<'a> {
    pub fn parts(&'a self, following_value: &str) -> PatternParts<'a> {
        match &self.special_case {
            Some(SpecialCasePattern { condition, pattern })
                if regex::Regex::new(&*condition)
                    .unwrap()
                    .is_match(following_value) =>
            {
                pattern.borrow_tuple()
            }
            _ => self.default.borrow_tuple(),
        }
    }
}

#[cfg(any(test, feature = "provider_transform_internals"))]
pub mod pattern_construction {
    use super::*;
    use crate::error::Error;
    use core::str::FromStr;

    impl<'data> FromStr for ListJoinerPattern<'data> {
        type Err = Error;
        fn from_str(pattern: &str) -> Result<Self, Self::Err> {
            match (pattern.find("{0}"), pattern.find("{1}")) {
                (Some(0), Some(index_1)) if index_1 - 3 < 256 => Ok(ListJoinerPattern {
                    string: Cow::Owned(pattern[3..index_1].to_string() + &pattern[index_1 + 3..]),
                    index_1: (index_1 - 3) as u8,
                }),
                _ => Err(Error::IllegalPattern(pattern.to_string())),
            }
        }
    }

    impl<'data> FromStr for ConditionalListJoinerPattern<'data> {
        type Err = Error;
        fn from_str(pattern: &str) -> Result<Self, Self::Err> {
            Ok(ConditionalListJoinerPattern {
                default: ListJoinerPattern::from_str(pattern)?,
                special_case: None,
            })
        }
    }

    /// Creates a conditional list joiner that will evaluate to the `then_pattern` when
    /// `regex` matches the following element, and to `else_pattern` otherwise.
    /// The regex is interpreted case-insensitive and anchored to the beginning, but
    /// to improve efficiency does not search for full matches. If a full match is
    /// required, use `$`.
    impl<'data> ConditionalListJoinerPattern<'data> {
        pub fn from_regex_and_strs(
            regex: &str,
            then_pattern: &str,
            else_pattern: &str,
        ) -> Result<Self, crate::error::Error> {
            Ok(ConditionalListJoinerPattern {
                default: ListJoinerPattern::from_str(else_pattern)?,
                special_case: Some(SpecialCasePattern {
                    condition: Cow::Owned("(?i)^(".to_string() + regex + ")"),
                    pattern: ListJoinerPattern::from_str(then_pattern)?,
                }),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn produces_correct_parts() {
        let pattern = ConditionalListJoinerPattern::from_str("{0}a{1}b").unwrap();
        assert_eq!(pattern.parts(""), ("a", "b"));
    }

    #[test]
    fn produces_correct_parts_conditionally() {
        let pattern =
            ConditionalListJoinerPattern::from_regex_and_strs("b", "{0}c{1}d", "{0}a{1}b").unwrap();
        // Only matches at the beginning of the string
        assert_eq!(pattern.parts("ab"), ("a", "b"));
        // Doesn't require a full match
        assert_eq!(pattern.parts("ba"), ("c", "d"));
    }

    #[test]
    fn fallbacks_work() {
        let comma = COMMA.clone();
        let period = ConditionalListJoinerPattern::from_str("{0}. {1}").unwrap();
        let semicolon = ConditionalListJoinerPattern::from_str("{0}; {1}").unwrap();
        let colon = ConditionalListJoinerPattern::from_str("{0}: {1}").unwrap();

        // Different fields are returned correctly
        let pattern = ListFormatterPatternsV1::new(
            comma.clone(),
            period.clone(),
            semicolon.clone(),
            colon.clone(),
            comma.clone(),
            period.clone(),
            semicolon.clone(),
            colon.clone(),
            comma.clone(),
            period.clone(),
            semicolon.clone(),
            colon.clone(),
        );
        assert_eq!(pattern.start(Width::Wide), &comma);
        assert_eq!(pattern.middle(Width::Wide), &period);
        assert_eq!(pattern.end(Width::Wide), &semicolon);
        assert_eq!(pattern.pair(Width::Wide), &colon);

        // Same fields are returned correctly
        let pattern = ListFormatterPatternsV1::new(
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
            comma.clone(),
        );
        assert_eq!(pattern.start(Width::Wide), &comma);
        assert_eq!(pattern.middle(Width::Wide), &comma);
        assert_eq!(pattern.end(Width::Wide), &comma);
        assert_eq!(pattern.pair(Width::Wide), &comma);

        // Pair/end fallback works correctly
        let pattern = ListFormatterPatternsV1::new(
            comma.clone(),
            comma.clone(),
            period.clone(),
            period.clone(),
            comma.clone(),
            comma.clone(),
            period.clone(),
            period.clone(),
            comma.clone(),
            comma.clone(),
            period.clone(),
            period.clone(),
        );
        assert_eq!(pattern.start(Width::Wide), &comma);
        assert_eq!(pattern.middle(Width::Wide), &comma);
        assert_eq!(pattern.end(Width::Wide), &period);
        assert_eq!(pattern.pair(Width::Wide), &period);
    }
}
