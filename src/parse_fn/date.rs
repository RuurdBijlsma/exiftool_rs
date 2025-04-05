use chrono::NaiveDate;
use serde::{self, Deserialize, Deserializer};
use serde_json::Value;

pub fn date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into a generic JSON value
    let value: Option<Value> = Option::deserialize(deserializer)?;

    if let Some(value) = value {
        match value {
            Value::String(s) => {
                // Try parsing the string as a NaiveDate
                NaiveDate::parse_from_str(&s, "%Y:%m:%d")
                    .map(Some)
                    .map_err(|_| serde::de::Error::custom(format!("invalid date format: {}", s)))
            }
            Value::Number(_) => Ok(None), // Gracefully skip numbers
            Value::Null => Ok(None),
            other => Err(serde::de::Error::custom(format!(
                "unexpected type for date: {:?}",
                other
            ))),
        }
    } else {
        Ok(None)
    }
}
