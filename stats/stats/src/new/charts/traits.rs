use std::fmt::Display;
use std::ops::Range;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use entity::sea_orm_active_enums::ChartType;
use crate::new::charts::duration::TimespanDuration;
pub use super::{ResolutionKind, TimespanValue};

pub trait NamedProperty {
    /// Name of this data that represents its contents.
    fn property_name() -> String;
}

pub trait ChartProperties: Sync + NamedProperty {
    /// Combination name + resolution must be unique for each chart
    type Resolution: Timespan;

    fn chart_type() -> ChartType;

    fn resolution() -> ResolutionKind {
        Self::Resolution::enum_variant()
    }

    /// Expected but not guaranteed to be unique for each chart
    fn key() -> ChartKey {
        ChartKey::new(Self::property_name(), Self::resolution())
    }

    fn missing_date_policy() -> MissingDatePolicy {
        MissingDatePolicy::FillZero
    }

    /// Number of last values that are considered approximate.
    /// (ordered by time)
    ///
    /// E.g. how many end values should be recalculated on (kinda)
    /// lazy update (where `get_update_start` is retrieved successfully).
    /// Also controls marking points as approximate when returning data.
    ///
    /// ## Value
    ///
    /// Usually set to 1 for line charts. Also, data for portion of the
    /// (latest) timeframe has to be recalculated on the next timespan.
    ///
    /// I.e. for number of blocks per day, stats for current day (0) are
    /// not complete because blocks will be produced till the end of the day.
    /// ```text
    ///    |===|=  |
    /// day -1   0
    /// ```
    ///
    /// ## Edge case
    ///
    /// If an update time is exactly at the start of timespan (e.g. midnight),
    /// one less point is considered approximate, because we've got full data
    /// for one timespan.
    fn approximate_trailing_points() -> u64 {
        if Self::chart_type() == ChartType::Counter {
            // there's only one value in counter
            0
        } else {
            1
        }
    }
}

pub trait Timespan {
    /// Converting type into runtime enum variant
    fn enum_variant() -> ResolutionKind;

    /// Construct the timespan from a date within the timespan.
    ///
    /// Note that `from` is not a reversible function.
    /// I.e. for some date `d`, `Timespan::from_date(d).into_date() != d`
    fn from_date(date: NaiveDate) -> Self;

    /// Convert the timespan into a corresponding date
    /// to store in database.
    ///
    /// Using `from_date` on resulting date will always result in equal timespan.
    fn into_date(self) -> NaiveDate;

    /// Get the next interval right after the current one (saturating)
    fn saturating_next_timespan(&self) -> Self
    where
        Self: Sized,
    {
        self.saturating_add(TimespanDuration::from_timespan_repeats(1))
    }

    /// Get the interval right before the current one (saturating)
    fn saturating_previous_timespan(&self) -> Self
    where
        Self: Sized,
    {
        self.saturating_sub(TimespanDuration::from_timespan_repeats(1))
    }

    /// Extract the start of given timespan as UTC timestamp
    fn saturating_start_timestamp(&self) -> DateTime<Utc>;

    /// Represent the timespan as UTC timestamp range
    fn into_time_range(self) -> Range<DateTime<Utc>>
    where
        Self: Sized,
    {
        self.saturating_start_timestamp()..self.saturating_next_timespan().saturating_start_timestamp()
    }

    fn saturating_add(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized;

    fn saturating_sub(&self, duration: TimespanDuration<Self>) -> Self
    where
        Self: Sized;
}

// if for some rare reason trait is needed
// (currently only for `ZeroTimespanValue` impl)
pub trait TimespanValueTrait {
    type Timespan;
    type Value;

    fn parts(&self) -> (&Self::Timespan, &Self::Value);
    fn into_parts(self) -> (Self::Timespan, Self::Value);
    fn from_parts(t: Self::Timespan, v: Self::Value) -> Self;
}

impl<T, V> TimespanValueTrait for TimespanValue<T, V> {
    type Timespan = T;
    type Value = V;

    fn parts(&self) -> (&Self::Timespan, &Self::Value) {
        (&self.timespan, &self.value)
    }

    fn into_parts(self) -> (Self::Timespan, Self::Value) {
        (self.timespan, self.value)
    }

    fn from_parts(t: Self::Timespan, v: Self::Value) -> Self {
        Self {
            timespan: t,
            value: v,
        }
    }
}

// generic to unify the parameters
pub trait ZeroTimespanValue<T>: TimespanValueTrait<Timespan = T> + Sized
where
    Self::Timespan: Ord,
{
    fn with_zero_value(timespan: Self::Timespan) -> Self;

    fn relevant_or_zero(self, current_timespan: Self::Timespan) -> Self {
        if self.parts().0 < &current_timespan {
            Self::with_zero_value(current_timespan)
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChartKey {
    name: String,
    resolution: ResolutionKind,
}

impl ChartKey {
    pub fn new(name: String, resolution: ResolutionKind) -> Self {
        Self { name, resolution }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn resolution(&self) -> &ResolutionKind {
        &self.resolution
    }
}

impl From<ChartKey> for String {
    fn from(value: ChartKey) -> Self {
        value.to_string()
    }
}

impl Display for ChartKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let resolution_string: String = self.resolution.into();
        write!(f, "{}_{}", self.name, resolution_string)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingDatePolicy {
    FillZero,
    FillPrevious,
}

// https://github.com/rust-lang/rfcs/issues/2758
macro_rules! impl_zero_timespan_value_for_zero_value {
    ($value:ty) => {
        impl<T: Ord> ZeroTimespanValue<T> for TimespanValue<T, $value> {
            fn with_zero_value(timespan: T) -> Self {
                Self {
                    timespan,
                    value: <$value as ::rust_decimal::prelude::Zero>::zero(),
                }
            }
        }
    };
}

impl_zero_timespan_value_for_zero_value!(i64);
impl_zero_timespan_value_for_zero_value!(f64);
impl_zero_timespan_value_for_zero_value!(Decimal);

impl<T: Ord> ZeroTimespanValue<T> for TimespanValue<T, String> {
    fn with_zero_value(timespan: T) -> Self {
        Self {
            timespan,
            value: "0".to_string(),
        }
    }
}

/// Indicates that this timespan can be broken into (multiple) `SmallerTimespan`s.
///
/// Examples (types may not be called exactly like this):
/// - `Week: ConsistsOf<Day>`
/// - `Month: ConsistsOf<Day>`
/// - `Year: ConsistsOf<Day>`
/// - `Year: ConsistsOf<Month>`
///
/// but not `Year: ConsistsOf<Week>` or `Month: ConsistsOf<Week>` because week borders might not align
/// with years'/months' borders.
pub trait ConsistsOf<SmallerTimespan> {
    /// Construct the timespan containing the smaller timespan.
    ///
    /// Note that `from` is usually not a reversible function.
    /// I.e. for some smaller timespan `s`, `T::from_smaller(s).into_smaller() != s`
    /// can be true
    fn from_smaller(smaller: SmallerTimespan) -> Self;
    /// Convert the timespan into a corresponding smaller timespan (contained in `self`).
    fn into_smaller(self) -> SmallerTimespan;
}
