// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::calendar_arithmetic::{ArithmeticDate, DateFieldsResolver, PackWithMD, ToExtendedYear};
use crate::error::{
    DateError, DateFromFieldsError, EcmaReferenceYearError, MonthCodeError, UnknownEraError,
};
use crate::options::{DateAddOptions, DateDifferenceOptions};
use crate::options::{DateFromFieldsOptions, Overflow};
use crate::types::{DateFields, LeapStatus, Month, MonthInfo};
use crate::RangeError;
use crate::{types, Calendar, Date};
use ::tinystr::tinystr;
use calendrical_calculations::hebrew_keviyah::{Keviyah, YearInfo};
use calendrical_calculations::rata_die::RataDie;

/// The [Hebrew Calendar](https://en.wikipedia.org/wiki/Hebrew_calendar)
///
/// The Hebrew calendar is a lunisolar calendar used as the Jewish liturgical calendar
/// as well as an official calendar in Israel.
///
/// This implementation uses civil month numbering, where Tishrei is the first month of the year.
///
/// The precise algorithm used to calculate the Hebrew Calendar has [changed over time], with
/// the modern one being in place since about 4536 AM (776 CE). This implementation extends
/// proleptically for dates before that.
///
/// [changed over time]: https://hakirah.org/vol20AjdlerAppendices.pdf
///
/// This corresponds to the `"hebrew"` [CLDR calendar](https://unicode.org/reports/tr35/#UnicodeCalendarIdentifier).
///
/// # Era codes
///
/// This calendar uses a single era code `am`, Anno Mundi. Dates before this era use negative years.
///
/// # Months and days
///
/// The 12 months are called Tishrei (`M01`, 30 days), Ḥešvan (`M02`, 29/30 days),
/// Kīslev (`M03`, 30/29 days), Ṭevet (`M04`, 29 days), Šəvaṭ (`M05`, 30 days), ʾĂdār (`M06`, 29 days),
/// Nīsān (`M07`, 30 days), ʾĪyyar (`M08`, 29 days), Sivan (`M09`, 30 days), Tammūz (`M10`, 29 days),
/// ʾAv (`M11`, 30 days), ʾElūl (`M12`, 29 days).
///
/// Due to Rosh Hashanah postponement rules, Ḥešvan and Kislev vary in length.
///
/// In leap years (years 3, 6, 8, 11, 17, 19 in a 19-year cycle), the leap month Adar I (`M05L`, 30 days)
/// is inserted before Adar, and Adar is called Adar II (the `formatting_code` returned by [`MonthInfo`]
/// will be `M06L` to mark this, while the `standard_code` remains `M06`).
///
/// Standard years thus have 353-355 days, and leap years 383-385.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
#[allow(clippy::exhaustive_structs)] // unit struct
pub struct Hebrew;

/// The inner date type used for representing [`Date`]s of [`Hebrew`]. See [`Date`] and [`Hebrew`] for more details.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct HebrewDateInner(ArithmeticDate<Hebrew>);

