// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

mod continuity_test;
mod exhaustive;
mod extrema;
mod not_enough_fields;

#[derive(Debug)]
pub(crate) struct TestCase {
    pub(crate) rd: crate::types::RataDie,
    pub(crate) era: Option<&'static str>,
    pub(crate) extended_year: i32,
    pub(crate) year: i32,
    pub(crate) month: MonthInfo,
    pub(crate) day: u8,
}

use crate::{
    error::DateFromFieldsError,
    options::{DateFromFieldsOptions, Overflow},
    types::{DateFields, DayOfMonth, Month, MonthInfo, YearInfo},
    Calendar, Date, IntoAnyCalendar, Ref,
};

impl TestCase {
    pub(crate) fn check<'a, C: Calendar>(&self, calendar: &'a C) -> Date<Ref<'a, C>> {
        let date = Date::from_rata_die(self.rd, Ref(calendar));

        assert_eq!(date.to_rata_die(), self.rd, "{self:?}");

        match date.year() {
            YearInfo::Era(era_year) => {
                assert_eq!(era_year.extended_year, self.extended_year, "{self:?}");
                assert_eq!(era_year.year, self.year, "{self:?}");
                assert_eq!(era_year.era, self.era.unwrap(), "{self:?}");

                assert_eq!(
                    Date::try_new_from_codes(
                        Some(&era_year.era),
                        era_year.year,
                        date.month().value.code(),
                        date.day_of_month().0,
                        Ref(calendar)
                    ),
                    Ok(date)
                );
            }
            YearInfo::Cyclic(cyclic_year) => {
                assert_eq!(cyclic_year.related_iso, self.extended_year, "{self:?}");
                assert_eq!(cyclic_year.year as i32, self.year, "{self:?}");
                assert_eq!(self.era, None);
            }
        };

        assert_eq!(date.month(), self.month, "{self:?}");
        #[allow(deprecated)]
        {
            // just checks consistency
            assert_eq!(
                date.month().standard_code,
                date.month().value.code(),
                "{self:?}"
            );
            assert_eq!(
                date.month().formatting_code,
                date.month().value.formatting_code(),
                "{self:?}"
            );
        }
        assert_eq!(date.day_of_month(), DayOfMonth(self.day), "{self:?}");

        assert_eq!(
            Date::try_new_from_codes(
                None,
                date.year().extended_year(),
                date.month().value.code(),
                date.day_of_month().0,
                Ref(calendar)
            ),
            Ok(date)
        );

        date
    }

    pub(crate) fn check_any<C: IntoAnyCalendar>(&self, calendar: C) {
        self.check(&calendar.to_any());
    }

    #[track_caller]
    pub fn check_constructor<
        C: Calendar,
        E: std::fmt::Debug + std::cmp::PartialEq,
        F: Fn(Date<Ref<C>>) -> Result<Date<C>, E>,
    >(
        &self,
        calendar: &C,
        reconstruct: F,
    ) {
        let date = Date::from_rata_die(self.rd, Ref(calendar));
        assert_eq!(
            reconstruct(date).as_ref().map(|d| d.as_borrowed()),
            Ok(date),
            "{self:?}"
        );
    }

    pub fn rd(&self, calendar: &impl Calendar) {
        println!(
            "rd: RataDie::new({})",
            Date::try_new_from_codes(
                None,
                self.extended_year,
                self.month.value.code(),
                self.day,
                Ref(calendar)
            )
            .unwrap()
            .to_rata_die()
            .to_i64_date()
        );
    }
}

pub(crate) struct ErrorTestCase {
    pub(crate) era: Option<&'static str>,
    pub(crate) year: i32,
    pub(crate) month: Month,
    pub(crate) day: u8,
    pub(crate) error: DateFromFieldsError,
}

