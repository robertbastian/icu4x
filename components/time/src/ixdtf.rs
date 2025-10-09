// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::{
    zone::{iana::IanaParserBorrowed, models, InvalidOffsetError, UtcOffset},
    DateTime, Time, TimeZone, TimeZoneInfo, ZonedDateTime,
};
use core::str::FromStr;
use icu_calendar::{AnyCalendarKind, AsCalendar, Date, DateError, Iso, RangeError};
use ixdtf::{
    encoding::Utf8,
    parsers::IxdtfParser,
    records::{
        IxdtfParseRecord, TimeRecord, TimeZoneAnnotation, TimeZoneRecord, UtcOffsetRecord,
        UtcOffsetRecordOrZ,
    },
    ParseError as Rfc9557ParseError,
};

/// The error type for parsing RFC 9557 strings.
#[derive(Debug, PartialEq, displaydoc::Display)]
#[non_exhaustive]
pub enum ParseError {
    /// Syntax error.
    #[displaydoc("Syntax error in the RFC 9557 string: {0}")]
    Syntax(Rfc9557ParseError),
    /// Parsed record is out of valid date range.
    #[displaydoc("Value out of range: {0}")]
    Range(RangeError),
    /// Parsed date and time records were not a valid ISO date.
    #[displaydoc("Parsed date and time records were not a valid ISO date: {0}")]
    Date(DateError),
    /// There were missing fields required to parse component.
    MissingFields,
    /// There were two offsets provided that were not consistent with each other.
    InconsistentTimeUtcOffsets,
    /// There was an invalid Offset.
    InvalidOffsetError,
    /// Parsed fractional digits had excessive precision beyond nanosecond.
    ExcessivePrecision,
    /// The set of time zone fields was not expected for the given type.
    /// For example, if a named time zone was present with offset-only parsing,
    /// or an offset was present with named-time-zone-only parsing.
    #[displaydoc("The set of time zone fields was not expected for the given type")]
    MismatchedTimeZoneFields,
    /// An unknown calendar was provided.
    UnknownCalendar,
    /// Expected a different calendar.
    #[displaydoc("Expected calendar {0} but found calendar {1}")]
    MismatchedCalendar(AnyCalendarKind, AnyCalendarKind),
    /// A timezone calculation is required to interpret this string, which is not supported.
    ///
    /// # Example
    /// ```
    /// use icu::calendar::Iso;
    /// use icu::time::{ZonedDateTime, ParseError, zone::IanaParser};
    ///
    /// // This timestamp is in UTC, and requires a time zone calculation in order to display a Zurich time.
    /// assert_eq!(
    ///     ZonedDateTime::try_lenient_from_str("2024-08-12T12:32:00Z[Europe/Zurich]", Iso, IanaParser::new()).unwrap_err(),
    ///     ParseError::RequiresCalculation,
    /// );
    ///
    /// // These timestamps are in local time
    /// ZonedDateTime::try_lenient_from_str("2024-08-12T14:32:00+02:00[Europe/Zurich]", Iso, IanaParser::new()).unwrap();
    /// ZonedDateTime::try_lenient_from_str("2024-08-12T14:32:00[Europe/Zurich]", Iso, IanaParser::new()).unwrap();
    /// ```
    #[displaydoc(
        "A timezone calculation is required to interpret this string, which is not supported"
    )]
    RequiresCalculation,
}

impl core::error::Error for ParseError {}

impl From<Rfc9557ParseError> for ParseError {
    fn from(value: Rfc9557ParseError) -> Self {
        Self::Syntax(value)
    }
}

impl From<RangeError> for ParseError {
    fn from(value: RangeError) -> Self {
        Self::Range(value)
    }
}

impl From<DateError> for ParseError {
    fn from(value: DateError) -> Self {
        Self::Date(value)
    }
}

impl From<InvalidOffsetError> for ParseError {
    fn from(_: InvalidOffsetError) -> Self {
        Self::InvalidOffsetError
    }
}

impl From<icu_calendar::ParseError> for ParseError {
    fn from(value: icu_calendar::ParseError) -> Self {
        match value {
            icu_calendar::ParseError::MissingFields => Self::MissingFields,
            icu_calendar::ParseError::Range(r) => Self::Range(r),
            icu_calendar::ParseError::Syntax(s) => Self::Syntax(s),
            icu_calendar::ParseError::UnknownCalendar => Self::UnknownCalendar,
            _ => unreachable!(),
        }
    }
}

