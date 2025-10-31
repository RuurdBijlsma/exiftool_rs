use serde::{Deserialize, Deserializer};

/// An internal enum to handle deserialization of either a single string or a vector of strings.
#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    /// Represents a vector of strings.
    Vec(Vec<String>),
    /// Represents a single string.
    String(String),
}

/// Deserializes a value that can be either a single string or a list of strings into a `Vec<String>`.
///
/// # Errors
///
/// Returns an error if the value being deserialized is not a string, an array of strings, or null.
pub fn string_list<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let helper: Option<StringOrVec> = Option::deserialize(deserializer)?;
    match helper {
        Some(StringOrVec::Vec(v)) => Ok(Some(v)),
        Some(StringOrVec::String(s)) => Ok(Some(vec![s])),
        None => Ok(None),
    }
}