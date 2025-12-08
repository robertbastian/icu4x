// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::cal::abstract_gregorian::{
    impl_with_abstract_gregorian, AbstractGregorian, GregorianYears,
};
use crate::calendar_arithmetic::ArithmeticDate;
use crate::error::UnknownEraError;
use crate::preferences::CalendarAlgorithm;
use crate::{types, Date, DateError, RangeError};
use tinystr::tinystr;

/// The [Republic of China Calendar](https://en.wikipedia.org/wiki/Republic_of_China_calendar)
///
/// The ROC Calendar is a variant of the [`Gregorian`](crate::cal::Gregorian) calendar
/// created by the government of the Republic of China. It is identical to the Gregorian
/// calendar except that is uses the ROC/Minguo/民国/民國 Era (1912 CE) instead of the Common Era.
///
/// This implementation extends proleptically for dates before the calendar's creation
/// in 1 Minguo (1912 CE).
///
/// The ROC calendar should not be confused with the [`ChineseTraditional`](crate::cal::ChineseTraditional)
/// lunisolar calendar.
///
/// This corresponds to the `"roc"` [CLDR calendar](https://unicode.org/reports/tr35/#UnicodeCalendarIdentifier).
///
/// # Era codes
///
/// This calendar uses two era codes: `roc`, corresponding to years in the 民國 era (CE year 1912 and
/// after), and `broc`, corresponding to years before the 民國 era (CE year 1911 and before).
#[derive(Copy, Clone, Debug, Default)]
#[allow(clippy::exhaustive_structs)] // this type is stable
pub struct Roc;

impl_with_abstract_gregorian!(crate::cal::Roc, RocDateInner, RocEra, _x, RocEra);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RocEra;

impl GregorianYears for RocEra {
    const EXTENDED_YEAR_OFFSET: i32 = 1911;

    fn extended_from_era_year(
        &self,
        era: Option<&[u8]>,
        year: i32,
    ) -> Result<i32, UnknownEraError> {
        match era {
            None => Ok(year),
            Some(b"roc") => Ok(year),
            Some(b"broc") => Ok(1 - year),
            Some(_) => Err(UnknownEraError),
        }
    }

    fn era_year_from_extended(&self, extended_year: i32, _month: u8, _day: u8) -> types::EraYear {
        if extended_year > 0 {
            types::EraYear {
                era: tinystr!(16, "roc"),
                era_index: Some(1),
                year: extended_year,
                extended_year,
                ambiguity: types::YearAmbiguity::CenturyRequired,
            }
        } else {
            types::EraYear {
                era: tinystr!(16, "broc"),
                era_index: Some(0),
                year: 1 - extended_year,
                extended_year,
                ambiguity: types::YearAmbiguity::EraAndCenturyRequired,
            }
        }
    }

    fn debug_name(&self) -> &'static str {
        "ROC"
    }

    fn calendar_algorithm(&self) -> Option<CalendarAlgorithm> {
        Some(CalendarAlgorithm::Roc)
    }
}

