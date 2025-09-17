// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::cal::iso::{Iso, IsoDateInner};
use crate::cal::EthiopianEraStyle;
use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
use crate::error::{range_check, DateError};
use crate::types::EraYear;
use crate::{types, Calendar, Date, DateDuration, DateDurationUnit, RangeError};
use calendrical_calculations::helpers::I32CastError;
use calendrical_calculations::rata_die::RataDie;
use tinystr::tinystr;

/// TODO
pub type Coptic = CopticBased<CopticEra>;

pub trait CopticEras: core::fmt::Debug + Clone {
    const EXTENDED_YEAR_OFFSET: i32;

    fn get(&self, extended_year: i32) -> EraYear;
    fn resolve(&self, era: Option<&str>, year: i32) -> Result<i32, DateError>;

    fn debug_name(&self) -> &'static str;
    fn calendar_algorithm(&self) -> Option<crate::preferences::CalendarAlgorithm>;
}

#[derive(Debug, Clone)]
pub struct CopticEra;

impl CopticEras for CopticEra {
    const EXTENDED_YEAR_OFFSET: i32 = 0;

    fn get(&self, extended_year: i32) -> EraYear {
        types::EraYear {
            era: tinystr!(16, "am"),
            era_index: Some(0),
            year: extended_year,
            extended_year,
            ambiguity: types::YearAmbiguity::CenturyRequired,
        }
    }

    fn resolve(&self, era: Option<&str>, year: i32) -> Result<i32, DateError> {
        match era {
            Some("am") | None => Ok(year),
            Some(_) => Err(DateError::UnknownEra),
        }
    }

    fn debug_name(&self) -> &'static str {
        "Coptic"
    }

    fn calendar_algorithm(&self) -> Option<crate::preferences::CalendarAlgorithm> {
        Some(crate::preferences::CalendarAlgorithm::Coptic)
    }
}


/// The number of years the Amete Alem epoch precedes the Amete Mihret epoch
const INCARNATION_OFFSET: i32 = 5500;

impl CopticEras for EthiopianEraStyle {
    const EXTENDED_YEAR_OFFSET: i32 = 275;

    fn get(&self, year: i32) -> EraYear {
        let extended_year = if *self == Self::AmeteAlem {
            year
        } else {
            year - INCARNATION_OFFSET
        };

        if self.0 || extended_year <= 0 {
            types::EraYear {
                era: tinystr!(16, "aa"),
                era_index: Some(0),
                year,
                extended_year,
                ambiguity: types::YearAmbiguity::CenturyRequired,
            }
        } else {
            types::EraYear {
                era: tinystr!(16, "am"),
                era_index: Some(1),
                year: year - INCARNATION_OFFSET,
                extended_year,
                ambiguity: types::YearAmbiguity::CenturyRequired,
            }
        }
    }

    fn resolve(&self, era: Option<&str>, year: i32) -> Result<i32, DateError> {
        Ok(match (self, era) {
            (EthiopianEraStyle::AmeteMihret, Some("am")) => {
                range_check(year, "year", 1..)? + INCARNATION_OFFSET
            }
            (EthiopianEraStyle::AmeteMihret, None) => year + INCARNATION_OFFSET,
            (EthiopianEraStyle::AmeteMihret, Some("aa")) => {
                range_check(year, "year", ..=INCARNATION_OFFSET)?
            }
            (EthiopianEraStyle::AmeteAlem, Some("aa") | None) => year,
            (_, Some(_)) => {
                return Err(DateError::UnknownEra);
            }
        })
    }

    fn debug_name(&self) -> &'static str {
        "Ethiopian"
    }

    fn calendar_algorithm(&self) -> Option<crate::preferences::CalendarAlgorithm> {
        Some(crate::preferences::CalendarAlgorithm::Ethiopic)
    }
}

/// The [Coptic Calendar]
///
/// The [Coptic calendar] is a solar calendar used by the Coptic Orthodox Church, with twelve normal months
/// and a thirteenth small epagomenal month.
///
/// This type can be used with [`Date`] to represent dates in this calendar.
///
/// [Coptic calendar]: https://en.wikipedia.org/wiki/Coptic_calendar
///
/// # Era codes
///
/// This calendar uses a single code: `am`, corresponding to the After Diocletian/Anno Martyrum
/// era. 1 A.M. is equivalent to 284 C.E.
///
/// # Month codes
///
/// This calendar supports 13 solar month codes (`"M01" - "M13"`), with `"M13"` being used for the short epagomenal month
/// at the end of the year.
#[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct CopticBased<E: CopticEras>(E);

impl CopticBased<CopticEra> {
    /// TODO
    pub const fn new() -> Self {
        Self(CopticEra)
    }
}

/// The inner date type used for representing [`Date`]s of [`Coptic`]. See [`Date`] and [`Coptic`] for more details.
#[derive(Clone, Debug)]
pub struct CopticDateInner<E: CopticEras>(pub(crate) ArithmeticDate<CopticBased<E>>);

impl<E: CopticEras> Copy for CopticDateInner<E> {}
impl<E: CopticEras> PartialEq for CopticDateInner<E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<E: CopticEras> Eq for CopticDateInner<E> {}

