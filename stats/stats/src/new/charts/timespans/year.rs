use std::{cmp::Ordering, fmt::Debug};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use crate::new::charts::duration::TimespanDuration;
use crate::new::charts::traits::{ConsistsOf, ResolutionKind, Timespan};
use super::Month;

/// Year number within range of `chrono::NaiveDate`
#[derive(Copy, Clone)]
pub struct Year(i32);

impl PartialEq for Year {
    fn eq(&self, other: &Self) -> bool {
        self.number_within_naive_date() == other.number_within_naive_date()
    }
}

impl Eq for Year {}

impl PartialOrd for Year {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Year {
    fn cmp(&self, other: &Self) -> Ordering {
        self.number_within_naive_date()
            .cmp(&other.number_within_naive_date())
    }
}

impl Debug for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Year")
            .field(&self.number_within_naive_date())
            .finish()
    }
}

impl Year {
    /// First day of the year or `NaiveDate::MIN`
    pub fn saturating_first_day(&self) -> NaiveDate {
        NaiveDate::from_yo_opt(self.0, 1).unwrap_or(NaiveDate::MIN)
    }

    /// Number of the year (within `NaiveDate::MIN`)
    pub fn number_within_naive_date(self) -> i32 {
        self.clamp_by_naive_date_range().0
    }

    fn clamp_by_naive_date_range(self) -> Self {
        if NaiveDate::from_yo_opt(self.0, 1).is_some() {
            self
        } else if self.0 > 0 {
            Self::from_date(NaiveDate::MAX)
        } else {
            Self::from_date(NaiveDate::MIN)
        }
    }
}

impl Timespan for Year {
    fn from_date(date: NaiveDate) -> Self {
        Self(date.year())
    }

    fn into_date(self) -> NaiveDate {
        self.clamp_by_naive_date_range().saturating_first_day()
    }

    fn enum_variant() -> ResolutionKind {
        ResolutionKind::Year
    }

    fn saturating_start_timestamp(&self) -> DateTime<Utc> {
        self.saturating_first_day().saturating_start_timestamp()
    }

    fn saturating_add(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized,
    {
        let add_years = duration.repeats().try_into().unwrap_or(i32::MAX);
        Self(self.number_within_naive_date().saturating_add(add_years)).clamp_by_naive_date_range()
    }

    fn saturating_sub(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized,
    {
        let sub_years: i32 = duration.repeats().try_into().unwrap_or(i32::MAX);
        Self(self.number_within_naive_date().saturating_sub(sub_years)).clamp_by_naive_date_range()
    }
}

impl ConsistsOf<NaiveDate> for Year {
    fn from_smaller(date: NaiveDate) -> Self {
        Year::from_date(date)
    }

    fn into_smaller(self) -> NaiveDate {
        Year::into_date(self)
    }
}

impl ConsistsOf<Month> for Year {
    fn from_smaller(month: Month) -> Self {
        Year::from_date(month.into_date())
    }

    fn into_smaller(self) -> Month {
        Month::from_date(Year::into_date(self))
    }
}

// impl_into_string_timespan_value!(Year, i64);
// impl_into_string_timespan_value!(Year, f64);
// impl_into_string_timespan_value!(Year, Decimal);