impl Hebrew {
    /// Construct a new [`Hebrew`]
    pub fn new() -> Self {
        Hebrew
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct HebrewYear {
    keviyah: Keviyah,
    value: i32,
}

impl PackWithMD for HebrewYear {
    /// The first byte is the [`Keviyah`], the remaining four the YMD as encoded by [`i32::pack`].
    type Packed = [u8; 5];

    fn pack(self, month: u8, day: u8) -> Self::Packed {
        let a = self.keviyah as u8;
        let [b, c, d, e] = self.value.pack(month, day);
        [a, b, c, d, e]
    }

    fn unpack_year([a, b, c, d, e]: Self::Packed) -> Self {
        let value = i32::unpack_year([b, c, d, e]);
        let keviyah = Keviyah::from_integer(a);
        Self { keviyah, value }
    }

    fn unpack_month([_, b, c, d, e]: Self::Packed) -> u8 {
        i32::unpack_month([b, c, d, e])
    }

    fn unpack_day([_, b, c, d, e]: Self::Packed) -> u8 {
        i32::unpack_day([b, c, d, e])
    }
}

impl ToExtendedYear for HebrewYear {
    fn to_extended_year(&self) -> i32 {
        self.value
    }
}

impl HebrewYear {
    /// Convenience method to compute for a given year. Don't use this if you actually need
    /// a YearInfo that you want to call .new_year() on.
    fn compute(value: i32) -> Self {
        Self {
            keviyah: YearInfo::compute_for(value).keviyah,
            value,
        }
    }

    fn for_rd(rd: RataDie) -> Self {
        let (year, value) = YearInfo::year_containing_rd(rd);
        Self {
            keviyah: year.keviyah,
            value,
        }
    }

    fn new_year(self) -> RataDie {
        self.keviyah.year_info(self.value).new_year()
    }
}

impl DateFieldsResolver for Hebrew {
    type YearInfo = HebrewYear;
    fn days_in_provided_month(year: HebrewYear, ordinal_month: u8) -> u8 {
        year.keviyah.month_len(ordinal_month)
    }

    fn months_in_provided_year(year: HebrewYear) -> u8 {
        12 + year.keviyah.is_leap() as u8
    }

    #[inline]
    fn year_info_from_era(
        &self,
        era: &[u8],
        era_year: i32,
    ) -> Result<Self::YearInfo, UnknownEraError> {
        match era {
            b"am" => Ok(HebrewYear::compute(era_year)),
            _ => Err(UnknownEraError),
        }
    }
    #[inline]
    fn year_info_from_extended(&self, extended_year: i32) -> Self::YearInfo {
        HebrewYear::compute(extended_year)
    }

    fn reference_year_from_month_day(
        &self,
        month: types::Month,
        day: u8,
    ) -> Result<Self::YearInfo, EcmaReferenceYearError> {
        // December 31, 1972 occurs on 4th month, 26th day, 5733 AM
        let hebrew_year = match (month.number(), month.is_leap()) {
            (1, false) => 5733,
            (2, false) => match day {
                // There is no day 30 in 5733 (there is in 5732)
                ..=29 => 5733,
                // Note (here and below): this must be > 29, not just == 30,
                // since we have not yet applied a potential Overflow::Constrain.
                _ => 5732,
            },
            (3, false) => match day {
                // There is no day 30 in 5733 (there is in 5732)
                ..=29 => 5733,
                _ => 5732,
            },
            (4, false) => match day {
                ..=26 => 5733,
                _ => 5732,
            },
            (5..=12, false) => 5732,
            // Neither 5731 nor 5732 is a leap year
            (5, true) => 5730,
            _ => {
                return Err(EcmaReferenceYearError::MonthCodeNotInCalendar);
            }
        };
        Ok(HebrewYear::compute(hebrew_year))
    }

    fn ordinal_from_month(
        &self,
        year: Self::YearInfo,
        month: types::Month,
        options: DateFromFieldsOptions,
    ) -> Result<u8, MonthCodeError> {
        let is_leap_year = year.keviyah.is_leap();
        let ordinal_month = match (month.number(), month.is_leap()) {
            (n @ 1..=12, false) => n + (n >= 6 && is_leap_year) as u8,
            (5, true) => {
                if is_leap_year {
                    6
                } else if matches!(options.overflow, Some(Overflow::Constrain)) {
                    // M05L maps to M06 in a common year
                    6
                } else {
                    return Err(MonthCodeError::NotInYear);
                }
            }
            _ => return Err(MonthCodeError::NotInCalendar),
        };
        Ok(ordinal_month)
    }

    fn month_from_ordinal(&self, year: Self::YearInfo, ordinal_month: u8) -> types::Month {
        let is_leap = year.keviyah.is_leap();
        Month::new_unchecked(
            ordinal_month - (is_leap && ordinal_month >= 6) as u8,
            if ordinal_month == 6 && is_leap {
                types::LeapStatus::Leap
            } else if ordinal_month == 7 && is_leap {
                // Use the leap name for Adar in a leap year
                LeapStatus::FormattingLeap
            } else {
                LeapStatus::Normal
            },
        )
    }
}

impl crate::cal::scaffold::UnstableSealed for Hebrew {}
impl Calendar for Hebrew {
    type DateInner = HebrewDateInner;
    type Year = types::EraYear;
    type DifferenceError = core::convert::Infallible;

    fn from_codes(
        &self,
        era: Option<&str>,
        year: i32,
        month_code: types::MonthCode,
        day: u8,
    ) -> Result<Self::DateInner, DateError> {
        ArithmeticDate::from_era_year_month_code_day(era, year, month_code, day, self)
            .map(HebrewDateInner)
    }

    #[cfg(feature = "unstable")]
    fn from_fields(
        &self,
        fields: DateFields,
        options: DateFromFieldsOptions,
    ) -> Result<Self::DateInner, DateFromFieldsError> {
        ArithmeticDate::from_fields(fields, options, self).map(HebrewDateInner)
    }

    fn from_rata_die(&self, rd: RataDie) -> Self::DateInner {
        let year = HebrewYear::for_rd(rd);

        // Clamp the RD to our year
        let rd = rd.clamp(
            year.new_year(),
            year.new_year() + year.keviyah.year_length() as i64,
        );

        let (month, day) = year
            .keviyah
            .month_day_for((rd - year.new_year()) as u16 + 1);

        // date is in the valid RD range
        HebrewDateInner(ArithmeticDate::new_unchecked(year, month, day))
    }

    fn to_rata_die(&self, date: &Self::DateInner) -> RataDie {
        date.0.year().new_year()
            + date.0.year().keviyah.days_preceding(date.0.month()) as i64
            + (date.0.day() - 1) as i64
    }

    fn has_cheap_iso_conversion(&self) -> bool {
        false
    }

    fn months_in_year(&self, date: &Self::DateInner) -> u8 {
        Self::months_in_provided_year(date.0.year())
    }

    fn days_in_year(&self, date: &Self::DateInner) -> u16 {
        date.0.year().keviyah.year_length()
    }

    fn days_in_month(&self, date: &Self::DateInner) -> u8 {
        Self::days_in_provided_month(date.0.year(), date.0.month())
    }

    #[cfg(feature = "unstable")]
    fn add(
        &self,
        date: &Self::DateInner,
        duration: types::DateDuration,
        options: DateAddOptions,
    ) -> Result<Self::DateInner, DateError> {
        date.0.added(duration, self, options).map(HebrewDateInner)
    }

    #[cfg(feature = "unstable")]
    fn until(
        &self,
        date1: &Self::DateInner,
        date2: &Self::DateInner,
        options: DateDifferenceOptions,
    ) -> Result<types::DateDuration, Self::DifferenceError> {
        Ok(date1.0.until(&date2.0, self, options))
    }

    fn debug_name(&self) -> &'static str {
        "Hebrew"
    }

    fn year_info(&self, date: &Self::DateInner) -> Self::Year {
        let extended_year = date.0.year().value;
        types::EraYear {
            era_index: Some(0),
            era: tinystr!(16, "am"),
            year: extended_year,
            extended_year,
            ambiguity: types::YearAmbiguity::CenturyRequired,
        }
    }

    fn is_in_leap_year(&self, date: &Self::DateInner) -> bool {
        date.0.year().keviyah.is_leap()
    }

    fn month(&self, date: &Self::DateInner) -> MonthInfo {
        MonthInfo::new(self, date.0)
    }

    fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
        types::DayOfMonth(date.0.day())
    }

