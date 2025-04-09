use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use serde::{self, Deserialize, Deserializer};
use serde_json::Value;

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
    // Deserialize into a string, even if the json value is a number.
    let v: Option<Value> = Deserialize::deserialize(deserializer)?;
    let s = v.map(|v| v.to_string());

    if let Some(s) = s {
        // Try parsing with full subseconds and offset like +03:00
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f%:z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
        // Try parsing with offset but without subseconds
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%:z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
        // Try Windows-style offset (e.g., +0300)
        if let Ok(zoned) = DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f%#z") {
            return Ok(Some(MaybeDateTime::Zoned(zoned)));
        }
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
        // Try parsing as a naive date without time
        if let Ok(naive_date) = NaiveDate::parse_from_str(&s, "%Y:%m:%d") {
            return Ok(Some(MaybeDateTime::Date(naive_date)));
        }

        // All parsing failed
        Ok(Some(MaybeDateTime::NotParsed(s)))
    } else {
        Ok(None)
    }
}
