use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{self, Deserialize, Deserializer};

pub fn fixed<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| DateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%#z"))
        .transpose()
        .map_err(serde::de::Error::custom)
}

pub fn naive<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S"))
        .transpose()
        .map_err(serde::de::Error::custom)
}

pub fn naive_with_subsec<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| NaiveDateTime::parse_from_str(&s, "%Y:%m:%d %H:%M:%S%.f"))
        .transpose()
        .map_err(serde::de::Error::custom)
}