    fn day_of_year(&self, date: &Self::DateInner) -> types::DayOfYear {
        types::DayOfYear(date.0.year().keviyah.days_preceding(date.0.month()) + date.0.day() as u16)
    }

    fn calendar_algorithm(&self) -> Option<crate::preferences::CalendarAlgorithm> {
        Some(crate::preferences::CalendarAlgorithm::Hebrew)
    }
}

impl Date<Hebrew> {
    /// This method uses an ordinal month, which is probably not what you want.
    ///
    /// Years are arithmetic, meaning there is a year 0 preceded by negative years, with a
    /// valid range of `-1,000,000..=1,000,000`.
    ///
    /// Use [`Date::try_new_from_codes`]
    #[deprecated(since = "2.1.0", note = "use `Date::try_new_from_codes`")]
    pub fn try_new_hebrew(
        year: i32,
        ordinal_month: u8,
        day: u8,
    ) -> Result<Date<Hebrew>, RangeError> {
        ArithmeticDate::from_year_month_day(year, ordinal_month, day, &Hebrew)
            .map(HebrewDateInner)
            .map(|inner| Date::from_raw(inner, Hebrew))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCase;

    pub const TISHREI: Month = Month::new(1);
    pub const ḤESHVAN: Month = Month::new(2);
    pub const KISLEV: Month = Month::new(3);
    pub const TEVET: Month = Month::new(4);
    pub const SHEVAT: Month = Month::new(5);
    pub const ADARI: Month = Month::leap(5);
    pub const ADAR: Month = Month::new(6);
    pub const NISAN: Month = Month::new(7);
    pub const IYYAR: Month = Month::new(8);
    pub const SIVAN: Month = Month::new(9);
    pub const TAMMUZ: Month = Month::new(10);
    pub const AV: Month = Month::new(11);
    pub const ELUL: Month = Month::new(12);

    fn month_info(month: Month, leap: bool) -> MonthInfo {
        MonthInfo::from_parts(
            month,
            if (month == ADARI || month.number() >= ADAR.number()) && leap {
                month.number() + 1
            } else {
                assert!(month != ADARI);
                month.number()
            },
        )
    }

    #[test]
    fn test_cases() {
        let cases = [
            TestCase {
                rd: Date::try_new_iso(2021, 1, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(TEVET, false),
                day: 26,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 1, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(SHEVAT, false),
                day: 12,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 2, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(SHEVAT, false),
                day: 28,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 2, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(ADAR, false),
                day: 13,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 3, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(ADAR, false),
                day: 26,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 3, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(NISAN, false),
                day: 12,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 4, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(NISAN, false),
                day: 28,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 4, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(IYYAR, false),
                day: 13,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 5, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(IYYAR, false),
                day: 28,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 5, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(SIVAN, false),
                day: 14,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 6, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(SIVAN, false),
                day: 30,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 6, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(TAMMUZ, false),
                day: 15,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 7, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(AV, false),
                day: 1,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 7, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(AV, false),
                day: 16,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 8, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(ELUL, false),
                day: 2,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 8, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5781,
                year: 5781,
                month: month_info(ELUL, false),
                day: 17,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 9, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(TISHREI, true),
                day: 4,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 9, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(TISHREI, true),
                day: 19,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 10, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ḤESHVAN, true),
                day: 4,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 10, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ḤESHVAN, true),
                day: 19,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 11, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(KISLEV, true),
                day: 6,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 11, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(KISLEV, true),
                day: 21,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 12, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(TEVET, true),
                day: 6,
            },
            TestCase {
                rd: Date::try_new_iso(2021, 12, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(TEVET, true),
                day: 21,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 1, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(SHEVAT, true),
                day: 8,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 1, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(SHEVAT, true),
                day: 23,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 2, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ADARI, true),
                day: 9,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 2, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ADARI, true),
                day: 24,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 3, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ADAR, true),
                day: 7,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 3, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ADAR, true),
                day: 22,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 4, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(NISAN, true),
                day: 9,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 4, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(NISAN, true),
                day: 24,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 5, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(IYYAR, true),
                day: 9,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 5, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(IYYAR, true),
                day: 24,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 6, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(SIVAN, true),
                day: 11,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 6, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(SIVAN, true),
                day: 26,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 7, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(TAMMUZ, true),
                day: 11,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 7, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(TAMMUZ, true),
                day: 26,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 8, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(AV, true),
                day: 13,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 8, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(AV, true),
                day: 28,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 9, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ELUL, true),
                day: 14,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 9, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5782,
                year: 5782,
                month: month_info(ELUL, true),
                day: 29,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 10, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5783,
                year: 5783,
                month: month_info(TISHREI, false),
                day: 15,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 10, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5783,
                year: 5783,
                month: month_info(TISHREI, false),
                day: 30,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 11, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5783,
                year: 5783,
                month: month_info(ḤESHVAN, false),
                day: 16,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 11, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5783,
                year: 5783,
                month: month_info(KISLEV, false),
                day: 1,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 12, 10).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5783,
                year: 5783,
                month: month_info(KISLEV, false),
                day: 16,
            },
            TestCase {
                rd: Date::try_new_iso(2022, 12, 25).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 5783,
                year: 5783,
                month: month_info(TEVET, false),
                day: 1,
            },
            TestCase {
                rd: RataDie::new(734822), // unverified
                era: Some("am"),
                extended_year: 5773,
                year: 5773,
                month: month_info(KISLEV, false),
                day: 1,
            },
            TestCase {
                rd: RataDie::new(450078), // unverified
                era: Some("am"),
                extended_year: 4993,
                year: 4993,
                month: month_info(NISAN, false),
                day: 21,
            },
            TestCase {
                rd: RataDie::new(-1410112), // unverified
                era: Some("am"),
                extended_year: -100,
                year: -100,
                month: month_info(NISAN, true),
                day: 21,
            },
            TestCase {
                rd: RataDie::new(457164), // unverified
                era: Some("am"),
                extended_year: 5012,
                year: 5012,
                month: month_info(ELUL, false),
                day: 20,
            },
            // In Hebrew, there is no inverse era, so negative extended years are negative era years
            TestCase {
                rd: Date::try_new_gregorian(-5000, 1, 1).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: -1240,
                year: -1240,
                month: month_info(SHEVAT, false), // unverified
                day: 28,                          // unverified
            },
            // https://github.com/unicode-org/icu4x/issues/4893
            TestCase {
                rd: Date::try_new_iso(-1, 8, 28).unwrap().to_rata_die(),
                era: Some("am"),
                extended_year: 3760,
                year: 3760,
                month: month_info(TISHREI, false),
                day: 1,
            },
        ];

        for case in cases {
            case.check(&Hebrew);
            case.check_any(Hebrew);
            #[allow(deprecated)]
            case.check_constructor(&Hebrew, |date| {
                Date::try_new_hebrew(
                    date.era_year().extended_year,
                    date.month().ordinal,
                    date.day_of_month().0,
                )
            });
        }
    }
}