impl UtcOffset {
    fn try_from_utc_offset_record(record: &UtcOffsetRecord) -> Result<Self, ParseError> {
        let hour_seconds = i32::from(record.hour()) * 3600;
        let minute_seconds = i32::from(record.minute()) * 60;
        Self::try_from_seconds(
            i32::from(record.sign() as i8)
                * (hour_seconds + minute_seconds + i32::from(record.second().unwrap_or(0))),
        )
        .map_err(Into::into)
    }

    fn try_from_utc_offset_record_or_z(record: &UtcOffsetRecordOrZ) -> Result<Self, ParseError> {
        let UtcOffsetRecordOrZ::Offset(record) = record else {
            return Ok(Self::zero());
        };
        Self::try_from_utc_offset_record(record)
    }

    fn try_from_annotation(
        record: &Option<TimeZoneAnnotation<Utf8>>,
    ) -> Result<Option<UtcOffset>, ParseError> {
        Ok(match record {
            &Some(TimeZoneAnnotation {
                tz: TimeZoneRecord::Offset(offset),
                ..
            }) => Some(UtcOffset::try_from_utc_offset_record(
                &UtcOffsetRecord::MinutePrecision(offset),
            )?),
            _ => None,
        })
    }
}

impl<A: AsCalendar> DateTime<A> {
    fn with_zone(
        self,
        zone: TimeZoneInfo<models::Base>,
    ) -> ZonedDateTime<A, TimeZoneInfo<models::AtTime>> {
        ZonedDateTime {
            zone: zone.at_date_time_iso(DateTime {
                date: self.date.to_iso(),
                time: self.time,
            }),
            date: self.date,
            time: self.time,
        }
    }
}

impl<A: AsCalendar> ZonedDateTime<A, UtcOffset> {
    /// Create a [`ZonedDateTime`] in any calendar from an RFC 9557 string.
    ///
    /// Returns an error if the string has a calendar annotation that does not
    /// match the calendar argument, unless the argument is [`Iso`].
    ///
    /// This function is "strict": the string should have only an offset and no named time zone.
    pub fn try_offset_only_from_str(rfc_9557_str: &str, calendar: A) -> Result<Self, ParseError> {
        Self::try_offset_only_from_utf8(rfc_9557_str.as_bytes(), calendar)
    }

    /// Create a [`ZonedDateTime`] in any calendar from RFC 9557 syntax UTF-8 bytes.
    ///
    /// See [`Self:try_offset_only_from_str`](Self::try_offset_only_from_str).
    pub fn try_offset_only_from_utf8(rfc_9557_str: &[u8], calendar: A) -> Result<Self, ParseError> {
        let ixdtf_record = IxdtfParser::from_utf8(rfc_9557_str).parse()?;
        let date = Date::try_from_ixdtf_record(&ixdtf_record, calendar)?;
        let time = Time::try_from_ixdtf_record(&ixdtf_record)?;

        let timestamp_offset = ixdtf_record
            .offset
            .as_ref()
            .map(UtcOffset::try_from_utc_offset_record_or_z)
            .transpose()?;

        let offset = UtcOffset::try_from_annotation(&ixdtf_record.tz)?;

        match (timestamp_offset, offset) {
            (None, None) => Err(ParseError::MismatchedTimeZoneFields),
            (Some(zone), None) | (None, Some(zone)) => Ok(ZonedDateTime { date, time, zone }),
            (Some(a), Some(b)) if a == b => Ok(ZonedDateTime {
                date,
                time,
                zone: a,
            }),
            // TODO: We could do this calculation...
            _ if matches!(ixdtf_record.offset, Some(UtcOffsetRecordOrZ::Z)) => Err(ParseError::RequiresCalculation),
            _ => Err(ParseError::InconsistentTimeUtcOffsets),
        }
    }
}

