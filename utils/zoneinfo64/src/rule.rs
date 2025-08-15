// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use super::{Offset, PossibleOffset};
use calendrical_calculations::iso;
use calendrical_calculations::rata_die::RataDie;
use icu_time::zone::UtcOffset;
use iso::is_leap_year;

const SECONDS_IN_UTC_DAY: i64 = 86400;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Rule<'a> {
    /// The year the rule starts applying
    pub(crate) start_year: u32,
    /// The offset of standard time
    pub(crate) standard_offset_seconds: i32,
    pub(crate) inner: &'a TzRule,
}

#[derive(Debug)]
pub(crate) struct TzRule {
    /// The amount of seconds to add to standard_offset_seconds
    /// to get the rule offset
    pub(crate) additional_offset_secs: i32,
    /// The yearly start date of the rule
    pub(crate) start: TzRuleDate,
    /// The yearly end date of the rule
    pub(crate) end: TzRuleDate,
}

#[derive(Debug)]
pub(crate) struct TzRuleDate {
    /// A 1-indexed day number
    pub(crate) day: u8,
    /// A 1-indexed day of the week (1 = Sunday)
    pub(crate) day_of_week: u8,
    /// A 0-indexed month number
    pub(crate) month: u8,
    /// The time in the day that the transition occurs, in seconds
    pub(crate) transition_time: u32,
    /// How to interpret transition_time
    pub(crate) time_mode: TimeMode,
    /// How to interpret day, day_of_week, and month
    pub(crate) mode: RuleMode,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum TimeMode {
    /// {transition_time} is local wall clock time in the time zone
    /// *before* the transition
    ///
    /// e.g. if the transition between LST and LDT is to happen at 02:00,
    /// the time that *would be* 02:00 LST would be the first time of LDT.
    ///
    /// This means that `{local_wall_clock_time}` may never actually be the
    /// wall clock time! The America/Los_Angeles transition occurs at Wall 02:00,
    /// however the transition from PST to PDT is
    /// `2025-03-09T01:59:59-08:00[America/Los_Angeles]` to
    /// 2025-03-09T03:00:00-07:00[America/Los_Angeles],
    /// so 2025-03-09T02:00:00 never occurs.
    ///
    /// This can be turned into Standard by subtracting the offset-from-standard
    /// of the time zone *before* this transition
    Wall = 0,
    /// {transition_time} is local standard time
    ///
    /// Will produce different results from Wall=0 for DST-to-STD transitions
    ///
    /// This can be turned into Wall by adding the offset-from-standard of the time zone
    /// *before* this transition.
    Standard = 1,
    /// {transition_time} is UTC time
    ///
    /// This is UTC time *on the UTC day* identified by this rule; which may
    /// end up on a different local day.
    ///
    /// For example, America/Santiago transitions to STD on the first Sunday after April 2.
    /// at UTC 03:00:00, which is `2025-04-06T03:00:00+00:00[UTC]`. This ends up being
    /// a transition from`2025-04-05T23:59:59-03:00[America/Santiago]` to
    /// `2025-04-05T23:00:00-04:00[America/Santiago]`).
    ///
    /// This can be turned into Standard by subtracting the standard-offset-from-UTC of the
    /// time zone. It can be turned into Wall by subtracting the offset-from-UTC of the time zone
    /// before this transition.
    Utc = 2,
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
/// How to interpret `{day}` `{day_of_week}` and `{month}`
pub(crate) enum RuleMode {
    /// The {day}th {day_of_week} in {month}
    ///
    /// Current zoneinfo64 does not use this, instead
    /// choosing to represent this as DOW_GEQ_DOM with day = 1/8/15/22
    DOW_IN_MONTH,
    /// {month} {day}
    ///
    /// Current zoneinfo64 does not use this
    DOM,
    /// The first {day_of_week} on or after {month} {day}
    DOW_GEQ_DOM,
    /// The first {day_of_week} on or before {month} {day}
    ///
    /// Typically, this represents rules like "Last Sunday in March" (Europe/London)
    DOW_LEQ_DOM,
}

/// The number of days in this year before this (0-based, from TzRuleDate.month) month starts
fn days_before_month(m: u8, is_leap: bool) -> u16 {
    // This takes a 1-based month
    const fn days_before_month_in_non_leap_year(month: u8) -> u16 {
        (iso::const_fixed_from_iso(2021, month, 1).to_i64_date()
            - iso::const_fixed_from_iso(2021, 1, 1).to_i64_date()) as u16
    }

    let leap_day = u16::from(is_leap);
    match m {
        0 => const { days_before_month_in_non_leap_year(1) },
        1 => const { days_before_month_in_non_leap_year(2) },
        2 => (const { days_before_month_in_non_leap_year(3) }) + leap_day,
        3 => (const { days_before_month_in_non_leap_year(4) }) + leap_day,
        4 => (const { days_before_month_in_non_leap_year(5) }) + leap_day,
        5 => (const { days_before_month_in_non_leap_year(6) }) + leap_day,
        6 => (const { days_before_month_in_non_leap_year(7) }) + leap_day,
        7 => (const { days_before_month_in_non_leap_year(8) }) + leap_day,
        8 => (const { days_before_month_in_non_leap_year(9) }) + leap_day,
        9 => (const { days_before_month_in_non_leap_year(10) }) + leap_day,
        10 => (const { days_before_month_in_non_leap_year(11) }) + leap_day,
        11 => (const { days_before_month_in_non_leap_year(12) }) + leap_day,
        _ => unreachable!(),
    }
}

/// The weekday before this year started (0 = Sun, 6 -=Sat)
fn weekday_before_year(year: i32) -> u8 {
    const SUNDAY: RataDie = iso::const_fixed_from_iso(2000, 1, 2);
    (iso::fixed_from_iso(year - 1, 12, 31) - SUNDAY).rem_euclid(7) as u8
}

/// Represent the year as a number of days since the start of the 1970 epoch
///
/// i.e. days_since_epoch(1970) = 0
fn days_since_epoch(year: i32) -> i64 {
    let rd = iso::fixed_from_iso(year, 1, 1);
    rd - super::EPOCH
}

impl TzRuleDate {
    /// Given a year, return the 0-indexed day number in that year for this transition
    pub(crate) fn day_in_year(&self, year: i32) -> u16 {
        let is_leap = is_leap_year(year);
        let weekday_before_year = weekday_before_year(year);
        let days_before_month = days_before_month(self.month, is_leap);
        let weekday_before_month =
            (weekday_before_year + days_before_month.rem_euclid(7) as u8).rem_euclid(7);

        // Turn this into a zero-indexed day of week
        let day_of_week_0idx = self.day_of_week - 1;
        let day_of_month = match self.mode {
            RuleMode::DOM => self.day,
            RuleMode::DOW_IN_MONTH => {
                // First we calculate the first {day_of_week} of the month
                let first_weekday = if day_of_week_0idx > weekday_before_year {
                    day_of_week_0idx - weekday_before_month
                } else {
                    7 + day_of_week_0idx - weekday_before_month
                };

                // Then we add additional weeks to it if desired
                first_weekday + (self.day - 1) * 7
            }
            // These two compute after/before an "anchor" day in the month
            RuleMode::DOW_GEQ_DOM => {
                let weekday_of_anchor = (weekday_before_month + self.day).rem_euclid(7);
                let days_to_add = if day_of_week_0idx >= weekday_of_anchor {
                    day_of_week_0idx - weekday_of_anchor
                } else {
                    7 + day_of_week_0idx - weekday_of_anchor
                };
                self.day + days_to_add
            }
            RuleMode::DOW_LEQ_DOM => {
                let weekday_of_anchor = (weekday_before_month + self.day).rem_euclid(7);
                let days_to_subtract = if day_of_week_0idx <= weekday_of_anchor {
                    weekday_of_anchor - day_of_week_0idx
                } else {
                    7 - day_of_week_0idx + weekday_of_anchor
                };
                self.day - days_to_subtract
            }
        };
        // Subtract one so we get a 0-indexed value (Jan 1 = day 0)
        days_before_month + u16::from(day_of_month) - 1
    }

    /// Converts the {transition_time} into a time in the UTC day, in seconds
    /// for either the start or end trnasition
    fn transition_time_to_utc(
        &self,
        standard_offset_seconds: i32,
        additional_offset_seconds: i32,
    ) -> i32 {
        let seconds_of_day = self.transition_time as i32;
        match self.time_mode {
            TimeMode::Utc => seconds_of_day,
            TimeMode::Standard => seconds_of_day - standard_offset_seconds,
            TimeMode::Wall => {
                seconds_of_day - (standard_offset_seconds + additional_offset_seconds)
            }
        }
    }

    /// Converts the {transition_time} into a time in the UTC day, in seconds
    /// for either the start or end trnasition
    fn transition_time_to_wall(
        &self,
        standard_offset_seconds: i32,
        additional_offset_seconds: i32,
    ) -> i32 {
        let seconds_of_day = self.transition_time as i32;
        match self.time_mode {
            TimeMode::Utc => seconds_of_day + standard_offset_seconds + additional_offset_seconds,
            TimeMode::Standard => seconds_of_day + additional_offset_seconds,
            TimeMode::Wall => seconds_of_day,
        }
    }
}

impl TzRule {
    pub(crate) fn from_raw(value: &[i32; 11]) -> Self {
        Self {
            additional_offset_secs: value[10],
            start: TzRuleDate::new(
                value[1] as i8,
                value[2] as i8,
                value[0] as u8,
                value[3] as u32,
                value[4] as i8,
            )
            .unwrap(),
            end: TzRuleDate::new(
                value[6] as i8,
                value[7] as i8,
                value[5] as u8,
                value[8] as u32,
                value[9] as i8,
            )
            .unwrap(),
        }
    }

    pub(crate) fn is_inverted(&self) -> bool {
        (self.start.month, self.start.day) > (self.end.month, self.end.day)
    }
}

impl Rule<'_> {
    fn first_for_year(&self, year: i32) -> Offset {
        if self.inner.is_inverted() {
            self.end_for_year(year)
        } else {
            self.start_for_year(year)
        }
    }

    fn second_for_year(&self, year: i32) -> Offset {
        if self.inner.is_inverted() {
            self.start_for_year(year)
        } else {
            self.end_for_year(year)
        }
    }

    fn start_for_year(&self, year: i32) -> Offset {
        let days_since_epoch = days_since_epoch(year);

        let start = &self.inner.start;
        let start_day_in_year = start.day_in_year(year);
        let start_seconds = start.transition_time_to_utc(self.standard_offset_seconds, 0);
        let start_epoch_seconds = (days_since_epoch + i64::from(start_day_in_year))
            * SECONDS_IN_UTC_DAY
            + i64::from(start_seconds);

        Offset {
            since: start_epoch_seconds,
            offset: UtcOffset::from_seconds_unchecked(
                self.standard_offset_seconds + self.inner.additional_offset_secs,
            ),
            rule_applies: true,
        }
    }

    fn end_for_year(&self, year: i32) -> Offset {
        let days_since_epoch = days_since_epoch(year);

        let end = &self.inner.end;
        let end_day_in_year = end.day_in_year(year);
        let end_seconds = end.transition_time_to_utc(
            self.standard_offset_seconds,
            self.inner.additional_offset_secs,
        );
        let end_epoch_seconds = (days_since_epoch + i64::from(end_day_in_year))
            * SECONDS_IN_UTC_DAY
            + i64::from(end_seconds);

        Offset {
            since: end_epoch_seconds,
            offset: UtcOffset::from_seconds_unchecked(self.standard_offset_seconds),
            rule_applies: false,
        }
    }

    pub fn for_date_time(
        &self,
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Option<PossibleOffset> {
        if year < self.start_year as i32 {
            return None;
        }
        let day_in_year =
            (iso::fixed_from_iso(year, month, day) - iso::fixed_from_iso(year, 1, 1)) as i64;
        let local_second_in_day = ((hour as i64 * 60) + minute as i64) * 60 + second as i64;

        let mut before_start_day = self.inner.start.day_in_year(year) as i64;
        let mut before_start_seconds =
            self.inner
                .start
                .transition_time_to_wall(self.standard_offset_seconds, 0) as i64;
        if before_start_seconds < 0 {
            before_start_seconds += SECONDS_IN_UTC_DAY;
            before_start_day -= 1;
        }

        let mut after_start_day = self.inner.start.day_in_year(year) as i64;
        let mut after_start_seconds = (self
            .inner
            .start
            .transition_time_to_wall(self.standard_offset_seconds, 0)
            + self.inner.additional_offset_secs) as i64;
        if after_start_seconds < 0 {
            after_start_seconds += SECONDS_IN_UTC_DAY;
            after_start_day -= 1;
        }

        let mut before_end_day = self.inner.end.day_in_year(year) as i64;
        let mut before_end_seconds = (self.inner.end.transition_time_to_wall(
            self.standard_offset_seconds,
            self.inner.additional_offset_secs,
        ) - self.inner.additional_offset_secs) as i64;
        if before_end_seconds < 0 {
            before_end_seconds += SECONDS_IN_UTC_DAY;
            before_end_day -= 1;
        }

        let mut after_end_day = self.inner.end.day_in_year(year) as i64;
        let mut after_end_seconds = self.inner.end.transition_time_to_wall(
            self.standard_offset_seconds,
            self.inner.additional_offset_secs,
        ) as i64;
        if after_end_seconds < 0 {
            after_end_seconds += SECONDS_IN_UTC_DAY;
            after_end_day -= 1;
        }

        #[allow(clippy::collapsible_else_if)] // symmetry
        if !self.inner.is_inverted() {
            if (day_in_year, local_second_in_day) < (before_start_day, before_start_seconds) {
                if year == self.start_year as i32 {
                    return None;
                }
                Some(PossibleOffset::Single(self.end_for_year(year - 1)))
            } else if (day_in_year, local_second_in_day) < (after_start_day, after_start_seconds) {
                Some(PossibleOffset::None)
            } else if (day_in_year, local_second_in_day) < (before_end_day, before_end_seconds) {
                Some(PossibleOffset::Single(self.start_for_year(year)))
            } else if (day_in_year, local_second_in_day) < (after_end_day, after_end_seconds) {
                Some(PossibleOffset::Ambiguous(
                    self.start_for_year(year),
                    self.end_for_year(year),
                ))
            } else {
                Some(PossibleOffset::Single(self.end_for_year(year)))
            }
        } else {
            if (day_in_year, local_second_in_day) < (before_end_day, before_end_seconds) {
                if year == self.start_year as i32 {
                    return None;
                }
                Some(PossibleOffset::Single(self.start_for_year(year - 1)))
            } else if (day_in_year, local_second_in_day) < (after_end_day, after_end_seconds) {
                // if year == self.start_year as i32 {
                //     return None;
                // }
                Some(PossibleOffset::Ambiguous(
                    self.start_for_year(year - 1),
                    self.end_for_year(year),
                ))
            } else if (day_in_year, local_second_in_day) < (before_start_day, before_start_seconds)
            {
                Some(PossibleOffset::Single(self.end_for_year(year)))
            } else if (day_in_year, local_second_in_day) < (after_start_day, after_start_seconds) {
                Some(PossibleOffset::None)
            } else {
                Some(PossibleOffset::Single(self.start_for_year(year)))
            }
        }
    }
}

impl TzRuleDate {
    fn new(
        mut day: i8,
        mut day_of_week: i8,
        month: u8,
        transition_time: u32,
        time_mode: i8,
    ) -> Option<Self> {
        const GREGORIAN_MONTHS: [i8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        if day == 0 {
            return None;
        }
        if month > 11 {
            return None;
        }
        if transition_time > 24 * 60 * 60 * 1000 {
            return None;
        }

        let time_mode = match time_mode {
            0 => TimeMode::Wall,
            1 => TimeMode::Standard,
            2 => TimeMode::Utc,
            _ => return None,
        };

        let mode;

        if day_of_week == 0 {
            mode = RuleMode::DOM;
        } else {
            if day_of_week > 0 {
                mode = RuleMode::DOW_IN_MONTH
            } else {
                day_of_week = -day_of_week;
                if day > 0 {
                    mode = RuleMode::DOW_GEQ_DOM;
                } else {
                    day = -day;
                    mode = RuleMode::DOW_LEQ_DOM;
                }
            }
            if day_of_week > 7 {
                return None;
            }
        }

        if mode == RuleMode::DOW_IN_MONTH {
            if !(-5..=5).contains(&day) {
                return None;
            }
        } else if day < 1 || day > GREGORIAN_MONTHS[month as usize] {
            return None;
        }

        debug_assert!(day >= 0);
        debug_assert!(day_of_week >= 0);

        Some(Self {
            day: u8::try_from(day).unwrap_or_default(),
            day_of_week: u8::try_from(day_of_week).unwrap_or_default(),
            month,
            transition_time,
            time_mode,
            mode,
        })
    }
}

impl Rule<'_> {
    // Returns the [`TransitionOffset`] for the given `seconds_since_epoch`, unless the rule has not been activated, i.e.
    // the year is `self.start_year` and the start date has not been reached.
    pub(crate) fn for_timestamp(&self, seconds_since_epoch: i64) -> Option<Offset> {
        let (year, _, _) =
            iso::iso_from_fixed(super::EPOCH + (seconds_since_epoch / SECONDS_IN_UTC_DAY)).unwrap();

        // TODO
        let local_year = year;

        if local_year < self.start_year as i32 {
            return None;
        }

        let first = self.first_for_year(local_year);
        let second = self.second_for_year(local_year);

        if seconds_since_epoch < first.since {
            if local_year == self.start_year as i32 {
                return None;
            }
            Some(self.second_for_year(local_year - 1))
        } else if seconds_since_epoch < second.since {
            Some(first)
        } else {
            Some(second)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TZDB;

    #[test]
    fn test_weekday_before_year() {
        // Dec 31 1999 was a Friday
        assert_eq!(weekday_before_year(2000), 5);
        // Dec 31 2024 was a Tuesday
        assert_eq!(weekday_before_year(2025), 2);
        // Dec 31 1969 was a Wednesday
        assert_eq!(weekday_before_year(1970), 3);
        // Dec 31 3029 will be a Thursday
        assert_eq!(weekday_before_year(3030), 4);
        // Dec 31 0999 (proleptic) was a Tuesday
        assert_eq!(weekday_before_year(1000), 2);
    }

    #[track_caller]
    fn test_single_year(
        tz: &str,
        year: i32,
        (start_month, start_day, (start_before, start_after)): (u8, u8, (i8, i8)),
        (end_month, end_day, (end_before, end_after)): (u8, u8, (i8, i8)),
    ) {
        let rule = TZDB.get(tz).unwrap().final_rule.unwrap();

        // start_before doesn't actually happen
        assert_eq!(
            rule.for_date_time(
                year,
                start_month,
                start_day - start_before.div_euclid(24).unsigned_abs(),
                start_before.rem_euclid(24).unsigned_abs(),
                0,
                0
            ),
            Some(PossibleOffset::None),
        );

        // start_after happens exactly once
        assert!(matches!(
            rule.for_date_time(
                year,
                start_month,
                start_day - start_after.div_euclid(24).unsigned_abs(),
                start_after.rem_euclid(24).unsigned_abs(),
                0,
                0
            ),
            Some(PossibleOffset::Single(_))
        ));

        // end_before happens exactly once
        assert!(matches!(
            rule.for_date_time(
                year,
                end_month,
                end_day - end_before.div_euclid(24).unsigned_abs(),
                end_before.rem_euclid(24).unsigned_abs(),
                0,
                0
            ),
            Some(PossibleOffset::Single(_))
        ));

        // end_after happens again after falling back
        assert!(matches!(
            rule.for_date_time(
                year,
                end_month,
                end_day - end_after.div_euclid(24).unsigned_abs(),
                end_after.rem_euclid(24).unsigned_abs(),
                0,
                0
            ),
            Some(PossibleOffset::Ambiguous(_, _)),
        ));
    }

    #[test]
    fn test_los_angeles() {
        // This is a Wall rule
        // so the transition happens at the same time in the
        // previous timezone
        test_single_year(
            "America/Los_Angeles",
            2025,
            // The transition happens at 02:00 in the previous offset
            // and 03:00/01:00 in the next
            (3, 9, (2, 3)),
            (11, 2, (2, 1)),
        );
    }

    #[test]
    fn test_london() {
        // This is a Standard rule, so the transition happens
        // at the same time in the standard timezone
        test_single_year("Europe/London", 2017, (3, 26, (1, 2)), (10, 29, (2, 1)));
    }

    #[test]
    fn test_santiago() {
        // This is a Utc rule, so the transition happens
        // at the same time in UTC
        test_single_year(
            "America/Santiago",
            2025,
            // Note: this is in the southern hemisphere,
            // the transition start is later in the year
            (9, 7, (0, 1)),
            // The transition day is April 6, but the backwards
            // transition briefly puts us back in April 5, so we get a -1
            (4, 6, (0, -1)),
        );
    }
}
