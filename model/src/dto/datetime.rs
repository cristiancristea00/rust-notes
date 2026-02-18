//! Custom date-time wrapper with human-readable serialisation.

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Serialize, Serializer};
use std::ops::Deref;

/// A UTC timestamp that serialises as a human-readable string of the form
/// `Friday, 3rd August 2034, 12:45:34 PM UTC`.
///
/// Implements [`Deref`] to [`DateTime<Utc>`] for transparent access to all
/// chrono methods, and [`From<DateTime<Utc>>`] for convenient construction.
#[derive(Debug, Clone)]
pub struct FormattedDateTime(DateTime<Utc>);

impl From<DateTime<Utc>> for FormattedDateTime {
    /// Wraps a [`DateTime<Utc>`] in a [`FormattedDateTime`].
    fn from(date_time_utc: DateTime<Utc>) -> Self {
        Self(date_time_utc)
    }
}

impl Deref for FormattedDateTime {
    type Target = DateTime<Utc>;

    /// Returns a reference to the inner [`DateTime<Utc>`].
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for FormattedDateTime {
    /// Serialises the timestamp as a human-readable string.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let day = self.day();

        let formatted = format!(
            "{weekday}, {day}{suffix} {month} {year}, {hour}:{minute:02}:{second:02} {ampm} UTC",
            weekday = self.format("%A"),
            suffix = ordinal_suffix(day),
            month = self.format("%B"),
            year = self.year(),
            hour = self.format("%I"),
            minute = self.minute(),
            second = self.second(),
            ampm = self.format("%p"),
        );

        serializer.serialize_str(&formatted)
    }
}

/// Returns the English ordinal suffix for a given day of the month.
///
/// Handles the 11th, 12th, and 13th edge cases correctly before falling
/// back to the standard `st`, `nd`, `rd`, and `th` suffixes.
fn ordinal_suffix(day: u32) -> &'static str {
    match (day % 100, day % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    }
}