impl IanaParserBorrowed<'_> {
    fn parse_annotation(
        &self,
        record: &Option<TimeZoneAnnotation<Utf8>>,
    ) -> Result<Option<TimeZone>, ParseError> {
        Ok(match record {
            Some(TimeZoneAnnotation {
                tz: TimeZoneRecord::Name(iana_identifier),
                ..
            }) => Some(self.parse_from_utf8(iana_identifier)),
            _ => None,
        })
    }
}

impl<A: AsCalendar> ZonedDateTime<A, TimeZoneInfo<models::AtTime>> {
    /// Create a [`ZonedDateTime`] in any calendar from an RFC 9557 string.
    ///
    /// Returns an error if the string has a calendar annotation that does not
    /// match the calendar argument, unless the argument is [`Iso`].
    ///
    /// This function is "strict": the string should have only a named time zone and no offset.
    pub fn try_location_only_from_str(
        rfc_9557_str: &str,
        calendar: A,
        iana_parser: IanaParserBorrowed,
    ) -> Result<Self, ParseError> {
        Self::try_location_only_from_utf8(rfc_9557_str.as_bytes(), calendar, iana_parser)
    }

    /// Create a [`ZonedDateTime`] in any calendar from RFC 9557 UTF-8 bytes.
    ///
    /// See [`Self::try_location_only_from_str`].
    pub fn try_location_only_from_utf8(
        rfc_9557_str: &[u8],
        calendar: A,
        iana_parser: IanaParserBorrowed,
    ) -> Result<Self, ParseError> {
        let ixdtf_record = IxdtfParser::from_utf8(rfc_9557_str).parse()?;
        let date = Date::try_from_ixdtf_record(&ixdtf_record, calendar)?;
        let time = Time::try_from_ixdtf_record(&ixdtf_record)?;

        let id = iana_parser
            .parse_annotation(&ixdtf_record.tz)?
            .ok_or(ParseError::MismatchedTimeZoneFields)?;

        match (ixdtf_record.offset.as_ref(), id.with_offset(None).offset()) {
            // timestamp is local
            (None, _) => Ok(DateTime { date, time }.with_zone(id.with_offset(None))),
            // timestamp is Z, but annotation doesn't have an offset
            (Some(UtcOffsetRecordOrZ::Z), None) => Err(ParseError::RequiresCalculation),
            // timestamp has an offset, treat as local offset
            (Some(UtcOffsetRecordOrZ::Offset(r)), None) => Ok(DateTime { date, time }
                .with_zone(id.with_offset(Some(UtcOffset::try_from_utc_offset_record(r)?)))),
            // annotation has an offset
            (Some(a), Some(b)) if UtcOffset::try_from_utc_offset_record_or_z(a)? == b => {
                Ok(DateTime { date, time }.with_zone(id.with_offset(Some(b))))
            }
            _ => Err(ParseError::InconsistentTimeUtcOffsets),
        }
    }

    /// Create a [`ZonedDateTime`] in any calendar from an RFC 9557 string.
    ///
    /// Returns an error if the string has a calendar annotation that does not
    /// match the calendar argument, unless the argument is [`Iso`].
    ///
    /// This function is "lenient": the string can have an offset, and named time zone, both, or
    /// neither. If the named time zone is missing, it is returned as Etc/Unknown.
    pub fn try_lenient_from_str(
        rfc_9557_str: &str,
        calendar: A,
        iana_parser: IanaParserBorrowed,
    ) -> Result<Self, ParseError> {
        Self::try_lenient_from_utf8(rfc_9557_str.as_bytes(), calendar, iana_parser)
    }

