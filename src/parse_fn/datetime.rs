use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{self, Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub enum MaybeDateTime {
    Naive(NaiveDateTime),
    Zoned(DateTime<FixedOffset>),
}

pub fn possible_timezone<'de, D>(deserializer: D) -> Result<Option<MaybeDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    // First, deserialize an Option<String>
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        // Try parsing as a timezone-aware datetime
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%#z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
        // Try parsing as a naive datetime with subseconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f") {
            return Ok(Some(MaybeDateTime::Naive(naive)));
        }
        // Try parsing as a naive datetime without subseconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S") {
            return Ok(Some(MaybeDateTime::Naive(naive)));
        }
        // If all parsing attempts fail, return an error.
        Err(serde::de::Error::custom("invalid datetime format"))
    } else {
        // If there's no string, return None.
        Ok(None)
    }
}

pub fn with_timezone<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%#z"))
        .transpose()
        .map_err(serde::de::Error::custom)
}

pub fn naive<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    // First, deserialize an Option<String>
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        // Try parsing as a naive datetime with subseconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f") {
            return Ok(Some(naive));
        }
        // Try parsing as a naive datetime without subseconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S") {
            return Ok(Some(naive));
        }
        // If all parsing attempts fail, return an error.
        Err(serde::de::Error::custom("invalid datetime format"))
    } else {
        // If there's no string, return None.
        Ok(None)
    }
}
