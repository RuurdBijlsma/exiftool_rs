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
    let s: Option<String> = Option::deserialize(deserializer)?;
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

        // All parsing failed
        dbg!("Parsing datetime failed: {}", &s);
        Ok(None)
    } else {
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
        // Handle known invalid zero datetime
        if s.trim().len() == 0 {
            return Ok(None);
        }
        if s.trim() == "0000:00:00 00:00:00" {
            let fallback = chrono::NaiveDate::from_ymd_opt(0, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            return Ok(Some(fallback));
        }

        // Try parsing as a naive datetime with subseconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f") {
            return Ok(Some(naive));
        }
        // Try parsing as a naive datetime without subseconds
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S") {
            return Ok(Some(naive));
        }
        // Try parsing as a naive datetime without time
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y:%m:%d") {
            return Ok(Some(naive));
        }
        // If all parsing attempts fail, just make it None;
        dbg!("Parsing datetime failed: {}", &s);
        Ok(None)
    } else {
        // If there's no string, return None.
        Ok(None)
    }
}