    /// Create a [`ZonedDateTime`] in any calendar from RFC 9557 UTF-8 bytes.
    ///
    /// See [`Self::try_lenient_from_str`].
    pub fn try_lenient_from_utf8(
        rfc_9557_str: &[u8],
        calendar: A,
        iana_parser: IanaParserBorrowed,
    ) -> Result<Self, ParseError> {
        let ixdtf_record = IxdtfParser::from_utf8(rfc_9557_str).parse()?;
        let date = Date::try_from_ixdtf_record(&ixdtf_record, calendar)?;
        let time = Time::try_from_ixdtf_record(&ixdtf_record)?;

        let id = iana_parser
            .parse_annotation(&ixdtf_record.tz)?
            .unwrap_or(TimeZone::UNKNOWN);
        let offset = UtcOffset::try_from_annotation(&ixdtf_record.tz)?;

        match (
            ixdtf_record.offset.as_ref(),
            id.without_offset().offset().or(offset),
        ) {
            // timestamp is local
            (None, _) => Ok(DateTime { date, time }.with_zone(id.with_offset(offset))),
            // timestamp is Z, but there is an annotation without an offset
            (Some(UtcOffsetRecordOrZ::Z), None)
                if id != TimeZone::UNKNOWN && id != TimeZoneInfo::utc().id() =>
            {
                Err(ParseError::RequiresCalculation)
            }
            // timestamp is Z, but there is no annotation
            (Some(UtcOffsetRecordOrZ::Z), None) => {
                Ok(DateTime { date, time }.with_zone(TimeZoneInfo::utc()))
            }
            // timestamp is an offset, treat as local offset
            (Some(UtcOffsetRecordOrZ::Offset(r)), None) => Ok(DateTime { date, time }
                .with_zone(id.with_offset(Some(UtcOffset::try_from_utc_offset_record(r)?)))),
            // timestamp has an offset, and annotation has an offset
            (Some(a), Some(b)) if UtcOffset::try_from_utc_offset_record_or_z(a)? == b => {
                Ok(DateTime { date, time }.with_zone(id.with_offset(Some(b))))
            }
            _ => Err(ParseError::InconsistentTimeUtcOffsets),
        }
    }

