use chrono::NaiveDate;
use serde::{self, Deserialize, Deserializer};
use serde_json::Value;

/// Deserializes a string in "%Y:%m:%d" format into a `NaiveDate`, gracefully skipping null or number values.
///
/// # Errors
///
/// Returns an error if the input is a string that does not match the required format or is an unsupported JSON type.
pub fn date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into a generic JSON value
    let value: Option<Value> = Option::deserialize(deserializer)?;

    value.map_or_else(
        || Ok(None),
        |value| match value {
            Value::String(s) => {
                // Try parsing the string as a NaiveDate
                NaiveDate::parse_from_str(&s, "%Y:%m:%d")
                    .map(Some)
                    .map_err(|_| serde::de::Error::custom(format!("invalid date format: {s}")))
            }
            Value::Number(_) | Value::Null => Ok(None), // Gracefully skip numbers
            other => Err(serde::de::Error::custom(format!(
                "unexpected type for date: {other:?}"
            ))),
        },
    )
}