impl Date<Roc> {
    /// Construct a new Republic of China calendar [`Date`].
    ///
    /// Years are arithmetic, meaning there is a year 0 preceded by negative years, with a
    /// valid range of `-1,000,000..=1,000,000`.
    ///
    /// ```rust
    /// use icu::calendar::Date;
    /// use icu::calendar::cal::Gregorian;
    /// use tinystr::tinystr;
    ///
    /// // Create a new ROC Date
    /// let date_roc = Date::try_new_roc(1, 2, 3)
    ///     .expect("Failed to initialize ROC Date instance.");
    ///
    /// assert_eq!(date_roc.era_year().era, "roc");
    /// assert_eq!(date_roc.era_year().year, 1, "ROC year check failed!");
    /// assert_eq!(date_roc.month().ordinal, 2, "ROC month check failed!");
    /// assert_eq!(date_roc.day_of_month().0, 3, "ROC day of month check failed!");
    ///
    /// // Convert to an equivalent Gregorian date
    /// let date_gregorian = date_roc.to_calendar(Gregorian);
    ///
    /// assert_eq!(date_gregorian.era_year().year, 1912, "Gregorian from ROC year check failed!");
    /// assert_eq!(date_gregorian.month().ordinal, 2, "Gregorian from ROC month check failed!");
    /// assert_eq!(date_gregorian.day_of_month().0, 3, "Gregorian from ROC day of month check failed!");
    pub fn try_new_roc(year: i32, month: u8, day: u8) -> Result<Date<Roc>, RangeError> {
        ArithmeticDate::from_year_month_day(year, month, day, &AbstractGregorian(RocEra))
            .map(ArithmeticDate::cast)
            .map(RocDateInner)
            .map(|i| Date::from_raw(i, Roc))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{
        cal::Iso,
        tests::TestCase,
        types::{Month, MonthInfo},
    };
    use calendrical_calculations::rata_die::RataDie;

    #[test]
    fn test_roc_current_era() {
        fn month_info(ordinal: u8) -> MonthInfo {
            MonthInfo::from_parts(Month::new(ordinal), ordinal)
        }
        let cases = [
            TestCase {
                rd: Date::try_new_iso(1912, 1, 1).unwrap().to_rata_die(),
                extended_year: 1,
                era: Some("roc"),
                year: 1,
                month: month_info(1),
                day: 1,
            },
            TestCase {
                rd: Date::try_new_iso(1912, 2, 29).unwrap().to_rata_die(),
                extended_year: 1,
                era: Some("roc"),
                year: 1,
                month: month_info(2),
                day: 29,
            },
            TestCase {
                rd: Date::try_new_iso(1913, 6, 30).unwrap().to_rata_die(),
                extended_year: 2,
                era: Some("roc"),
                year: 2,
                month: month_info(6),
                day: 30,
            },
            TestCase {
                rd: Date::try_new_iso(2023, 7, 13).unwrap().to_rata_die(),
                extended_year: 112,
                era: Some("roc"),
                year: 112,
                month: month_info(7),
                day: 13,
            },
            TestCase {
                rd: Date::try_new_iso(1911, 12, 31).unwrap().to_rata_die(),
                era: Some("broc"),
                extended_year: 1 - 1,
                year: 1,
                month: month_info(12),
                day: 31,
            },
            TestCase {
                rd: Date::try_new_iso(1911, 1, 1).unwrap().to_rata_die(),
                era: Some("broc"),
                extended_year: 1 - 1,
                year: 1,
                month: month_info(1),
                day: 1,
            },
            TestCase {
                rd: Date::try_new_iso(1910, 12, 31).unwrap().to_rata_die(),
                era: Some("broc"),
                extended_year: 1 - 2,
                year: 2,
                month: month_info(12),
                day: 31,
            },
            TestCase {
                rd: Date::try_new_iso(1908, 2, 29).unwrap().to_rata_die(),
                era: Some("broc"),
                extended_year: 1 - 4,
                year: 4,
                month: month_info(2),
                day: 29,
            },
            TestCase {
                rd: Date::try_new_iso(1, 1, 1).unwrap().to_rata_die(),
                era: Some("broc"),
                extended_year: 1 - 1911,
                year: 1911,
                month: month_info(1),
                day: 1,
            },
            TestCase {
                rd: Date::try_new_iso(0, 12, 31).unwrap().to_rata_die(),
                era: Some("broc"),
                extended_year: 1 - 1912,
                year: 1912,
                month: month_info(12),
                day: 31,
            },
            TestCase {
                rd: RataDie::new(701388),
                era: Some("roc"),
                extended_year: 10,
                year: 10,
                month: month_info(5),
                day: 3,
            },
            TestCase {
                rd: RataDie::new(692510),
                era: Some("broc"),
                extended_year: 1 - 15,
                year: 15,
                month: month_info(1),
                day: 10,
            },
            TestCase {
                rd: RataDie::new(734440),
                era: Some("roc"),
                extended_year: 100,
                year: 100,
                month: month_info(10),
                day: 30,
            },
            TestCase {
                rd: RataDie::new(661391),
                era: Some("broc"),
                extended_year: -100,
                year: 101,
                month: month_info(10),
                day: 30,
            },
        ];

        for case in cases {
            case.check(&Roc);
            case.check_any(Roc);
            case.check_constructor(&Roc, |date| {
                Date::try_new_roc(
                    date.extended_year(),
                    date.month().ordinal,
                    date.day_of_month().0,
                )
            });
        }
    }

    #[test]
    fn test_roc_directionality_near_epoch() {
        // Tests that for a large range of RDs near the beginning of the minguo era (CE 1912),
        // the comparison between those two RDs should be equal to the comparison between their
        // corresponding YMD.
        let rd_epoch_start = 697978;
        for i in (rd_epoch_start - 100)..=(rd_epoch_start + 100) {
            for j in (rd_epoch_start - 100)..=(rd_epoch_start + 100) {
                let iso_i = Date::from_rata_die(RataDie::new(i), Iso);
                let iso_j = Date::from_rata_die(RataDie::new(j), Iso);

                let roc_i = Date::from_rata_die(RataDie::new(i), Roc);
                let roc_j = Date::from_rata_die(RataDie::new(j), Roc);

                assert_eq!(
                    i.cmp(&j),
                    iso_i.cmp(&iso_j),
                    "ISO directionality inconsistent with directionality for i: {i}, j: {j}"
                );
                assert_eq!(
                    i.cmp(&j),
                    roc_i.cmp(&roc_j),
                    "ROC directionality inconsistent with directionality for i: {i}, j: {j}"
                );
            }
        }
    }

    #[test]
    fn test_roc_directionality_near_rd_zero() {
        // Same as `test_directionality_near_epoch`, but with a focus around RD 0
        for i in -100..=100 {
            for j in -100..100 {
                let iso_i = Date::from_rata_die(RataDie::new(i), Iso);
                let iso_j = Date::from_rata_die(RataDie::new(j), Iso);

                let roc_i = Date::from_rata_die(RataDie::new(i), Roc);
                let roc_j = Date::from_rata_die(RataDie::new(j), Roc);

                assert_eq!(
                    i.cmp(&j),
                    iso_i.cmp(&iso_j),
                    "ISO directionality inconsistent with directionality for i: {i}, j: {j}"
                );
                assert_eq!(
                    i.cmp(&j),
                    roc_i.cmp(&roc_j),
                    "ROC directionality inconsistent with directionality for i: {i}, j: {j}"
                );
            }
        }
    }
}
