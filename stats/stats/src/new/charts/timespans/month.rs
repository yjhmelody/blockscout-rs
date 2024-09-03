use std::cmp::Ordering;

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use crate::new::charts::duration::TimespanDuration;
use crate::new::charts::ResolutionKind;
use crate::new::charts::traits::{ConsistsOf, Timespan};

#[derive(Copy, Clone)]
pub struct Month {
    date_in_month: NaiveDate,
}

impl std::fmt::Debug for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Month")
            .field("year", &self.date_in_month.year())
            .field("month", &self.date_in_month.month())
            .finish()
    }
}

impl PartialEq for Month {
    fn eq(&self, other: &Self) -> bool {
        self.saturating_first_day() == other.saturating_first_day()
    }
}

impl Eq for Month {}

impl PartialOrd for Month {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Month {
    fn cmp(&self, other: &Self) -> Ordering {
        self.saturating_first_day()
            .cmp(&other.saturating_first_day())
    }
}

impl Month {
    fn saturating_first_month_day(date: NaiveDate) -> NaiveDate {
        date.with_day(1).unwrap_or(NaiveDate::MIN)
    }

    pub fn saturating_first_day(&self) -> NaiveDate {
        Self::saturating_first_month_day(self.date_in_month)
    }
}

impl Timespan for Month {
    fn enum_variant() -> ResolutionKind {
        ResolutionKind::Month
    }

    fn from_date(date: NaiveDate) -> Self {
        Self {
            date_in_month: Self::saturating_first_month_day(date),
        }
    }

    fn into_date(self) -> NaiveDate {
        Self::saturating_first_month_day(self.date_in_month)
    }

    fn saturating_start_timestamp(&self) -> DateTime<Utc> {
        self.saturating_first_day().saturating_start_timestamp()
    }

    fn saturating_add(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized,
    {
        let result_month_date = self
            .date_in_month
            .checked_add_months(chrono::Months::new(
                duration.repeats().try_into().unwrap_or(u32::MAX),
            ))
            .unwrap_or(NaiveDate::MAX);
        Self::from_date(result_month_date)
    }

    fn saturating_sub(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized,
    {
        let result_month_date = self
            .date_in_month
            .checked_sub_months(chrono::Months::new(
                duration.repeats().try_into().unwrap_or(u32::MAX),
            ))
            .unwrap_or(NaiveDate::MIN);
        Self::from_date(result_month_date)
    }
}

impl ConsistsOf<NaiveDate> for Month {
    fn from_smaller(date: NaiveDate) -> Self {
        Month::from_date(date)
    }

    fn into_smaller(self) -> NaiveDate {
        Month::into_date(self)
    }
}

// impl_into_string_timespan_value!(Month, i64);
// impl_into_string_timespan_value!(Month, f64);
// impl_into_string_timespan_value!(Month, Decimal);
