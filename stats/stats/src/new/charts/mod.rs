pub mod traits;
pub mod timespans;
pub mod duration;
pub mod counters;
pub mod lines;

use sea_orm::Set;
use entity::chart_data;
use entity::sea_orm_active_enums::ChartResolution;
use crate::new::charts::traits::Timespan;

pub use duration::TimespanDuration;

/// Some value for some time interval
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimespanValue<T, V> {
    pub timespan: T,
    pub value: V,
}

impl<T> TimespanValue<T, String> {
    pub fn with_zero_value(timespan: T) -> Self {
        Self {
            timespan,
            value: "0".to_string(),
        }
    }
}

impl<T: Ord> TimespanValue<T, String> {
    pub fn relevant_or_zero(self, current_timespan: T) -> Self {
        if self.timespan < current_timespan {
            Self::with_zero_value(current_timespan)
        } else {
            self
        }
    }
}

impl<T: Timespan + Clone> TimespanValue<T, String> {
    pub fn active_model(
        &self,
        chart_id: i32,
        min_blockscout_block: Option<i64>,
    ) -> chart_data::ActiveModel {
        chart_data::ActiveModel {
            id: Default::default(),
            chart_id: Set(chart_id),
            date: Set(self.timespan.clone().into_date()),
            value: Set(self.value.clone()),
            created_at: Default::default(),
            min_blockscout_block: Set(min_blockscout_block),
        }
    }
}

/// Marked as precise or approximate
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtendedTimespanValue<T, V> {
    pub timespan: T,
    pub value: V,
    pub is_approximate: bool,
}

impl<T, V> ExtendedTimespanValue<T, V> {
    pub fn from_timespan_value(v: TimespanValue<T, V>, is_approximate: bool) -> Self {
        Self {
            timespan: v.timespan,
            value: v.value,
            is_approximate,
        }
    }
}

impl<T, V> From<ExtendedTimespanValue<T, V>> for TimespanValue<T, V> {
    fn from(dv: ExtendedTimespanValue<T, V>) -> Self {
        Self {
            timespan: dv.timespan,
            value: dv.value,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolutionKind {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl From<ChartResolution> for ResolutionKind {
    fn from(value: ChartResolution) -> Self {
        match value {
            ChartResolution::Hour => ResolutionKind::Hour,
            ChartResolution::Day => ResolutionKind::Day,
            ChartResolution::Week => ResolutionKind::Week,
            ChartResolution::Month => ResolutionKind::Month,
            ChartResolution::Year => ResolutionKind::Year,
        }
    }
}

impl From<ResolutionKind> for ChartResolution {
    fn from(value: ResolutionKind) -> Self {
        match value {
            ResolutionKind::Hour => ChartResolution::Hour,
            ResolutionKind::Day => ChartResolution::Day,
            ResolutionKind::Week => ChartResolution::Week,
            ResolutionKind::Month => ChartResolution::Month,
            ResolutionKind::Year => ChartResolution::Year,
        }
    }
}

impl From<ResolutionKind> for String {
    fn from(value: ResolutionKind) -> Self {
        match value {
            ResolutionKind::Hour => "HOUR",
            ResolutionKind::Day => "DAY",
            ResolutionKind::Week => "WEEK",
            ResolutionKind::Month => "MONTH",
            ResolutionKind::Year => "YEAR",
        }
            .into()
    }
}