    /// Create a [`ZonedDateTime`] in any calendar from an RFC 9557 string.
    ///
    /// Returns an error if the string has a calendar annotation that does not
    /// match the calendar argument, unless the argument is [`Iso`].
    ///
    /// The string should have both an offset and a named time zone.
    ///
    /// For more information on RFC 9557, see the [`ixdtf`] crate.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use icu::calendar::cal::Hebrew;
    /// use icu::locale::subtags::subtag;
    /// use icu::time::{
    ///     zone::{IanaParser, TimeZoneVariant, UtcOffset},
    ///     TimeZone, TimeZoneInfo, ZonedDateTime,
    /// };
    ///
    /// let zoneddatetime = ZonedDateTime::try_strict_from_str(
    ///     "2024-08-08T12:08:19-05:00[America/Chicago][u-ca=hebrew]",
    ///     Hebrew,
    ///     IanaParser::new(),
    /// )
    /// .unwrap();
    ///
    /// assert_eq!(zoneddatetime.date.extended_year(), 5784);
    /// assert_eq!(
    ///     zoneddatetime.date.month().standard_code,
    ///     icu::calendar::types::MonthCode(tinystr::tinystr!(4, "M11"))
    /// );
    /// assert_eq!(zoneddatetime.date.day_of_month().0, 4);
    ///
    /// assert_eq!(zoneddatetime.time.hour.number(), 12);
    /// assert_eq!(zoneddatetime.time.minute.number(), 8);
    /// assert_eq!(zoneddatetime.time.second.number(), 19);
    /// assert_eq!(zoneddatetime.time.subsecond.number(), 0);
    /// assert_eq!(zoneddatetime.zone.id(), TimeZone(subtag!("uschi")));
    /// assert_eq!(
    ///     zoneddatetime.zone.offset(),
    ///     Some(UtcOffset::try_from_seconds(-18000).unwrap())
    /// );
    /// let _ = zoneddatetime.zone.zone_name_timestamp();
    /// ```
    ///
    /// An RFC 9557 string can provide a time zone in two parts: the DateTime UTC Offset or the Time Zone
    /// Annotation. A DateTime UTC Offset is the time offset as laid out by RFC 3339; meanwhile, the Time
    /// Zone Annotation is the annotation laid out by RFC 9557 and is defined as a UTC offset or IANA Time
    /// Zone identifier.
    ///
    /// ## DateTime UTC Offsets
    ///
    /// Below is an example of a time zone from a DateTime UTC Offset. The syntax here is familiar to a RFC 3339
    /// DateTime string.
    ///
    /// ```
    /// use icu::calendar::Iso;
    /// use icu::time::{zone::UtcOffset, TimeZoneInfo, ZonedDateTime};
    ///
    /// let tz_from_offset = ZonedDateTime::try_offset_only_from_str(
    ///     "2024-08-08T12:08:19-05:00",
    ///     Iso,
    /// )
    /// .unwrap();
    ///
    /// assert_eq!(
    ///     tz_from_offset.zone,
    ///     UtcOffset::try_from_seconds(-18000).unwrap()
    /// );
    /// ```
    ///
    /// ## Time Zone Annotations
    ///
    /// Below is an example of a time zone being provided by a time zone annotation.
    ///
    /// ```
    /// use icu::calendar::Iso;
    /// use icu::locale::subtags::subtag;
    /// use icu::time::{
    ///     zone::{IanaParser, TimeZoneVariant, UtcOffset},
    ///     TimeZone, TimeZoneInfo, ZonedDateTime,
    /// };
    ///
    /// let tz_from_offset_annotation = ZonedDateTime::try_offset_only_from_str(
    ///     "2024-08-08T12:08:19[-05:00]",
    ///     Iso,
    /// )
    /// .unwrap();
    /// let tz_from_iana_annotation = ZonedDateTime::try_location_only_from_str(
    ///     "2024-08-08T12:08:19[America/Chicago]",
    ///     Iso,
    ///     IanaParser::new(),
    /// )
    /// .unwrap();
    ///
    /// assert_eq!(
    ///     tz_from_offset_annotation.zone,
    ///     UtcOffset::try_from_seconds(-18000).unwrap()
    /// );
    ///
    /// assert_eq!(
    ///     tz_from_iana_annotation.zone.id(),
    ///     TimeZone(subtag!("uschi"))
    /// );
    /// assert_eq!(tz_from_iana_annotation.zone.offset(), None);
    /// ```
    ///
    /// ## UTC Offset and time zone annotations.
    ///
    /// An RFC 9557 string may contain both a UTC Offset and time zone annotation. This is fine as long as
    /// the time zone parts can be deemed as inconsistent or unknown consistency.
    ///
    /// ### DateTime UTC offset with UTC Offset annotation.
    ///
    /// If these annotations don't match, the timestamp is converted into the offset of the annotation.
    ///
    /// ```
    /// use icu::calendar::Iso;
    /// use icu::time::{
    ///     zone::UtcOffset, ParseError, TimeZone, TimeZoneInfo, ZonedDateTime,
    /// };
    /// use tinystr::tinystr;
    ///
    /// let consistent_offsets_from_both = ZonedDateTime::try_offset_only_from_str(
    ///     "2024-08-08T12:08:19-05:00[-05:00]",
    ///     Iso,
    /// )
    /// .unwrap();
    ///
    /// assert_eq!(
    ///     consistent_offsets_from_both.zone,
    ///     UtcOffset::try_from_seconds(-18000).unwrap()
    /// );
    ///
    /// let inconsistent_offsets_from_both = ZonedDateTime::try_offset_only_from_str(
    ///     "2024-08-08T12:08:19-05:00[+05:00]",
    ///     Iso,
    /// ).unwrap();
    ///
    /// assert_eq!(inconsistent_offsets_from_both.time.hour.number(), 22);
    /// ```
    pub fn try_strict_from_str(
        rfc_9557_str: &str,
        calendar: A,
        iana_parser: IanaParserBorrowed,
    ) -> Result<Self, ParseError> {
        Self::try_strict_from_utf8(rfc_9557_str.as_bytes(), calendar, iana_parser)
    }

    /// Create a [`ZonedDateTime`] in any calendar from RFC 9557 UTF-8 bytes.
    ///
    /// See [`Self::try_strict_from_str`].
    pub fn try_strict_from_utf8(
        rfc_9557_str: &[u8],
        calendar: A,
        iana_parser: IanaParserBorrowed,
    ) -> Result<Self, ParseError> {
        let ixdtf_record = IxdtfParser::from_utf8(rfc_9557_str).parse()?;
        let date = Date::try_from_ixdtf_record(&ixdtf_record, calendar)?;
        let time = Time::try_from_ixdtf_record(&ixdtf_record)?;

        let timestamp_offset = UtcOffset::try_from_utc_offset_record_or_z(
            ixdtf_record
                .offset
                .as_ref()
                .ok_or(ParseError::MismatchedTimeZoneFields)?,
        )?;

        let id = iana_parser
            .parse_annotation(&ixdtf_record.tz)?
            .ok_or(ParseError::MismatchedTimeZoneFields)?;

        if let Some(id_offset) = id.without_offset().offset() {
            if id_offset != timestamp_offset {
                return Err(ParseError::InconsistentTimeUtcOffsets);
            }
            Ok(DateTime { date, time }.with_zone(id.without_offset()))
        } else if ixdtf_record.offset == Some(UtcOffsetRecordOrZ::Z) {
            // the timestamp is not in local time
            Err(ParseError::RequiresCalculation)
        } else {
            // use the timestamp offset as local time
            Ok(DateTime { date, time }.with_zone(id.with_offset(Some(timestamp_offset))))
        }
    }
}