impl ErrorTestCase {
    pub(crate) fn check<C: Calendar>(&self, calendar: &C) {
        assert_eq!(
            Date::try_from_fields(
                DateFields {
                    era_year: self.era.is_some().then_some(self.year),
                    extended_year: self.era.is_none().then_some(self.year),
                    era: self.era.map(|s| s.as_bytes()),
                    month_code: Some(self.month.code().0.as_bytes()),
                    ordinal_month: None,
                    day: Some(self.day),
                },
                DateFromFieldsOptions {
                    overflow: Some(Overflow::Reject),
                    ..Default::default()
                },
                Ref(calendar)
            ),
            Err(self.error)
        )
    }

    pub(crate) fn check_any<C: IntoAnyCalendar>(&self, calendar: C) {
        self.check(&calendar.to_any())
    }
}

macro_rules! test_all_cals {
    ($(#[$meta:meta])* fn $name:ident<C: Calendar>($cal:ident: Ref<C>) $tt:tt) => {
        mod $name {
            #[allow(unused_imports)]
            use super::*;

            fn test<C: crate::Calendar>(cal: C) {
                let $cal = crate::Ref(&cal);
                $tt
            }

            $(#[$meta])*
            #[test]
            fn buddhist() {
                test(crate::cal::Buddhist);
            }

            $(#[$meta])*
            #[test]
            fn chinese_traditional() {
                test(crate::cal::east_asian_traditional::EastAsianTraditional(crate::cal::east_asian_traditional_internal::EastAsianTraditionalYears::new(crate::cal::east_asian_traditional::China::default())));
            }

            $(#[$meta])*
            #[test]
            fn coptic() {
                test(crate::cal::Coptic);
            }

            $(#[$meta])*
            #[test]
            fn korean_traditional() {
                test(crate::cal::east_asian_traditional::EastAsianTraditional(crate::cal::east_asian_traditional_internal::EastAsianTraditionalYears::new(crate::cal::east_asian_traditional::Korea::default())));
            }

            $(#[$meta])*
            #[test]
            fn ethiopian() {
                test(crate::cal::Ethiopian::new());
            }

            $(#[$meta])*
            #[test]
            fn ethiopian_amete_alem() {
                test(crate::cal::Ethiopian::new_with_era_style(crate::cal::EthiopianEraStyle::AmeteAlem));
            }

            $(#[$meta])*
            #[test]
            fn gregorian() {
                test(crate::cal::Gregorian);
            }

            $(#[$meta])*
            #[test]
            fn hebrew() {
                test(crate::cal::Hebrew::new());
            }

            $(#[$meta])*
            #[test]
            fn hijri_tabular_friday() {
                test(crate::cal::Hijri::new_tabular(crate::cal::hijri::TabularAlgorithmLeapYears::TypeII, crate::cal::hijri::TabularAlgorithmEpoch::Friday));
            }

            $(#[$meta])*
            #[test]
            fn hijri_tabular_thursday() {
                test(crate::cal::Hijri::new_tabular(crate::cal::hijri::TabularAlgorithmLeapYears::TypeII, crate::cal::hijri::TabularAlgorithmEpoch::Thursday));
            }

            $(#[$meta])*
            #[test]
            fn hijri_uaq() {
                test(crate::cal::Hijri::new_umm_al_qura());
            }

            $(#[$meta])*
            #[test]
            fn indian() {
                test(crate::cal::Indian::new());
            }

            $(#[$meta])*
            #[test]
            fn iso() {
                test(crate::cal::Iso::new());
            }

            $(#[$meta])*
            #[test]
            fn julian() {
                test(crate::cal::Julian::new());
            }

            $(#[$meta])*
            #[test]
            fn japanese() {
                test(crate::cal::Japanese::new());
            }

            $(#[$meta])*
            #[test]
            fn japanese_extended() {
                test(crate::cal::JapaneseExtended::new());
            }

            $(#[$meta])*
            #[test]
            fn persian() {
                test(crate::cal::Persian::new());
            }

            $(#[$meta])*
            #[test]
            fn roc() {
                test(crate::cal::Roc);
            }
        }
    };
}
use test_all_cals;
