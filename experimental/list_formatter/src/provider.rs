// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Data provider struct definitions for this ICU4X component.
//!
//! Read more about data providers: [`icu_provider`]

use crate::options::Width;
use alloc::borrow::Cow;
use icu_provider::yoke::{self, *};

pub mod key {
    //! Resource keys for [`list_formatter`](crate).
    use icu_provider::{resource_key, ResourceKey};

    // Resource key: symbols used for list formatting.
    pub const LIST_FORMAT_AND_V1: ResourceKey = resource_key!(ListFormatter, "and", 1);
    pub const LIST_FORMAT_OR_V1: ResourceKey = resource_key!(ListFormatter, "or", 1);
    pub const LIST_FORMAT_UNIT_V1: ResourceKey = resource_key!(ListFormatter, "unit", 1);
}

/// Symbols and metadata required for [`ListFormatter`](crate::ListFormatter).
/// Absent values follow this fallback structure:
#[icu_provider::data_struct]
#[derive(Debug)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct ListFormatterPatternsV1<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    patterns: [OptimizedConditionalListJoinerPattern<'data>; 12],
}

#[derive(Debug, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
#[serde(untagged)]
enum OptimizedConditionalListJoinerPattern<'data> {
    Comma,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    Value(ConditionalListJoinerPattern<'data>),
    Fallback(Field),
}

#[derive(Clone, Copy, Debug, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
enum Field {
    Start,
    Middle,
    End,
    Pair,
    ShortStart,
    ShortMiddle,
    ShortEnd,
    ShortPair,
    NarrowStart,
    NarrowMiddle,
    NarrowEnd,
    NarrowPair,
}

impl<'data> ListFormatterPatternsV1<'data> {
    fn get(&self, field: Field) -> &ConditionalListJoinerPattern<'data> {
        match &self.patterns[field as usize] {
            OptimizedConditionalListJoinerPattern::Comma => Default::default(),
            OptimizedConditionalListJoinerPattern::Value(pattern) => pattern,
            // Valid structs will only have one layer of indirection.
            OptimizedConditionalListJoinerPattern::Fallback(fallback) => self.get(*fallback),
        }
    }

    pub fn start(&'data self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get(match width {
            Width::Wide => Field::Start,
            Width::Short => Field::ShortStart,
            Width::Narrow => Field::NarrowStart,
        })
    }

    pub fn middle(&'data self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get(match width {
            Width::Wide => Field::Middle,
            Width::Short => Field::ShortMiddle,
            Width::Narrow => Field::NarrowMiddle,
        })
    }

    pub fn end(&'data self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get(match width {
            Width::Wide => Field::End,
            Width::Short => Field::ShortEnd,
            Width::Narrow => Field::NarrowEnd,
        })
    }

    pub fn pair(&'data self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        self.get(match width {
            Width::Wide => Field::Pair,
            Width::Short => Field::ShortPair,
            Width::Narrow => Field::NarrowPair,
        })
    }
}

/// A pattern that can behave conditionally on the next element.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
pub struct ConditionalListJoinerPattern<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    default: ListJoinerPattern<'data>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    special_case: Option<SpecialCasePattern<'data>>,
}

impl<'data> Default for &ConditionalListJoinerPattern<'data> {
    fn default() -> &'static ConditionalListJoinerPattern<'data> {
        &ConditionalListJoinerPattern {
            default: ListJoinerPattern {
                string: Cow::Borrowed(", "),
                index_1: 2,
            },
            special_case: None,
        }
    }
}

/// A pattern that can behave conditionally on the next element.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
)]
struct SpecialCasePattern<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    condition: Cow<'data, str>, // TODO: Serialize a compiled regex instead
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    pattern: ListJoinerPattern<'data>,
}

/// A pattern containing two numeric placeholders ("{0}, and {1}.")
#[derive(Clone, Debug, Eq, Hash, PartialEq, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Deserialize, serde::Serialize)
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
    use std::collections::HashMap;
    use std::str::FromStr;

    impl<'data> Default for OptimizedConditionalListJoinerPattern<'data> {
        fn default() -> Self {
            OptimizedConditionalListJoinerPattern::Comma
        }
    }

    impl<'data> ListFormatterPatternsV1<'data> {
        #[allow(clippy::too_many_arguments)] // We don't want to expose the Field enum
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
            let mut pattern_to_fields: HashMap<ConditionalListJoinerPattern<'data>, Vec<Field>> =
                HashMap::new();
            for (pattern, field) in [
                (start, Field::Start),
                (middle, Field::Middle),
                (end, Field::End),
                (pair, Field::Pair),
                (short_start, Field::ShortStart),
                (short_middle, Field::ShortMiddle),
                (short_end, Field::ShortEnd),
                (short_pair, Field::ShortPair),
                (narrow_start, Field::NarrowStart),
                (narrow_middle, Field::NarrowMiddle),
                (narrow_end, Field::NarrowEnd),
                (narrow_pair, Field::NarrowPair),
            ] {
                pattern_to_fields.entry(pattern).or_default().push(field);
            }
            pattern_to_fields.remove(Default::default());

            let mut patterns: [OptimizedConditionalListJoinerPattern; 12] = Default::default();

            for (pattern, mut fields) in pattern_to_fields {
                let representative = fields.remove(0);
                patterns[representative as usize] =
                    OptimizedConditionalListJoinerPattern::Value(pattern);
                for field in fields {
                    patterns[field as usize] =
                        OptimizedConditionalListJoinerPattern::Fallback(representative);
                }
            }

            Self { patterns }
        }
    }

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

    impl<'data> ConditionalListJoinerPattern<'data> {
        pub fn from_regex_and_strs(
            regex: &str,
            then_pattern: &str,
            else_pattern: &str,
        ) -> Result<Self, crate::error::Error> {
            Ok(ConditionalListJoinerPattern {
                default: ListJoinerPattern::from_str(else_pattern)?,
                special_case: Some(SpecialCasePattern {
                    condition: Cow::Owned(regex.to_string()),
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
            ConditionalListJoinerPattern::from_regex_and_strs("b.*", "{0}c{1}d", "{0}a{1}b")
                .unwrap();
        assert_eq!(pattern.parts("a"), ("a", "b"));
        assert_eq!(pattern.parts("b"), ("c", "d"));
    }

    #[test]
    fn fallbacks_work() {
        let comma = ConditionalListJoinerPattern::from_str("{0}, {1}").unwrap();
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
        assert_eq!(pattern.start(Width::Short), &comma);
        assert_eq!(pattern.start(Width::Narrow), &comma);
        assert_eq!(pattern.middle(Width::Wide), &period);
        assert_eq!(pattern.middle(Width::Short), &period);
        assert_eq!(pattern.middle(Width::Narrow), &period);
        assert_eq!(pattern.end(Width::Wide), &semicolon);
        assert_eq!(pattern.end(Width::Short), &semicolon);
        assert_eq!(pattern.end(Width::Narrow), &semicolon);
        assert_eq!(pattern.pair(Width::Wide), &colon);
        assert_eq!(pattern.pair(Width::Short), &colon);
        assert_eq!(pattern.pair(Width::Narrow), &colon);
    }
}
