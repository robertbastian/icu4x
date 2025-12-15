// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::pattern::PatternLoadError;
use displaydoc::Display;
use icu_calendar::preferences::CalendarAlgorithm;
use icu_provider::DataError;

/// An error from constructing a formatter.
#[derive(Display, Debug, Copy, Clone, PartialEq)]
#[non_exhaustive]
pub enum DateTimeFormatterLoadError {
    /// An error while loading display names for a field.
    #[displaydoc("{0}")]
    Names(PatternLoadError),
    /// An error while loading some other required data,
    /// such as skeleton patterns or calendar conversions.
    #[displaydoc("{0}")]
    Data(DataError),
}

impl core::error::Error for DateTimeFormatterLoadError {}

impl From<DataError> for DateTimeFormatterLoadError {
    fn from(error: DataError) -> Self {
        Self::Data(error)
    }
}

/// The specific field for which an error occurred.
#[derive(Display, Debug, Copy, Clone, PartialEq)]
pub struct ErrorField(pub(crate) crate::provider::fields::Field);

/// An error from mixing calendar types in a formatter.
#[derive(Display, Debug, Copy, Clone, PartialEq)]
#[displaydoc(
    "DateTimeFormatter for {this_algorithm:?} calendar was given a {date_algorithm:?} calendar"
)]
#[non_exhaustive]
pub struct MismatchedCalendarError {
    /// The calendar algorithm of the target object (formatter).
    pub this_algorithm: CalendarAlgorithm,
    /// The calendar algorithm of the input object (date being formatted).
    /// Can be `None` if the input calendar was not specified.
    pub date_algorithm: Option<CalendarAlgorithm>,
}

impl core::error::Error for MismatchedCalendarError {}