impl<E: CopticEras> CalendarArithmetic for CopticBased<E> {
    type YearInfo = i32;

    fn days_in_provided_month(year: i32, month: u8) -> u8 {
        if (1..=12).contains(&month) {
            30
        } else if month == 13 {
            if Self::provided_year_is_leap(year) {
                6
            } else {
                5
            }
        } else {
            0
        }
    }

    fn months_in_provided_year(_: i32) -> u8 {
        13
    }

    fn provided_year_is_leap(year: i32) -> bool {
        year.rem_euclid(4) == 3
    }

    fn last_month_day_in_provided_year(year: i32) -> (u8, u8) {
        if Self::provided_year_is_leap(year) {
            (13, 6)
        } else {
            (13, 5)
        }
    }

    fn days_in_provided_year(year: i32) -> u16 {
        if Self::provided_year_is_leap(year) {
            366
        } else {
            365
        }
    }
}

impl<E: CopticEras> crate::cal::scaffold::UnstableSealed for CopticBased<E> {}
impl<E: CopticEras> Calendar for CopticBased<E> {
    type DateInner = CopticDateInner<E>;
    type Year = types::EraYear;
    fn from_codes(
        &self,
        era: Option<&str>,
        year: i32,
        month_code: types::MonthCode,
        day: u8,
    ) -> Result<Self::DateInner, DateError> {
        ArithmeticDate::new_from_codes(self, self.0.resolve(era, year)? + E::EXTENDED_YEAR_OFFSET, month_code, day).map(CopticDateInner)
    }

    fn from_rata_die(&self, rd: RataDie) -> Self::DateInner {
        CopticDateInner(
            match calendrical_calculations::coptic::coptic_from_fixed(rd) {
                Err(I32CastError::BelowMin) => ArithmeticDate::min_date(),
                Err(I32CastError::AboveMax) => ArithmeticDate::max_date(),
                Ok((year, month, day)) => ArithmeticDate::new_unchecked(year, month, day),
            },
        )
    }

    fn to_rata_die(&self, date: &Self::DateInner) -> RataDie {
        calendrical_calculations::coptic::fixed_from_coptic(date.0.year, date.0.month, date.0.day)
    }

    fn from_iso(&self, iso: IsoDateInner) -> CopticDateInner<E> {
        self.from_rata_die(Iso.to_rata_die(&iso))
    }

    fn to_iso(&self, date: &Self::DateInner) -> IsoDateInner {
        Iso.from_rata_die(self.to_rata_die(date))
    }

    fn months_in_year(&self, date: &Self::DateInner) -> u8 {
        date.0.months_in_year()
    }

    fn days_in_year(&self, date: &Self::DateInner) -> u16 {
        date.0.days_in_year()
    }

    fn days_in_month(&self, date: &Self::DateInner) -> u8 {
        date.0.days_in_month()
    }

    fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
        date.0.offset_date(offset, &());
    }

    fn until(
        &self,
        date1: &Self::DateInner,
        date2: &Self::DateInner,
        _calendar2: &Self,
        _largest_unit: DateDurationUnit,
        _smallest_unit: DateDurationUnit,
    ) -> DateDuration<Self> {
        date1.0.until(date2.0, _largest_unit, _smallest_unit)
    }

    fn year_info(&self, date: &Self::DateInner) -> Self::Year {
        self.0.get(date.0.extended_year() - E::EXTENDED_YEAR_OFFSET)
    }

    fn is_in_leap_year(&self, date: &Self::DateInner) -> bool {
        Self::provided_year_is_leap(date.0.year)
    }

    fn month(&self, date: &Self::DateInner) -> types::MonthInfo {
        date.0.month()
    }

    fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
        date.0.day_of_month()
    }

    fn day_of_year(&self, date: &Self::DateInner) -> types::DayOfYear {
        date.0.day_of_year()
    }

    fn debug_name(&self) -> &'static str {
        self.0.debug_name()
    }

    fn calendar_algorithm(&self) -> Option<crate::preferences::CalendarAlgorithm> {
        self.0.calendar_algorithm()
    }
}

impl Date<CopticBased<CopticEra>> {
    /// Construct new Coptic Date.
    ///
    /// ```rust
    /// use icu::calendar::Date;
    ///
    /// let date_coptic = Date::try_new_coptic(1686, 5, 6)
    ///     .expect("Failed to initialize Coptic Date instance.");
    ///
    /// assert_eq!(date_coptic.era_year().year, 1686);
    /// assert_eq!(date_coptic.month().ordinal, 5);
    /// assert_eq!(date_coptic.day_of_month().0, 6);
    /// ```
    pub fn try_new_coptic(year: i32, month: u8, day: u8) -> Result<Self, RangeError> {
        ArithmeticDate::new_from_ordinals(year, month, day)
            .map(CopticDateInner)
            .map(|inner| Date::from_raw(inner, CopticBased(CopticEra)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_coptic_regression() {
        // https://github.com/unicode-org/icu4x/issues/2254
        let iso_date = Date::try_new_iso(-100, 3, 3).unwrap();
        let coptic = iso_date.to_calendar(Coptic::new());
        let recovered_iso = coptic.to_iso();
        assert_eq!(iso_date, recovered_iso);
    }
}
