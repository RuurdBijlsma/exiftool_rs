use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde::{self, Deserialize, Deserializer};

/// Represents a datetime that can be one of several formats or an unparsed string.
#[derive(Debug, Clone)]
pub enum MaybeDateTime {
    /// A naive datetime without timezone information.
    Naive(NaiveDateTime),
    /// A datetime with a fixed timezone offset.
    Zoned(DateTime<FixedOffset>),
    /// A naive date without time information.
    Date(NaiveDate),
    /// The original string if parsing into a known format fails.
    NotParsed(String),
}

/// Deserializes a string by attempting to parse it into several possible datetime formats.
///
/// # Errors
///
/// Returns an error if the deserialized value is not a string or null.
pub fn guess_datetime<'de, D>(deserializer: D) -> Result<Option<MaybeDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    // Directly deserialize into an Option<String>
    let s: Option<String> = Deserialize::deserialize(deserializer)?;

    if let Some(s) = s {
        // Now 's' does not have extra quotes.
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f%:z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
        // continue with other parse attempts...
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%:z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f%#z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%#z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f") {
            return Ok(Some(MaybeDateTime::Naive(naive)));
        }
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S") {
            return Ok(Some(MaybeDateTime::Naive(naive)));
        }
        if let Ok(naive_date) = NaiveDate::parse_from_str(&s, "%Y:%m:%d") {
            return Ok(Some(MaybeDateTime::Date(naive_date)));
        }

        Ok(Some(MaybeDateTime::NotParsed(s)))
    } else {
        Ok(None)
    }
}