#[allow(deprecated)]
impl<A: AsCalendar> ZonedDateTime<A, TimeZoneInfo<models::Full>> {
    /// Create a [`ZonedDateTime`] in any calendar from an RFC 9557 string.
    #[deprecated(since = "2.1.0", note = "use `try_strict_from_str`")]
    pub fn try_full_from_str(
        rfc_9557_str: &str,
        calendar: A,
        iana_parser: IanaParserBorrowed,
        offset_calculator: crate::zone::VariantOffsetsCalculatorBorrowed,
    ) -> Result<Self, ParseError> {
        Self::try_full_from_utf8(
            rfc_9557_str.as_bytes(),
            calendar,
            iana_parser,
            offset_calculator,
        )
    }

    /// Create a [`ZonedDateTime`] in any calendar from RFC 9557 UTF-8 bytes.
    ///
    /// See [`Self::try_full_from_str`].
    #[deprecated(since = "2.1.0", note = "use `try_strict_from_utf8`")]
    pub fn try_full_from_utf8(
        rfc_9557_str: &[u8],
        calendar: A,
        iana_parser: IanaParserBorrowed,
        offset_calculator: crate::zone::VariantOffsetsCalculatorBorrowed,
    ) -> Result<Self, ParseError> {
        let ZonedDateTime { date, time, zone } =
            ZonedDateTime::try_strict_from_utf8(rfc_9557_str, calendar, iana_parser)?;

        Ok(ZonedDateTime {
            date,
            time,
            zone: zone.infer_variant(offset_calculator),
        })
    }
}

impl FromStr for DateTime<Iso> {
    type Err = ParseError;
    fn from_str(rfc_9557_str: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(rfc_9557_str, Iso)
    }
}

impl<A: AsCalendar> DateTime<A> {
    /// Creates a [`DateTime`] in any calendar from an RFC 9557 string.
    ///
    /// Returns an error if the string has a calendar annotation that does not
    /// match the calendar argument, unless the argument is [`Iso`].
    ///
    /// ✨ *Enabled with the `ixdtf` Cargo feature.*
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::calendar::cal::Hebrew;
    /// use icu::time::DateTime;
    ///
    /// let datetime =
    ///     DateTime::try_from_str("2024-07-17T16:01:17.045[u-ca=hebrew]", Hebrew)
    ///         .unwrap();
    ///
    /// assert_eq!(datetime.date.era_year().year, 5784);
    /// assert_eq!(
    ///     datetime.date.month().standard_code,
    ///     icu::calendar::types::MonthCode(tinystr::tinystr!(4, "M10"))
    /// );
    /// assert_eq!(datetime.date.day_of_month().0, 11);
    ///
    /// assert_eq!(datetime.time.hour.number(), 16);
    /// assert_eq!(datetime.time.minute.number(), 1);
    /// assert_eq!(datetime.time.second.number(), 17);
    /// assert_eq!(datetime.time.subsecond.number(), 45000000);
    /// ```
    pub fn try_from_str(rfc_9557_str: &str, calendar: A) -> Result<Self, ParseError> {
        Self::try_from_utf8(rfc_9557_str.as_bytes(), calendar)
    }

    /// Creates a [`DateTime`] in any calendar from an RFC 9557 string.
    ///
    /// See [`Self::try_from_str()`].
    ///
    /// ✨ *Enabled with the `ixdtf` Cargo feature.*
    pub fn try_from_utf8(rfc_9557_str: &[u8], calendar: A) -> Result<Self, ParseError> {
        let ixdtf_record = IxdtfParser::from_utf8(rfc_9557_str).parse()?;
        let date = Date::try_from_ixdtf_record(&ixdtf_record, calendar)?;
        let time = Time::try_from_ixdtf_record(&ixdtf_record)?;
        Ok(Self { date, time })
    }
}

