use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Duration, Timelike};
use crate::new::charts::duration::TimespanDuration;
use crate::new::charts::ResolutionKind;
use crate::new::charts::traits::Timespan;

pub struct Hour(NaiveDateTime);

// impl Hour {
//     pub const MAX: Hour = Self(NaiveDateTime::MAX);
//     pub const MIN: Hour = Self(NaiveDateTime::MIN);
// }

impl Timespan for Hour {
    fn enum_variant() -> ResolutionKind {
        ResolutionKind::Hour
    }

    fn from_date(date: NaiveDate) -> Self {
        Self(NaiveDateTime::new(date, NaiveTime::MIN))
    }

    fn into_date(self) -> NaiveDate {
        self.0.date()
    }

    fn saturating_next_timespan(&self) -> Self {
        Self(
            self.0 + Duration::hours(1)
        )
    }

    fn saturating_previous_timespan(&self) -> Self {
        Self(
            self.0 - Duration::hours(1)
        )
    }

    fn saturating_start_timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        Self::new(self.0).0.and_utc()
    }

    fn saturating_add(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized,
    {
        let dur = Duration::hours(duration.repeats() as _);
        Self(self.0 + dur)
    }

    fn saturating_sub(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized,
    {
        let dur = Duration::hours(duration.repeats() as _);
        Self(self.0 - dur)
    }
}

impl Hour {
    pub fn new(time: NaiveDateTime) -> Self {
        Self(time.with_minute(0).unwrap().with_second(0).unwrap())
    }
}

// impl_into_string_timespan_value!(Hour, i128);
// impl_into_string_timespan_value!(Hour, u128);
// impl_into_string_timespan_value!(Hour, Decimal);