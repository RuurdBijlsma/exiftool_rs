use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::{self, Deserialize, Deserializer};

pub(crate) fn parse_fixed_datetime<'de, D>(
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

pub(crate) fn parse_naive_datetime<'de, D>(
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

pub(crate) fn parse_naive_datetime_with_subsec<'de, D>(
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
