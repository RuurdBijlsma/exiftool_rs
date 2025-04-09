use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde::{self, Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub enum MaybeDateTime {
    Naive(NaiveDateTime),
    Zoned(DateTime<FixedOffset>),
    Date(NaiveDate),
    NotParsed(String),
}

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