impl Time {
    /// Creates a [`Time`] from an RFC 9557 string of a time.
    ///
    /// Does not support parsing an RFC 9557 string with a date and time; for that, use [`DateTime`].
    ///
    /// ✨ *Enabled with the `ixdtf` Cargo feature.*
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::time::Time;
    ///
    /// let time = Time::try_from_str("16:01:17.045").unwrap();
    ///
    /// assert_eq!(time.hour.number(), 16);
    /// assert_eq!(time.minute.number(), 1);
    /// assert_eq!(time.second.number(), 17);
    /// assert_eq!(time.subsecond.number(), 45000000);
    /// ```
    pub fn try_from_str(rfc_9557_str: &str) -> Result<Self, ParseError> {
        Self::try_from_utf8(rfc_9557_str.as_bytes())
    }

    /// Creates a [`Time`] in the ISO-8601 calendar from an RFC 9557 string.
    ///
    /// ✨ *Enabled with the `ixdtf` Cargo feature.*
    ///
    /// See [`Self::try_from_str()`].
    pub fn try_from_utf8(rfc_9557_str: &[u8]) -> Result<Self, ParseError> {
        let ixdtf_record = IxdtfParser::from_utf8(rfc_9557_str).parse_time()?;
        Self::try_from_ixdtf_record(&ixdtf_record)
    }

    fn try_from_ixdtf_record(
        ixdtf_record: &IxdtfParseRecord<'_, Utf8>,
    ) -> Result<Self, ParseError> {
        let time_record = ixdtf_record.time.ok_or(ParseError::MissingFields)?;
        Self::try_from_time_record(&time_record)
    }

    fn try_from_time_record(time_record: &TimeRecord) -> Result<Self, ParseError> {
        let nanosecond = time_record
            .fraction
            .map(|fraction| {
                fraction
                    .to_nanoseconds()
                    .ok_or(ParseError::ExcessivePrecision)
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self::try_new(
            time_record.hour,
            time_record.minute,
            time_record.second,
            nanosecond,
        )?)
    }
}

impl FromStr for Time {
    type Err = ParseError;
    fn from_str(rfc_9557_str: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(rfc_9557_str)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::TimeZone;

    #[test]
    fn max_possible_rfc_9557_utc_offset() {
        assert_eq!(
            ZonedDateTime::try_offset_only_from_str("2024-08-08T12:08:19+23:59:59.999999999", Iso)
                .unwrap_err(),
            ParseError::InvalidOffsetError
        );
    }

    #[test]
    fn zone_calculations() {
        ZonedDateTime::try_offset_only_from_str("2024-08-08T12:08:19Z", Iso).unwrap();
        assert_eq!(
            ZonedDateTime::try_offset_only_from_str("2024-08-08T12:08:19Z[+08:00]", Iso)
                .unwrap_err(),
            ParseError::RequiresCalculation
        );
        ZonedDateTime::try_offset_only_from_str("2024-08-08T12:08:19+08:00[Etc/GMT-8]", Iso)
            .unwrap();
        assert_eq!(
            ZonedDateTime::try_offset_only_from_str("2024-08-08T12:08:19Z[Europe/Zurich]", Iso)
                .unwrap_err(),
            ParseError::MismatchedTimeZoneFields
        );
    }

    #[test]
    fn future_zone() {
        let result = ZonedDateTime::try_location_only_from_str(
            "2024-08-08T12:08:19[Future/Zone]",
            Iso,
            IanaParserBorrowed::new(),
        )
        .unwrap();
        assert_eq!(result.zone.id(), TimeZone::UNKNOWN);
        assert_eq!(result.zone.offset(), None);
    }

    #[test]
    fn lax() {
        ZonedDateTime::try_location_only_from_str(
            "2024-10-18T15:44[America/Los_Angeles]",
            icu_calendar::cal::Gregorian,
            IanaParserBorrowed::new(),
        )
        .unwrap();
    }
}
