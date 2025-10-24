//! Time scope definitions for the MIA storage system
//!
//! This module defines the TimeScope enum and its implementations
//! for time-based queries.

/// Time scope for queries
///
/// Defines different time ranges for filtering data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeScope {
    /// Current chat context
    CurrentChat,
    /// Today (last 24 hours)
    Today,
    /// This week (last 7 days)
    ThisWeek,
    /// This month (last 30 days)
    ThisMonth,
    /// This quarter (last 90 days)
    ThisQuarter,
    /// This year (last 365 days)
    ThisYear,
    /// Last year
    LastYear,
    /// Custom time range
    Custom(i64, i64), // start_ms, end_ms
}

impl TimeScope {
    /// Get the start and end timestamps for this time scope
    ///
    /// # Arguments
    /// * `current_time_ms` - Current timestamp in milliseconds
    ///
    /// # Returns
    /// * (start_timestamp_ms, end_timestamp_ms)
    pub fn get_time_range(&self, current_time_ms: i64) -> (i64, i64) {
        match self {
            TimeScope::CurrentChat => (0, i64::MAX), // Will be filtered by chat ID instead
            TimeScope::Today => {
                let day_ms = 24 * 60 * 60 * 1000;
                (current_time_ms - day_ms, current_time_ms)
            }
            TimeScope::ThisWeek => {
                let week_ms = 7 * 24 * 60 * 60 * 1000;
                (current_time_ms - week_ms, current_time_ms)
            }
            TimeScope::ThisMonth => {
                let month_ms = 30 * 24 * 60 * 60 * 1000;
                (current_time_ms - month_ms, current_time_ms)
            }
            TimeScope::ThisQuarter => {
                let quarter_ms = 90 * 24 * 60 * 60 * 1000;
                (current_time_ms - quarter_ms, current_time_ms)
            }
            TimeScope::ThisYear => {
                let year_ms = 365 * 24 * 60 * 60 * 1000;
                (current_time_ms - year_ms, current_time_ms)
            }
            TimeScope::LastYear => {
                let year_ms = 365 * 24 * 60 * 60 * 1000;
                (current_time_ms - 2 * year_ms, current_time_ms - year_ms)
            }
            TimeScope::Custom(start, end) => (*start, *end),
        }
    }

    /// Check if a timestamp falls within this time scope
    ///
    /// # Arguments
    /// * `timestamp_ms` - Timestamp to check in milliseconds
    /// * `current_time_ms` - Current timestamp in milliseconds
    ///
    /// # Returns
    /// * true if timestamp is within the time scope
    pub fn contains(&self, timestamp_ms: i64, current_time_ms: i64) -> bool {
        let (start, end) = self.get_time_range(current_time_ms);
        timestamp_ms >= start && timestamp_ms <= end
    }
}
