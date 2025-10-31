use serde::{
    de::{self, Deserializer},
    Deserialize,
};
use serde_json::Value;

/// Deserializes a string or number into an `f64`, treating "undef" and null as `None`.
///
/// # Errors
///
/// Returns an error if the input is a string that cannot be parsed as a float or is an unsupported JSON type.
pub fn float<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into a generic JSON value
    let value: Option<Value> = Option::deserialize(deserializer)?;

    value.map_or_else(
        || Ok(None),
        |value| match value {
            Value::String(s) => {
                if s == "undef" {
                    Ok(None)
                } else {
                    s.parse::<f64>().map(Some).map_err(|_| {
                        de::Error::custom(format!("string can't be parsed to f64: {s}"))
                    })
                }
            }
            Value::Number(n) => n
                .as_f64()
                .ok_or_else(|| de::Error::custom("invalid number"))
                .map(Some),
            Value::Null => Ok(None),
            other => Err(de::Error::custom(format!(
                "unexpected type for float: {other}"
            ))),
        },
    )
}
