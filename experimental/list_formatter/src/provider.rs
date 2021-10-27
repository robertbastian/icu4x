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
    pub const LIST_FORMAT_AND_V1: ResourceKey = resource_key!(ListFormatter, "list/and", 1);
    pub const LIST_FORMAT_OR_V1: ResourceKey = resource_key!(ListFormatter, "list/or", 1);
    pub const LIST_FORMAT_UNIT_V1: ResourceKey = resource_key!(ListFormatter, "list/unit", 1);
}

/// Symbols and metadata required for [`ListFormatter`](crate::ListFormatter). 
/// Absent values follow this fallback structure:
/// ", " - start - middle
///            |-- end - pair
///            |     \ short_end - short_pair
///            |               \ narrow_end - narrow_pair
///             \ short_start - short_middle
///                         \ narrow_start <- narrow_middle
#[icu_provider::data_struct]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ListFormatterPatternsV1<'data> {
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    start: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    middle: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    end: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    pair: Option<ConditionalListJoinerPattern<'data>>,

    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    short_start: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    short_middle: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    short_end: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    short_pair: Option<ConditionalListJoinerPattern<'data>>,

    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    narrow_start: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    narrow_middle: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    narrow_end: Option<ConditionalListJoinerPattern<'data>>,
    #[cfg_attr(feature = "provider_serde", serde(borrow))]
    narrow_pair: Option<ConditionalListJoinerPattern<'data>>,
}

#[rustfmt::skip] // This is more readable without excessive line breaks
impl<'data> ListFormatterPatternsV1<'data> {
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
        Self {
            narrow_pair: if narrow_pair == narrow_end { None } else { Some(narrow_pair) },
            narrow_end: if narrow_end == short_end { None } else { Some(narrow_end) },
            narrow_middle: if narrow_middle == narrow_start { None  } else { Some(narrow_middle) },
            narrow_start: if narrow_start == short_start { None  } else { Some(narrow_start) },

            short_pair: if short_pair == short_end { None } else { Some(short_pair) },
            short_end: if short_end == end { None } else { Some(short_end) },
            short_middle: if short_middle == short_start { None  } else { Some(short_middle) },
            short_start: if short_start == start { None } else { Some(short_start) },

            pair: if pair == end { None } else { Some(pair) },
            end: if end == start { None } else { Some(end) },
            middle: if middle == start { None } else { Some(middle) },
            start: if start == *LATIN_COMMA { None } else { Some(start)},
        }
    }

    pub fn start(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        match width {
            Width::Wide => self.start.as_ref(),
            Width::Short => self.short_start.as_ref().or(self.start.as_ref()),
            Width::Narrow => self.narrow_start.as_ref().or(self.short_start.as_ref()).or(self.start.as_ref()),
        }.unwrap_or(LATIN_COMMA)
    }

    pub fn middle(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        match width {
            Width::Wide => 
                self.middle.as_ref(),
            Width::Short => 
                self.short_middle.as_ref().or(self.short_start.as_ref()),
            Width::Narrow => 
                self.narrow_middle.as_ref().or(self.narrow_start.as_ref()).or(self.short_start.as_ref()),
        }.or(self.start.as_ref()).unwrap_or(LATIN_COMMA)
    }

    pub fn end(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        match width {
            Width::Wide => 
                self.end.as_ref(),
            Width::Short => 
                self.short_end.as_ref().or(self.end.as_ref()),
            Width::Narrow => 
                self.narrow_end.as_ref().or(self.short_end.as_ref()).or(self.end.as_ref()),
        }.or(self.start.as_ref()).unwrap_or(LATIN_COMMA)
    }

    pub fn pair(&self, width: Width) -> &ConditionalListJoinerPattern<'data> {
        match width {
            Width::Wide => 
                self.pair.as_ref(),
            Width::Short => 
                self.short_pair.as_ref().or(self.short_end.as_ref()),
            Width::Narrow => 
                self.narrow_pair.as_ref().or(self.narrow_end.as_ref()).or(self.short_end.as_ref()),
        }.or(self.end.as_ref()).or(self.start.as_ref()).unwrap_or(LATIN_COMMA)
    }
}

const LATIN_COMMA: &'static ConditionalListJoinerPattern = &ConditionalListJoinerPattern {
    default: ListJoinerPattern {
        string: Cow::Borrowed(", "),
        insertion_points: 0x0_2, // [0, 2]
    },
    special_case: None,
};

/// A pattern that can behave conditionally on the next element.
#[derive(Debug, PartialEq, Eq, Clone, Yokeable, ZeroCopyFrom)]
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
#[derive(Debug, PartialEq, Eq, Clone, Yokeable, ZeroCopyFrom)]
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
#[derive(Debug, PartialEq, Eq, Clone, Yokeable, ZeroCopyFrom)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
struct ListJoinerPattern<'data> {
    /// The pattern string without the placeholders
    string: Cow<'data, str>,
    /// The indices of the two placeholders, using 4 bytes each
    insertion_points: u8,
}

pub type PatternParts<'a> = (&'a str, &'a str, &'a str);

impl<'data> ListJoinerPattern<'data> {
    fn borrow_tuple(&'data self) -> PatternParts<'data> {
        (
            &self.string[0..(self.insertion_points >> 4) as usize],
            &self.string
                [(self.insertion_points >> 4) as usize..(self.insertion_points & 0xF) as usize],
            &self.string[(self.insertion_points & 0xF) as usize..],
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
                (Some(index_0), Some(index_1)) if index_0 + 3 <= index_1 => Ok(ListJoinerPattern {
                    string: Cow::Owned(
                        pattern[0..index_0].to_string()
                            + &pattern[index_0 + 3..index_1]
                            + &pattern[index_1 + 3..],
                    ),
                    insertion_points: ((index_0 << 4) | (index_1 - 3)) as u8,
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
        let pattern = ConditionalListJoinerPattern::from_str("a{0}b{1}c").unwrap();
        assert_eq!(pattern.parts(""), ("a", "b", "c"));
    }

    #[test]
    fn produces_correct_parts_conditionally() {
        let pattern =
            ConditionalListJoinerPattern::from_regex_and_strs("b.*", "c{0}d{1}e", "a{0}b{1}c")
                .unwrap();
        assert_eq!(pattern.parts("a"), ("a", "b", "c"));
        assert_eq!(pattern.parts("b"), ("c", "d", "e"));
    }

    #[test]
    fn fallbacks_work() {
        let comma = LATIN_COMMA.clone();
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
