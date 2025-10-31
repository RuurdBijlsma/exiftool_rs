use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;

/// Deserializes a string of space-separated floating-point numbers into a vector of f64.
///
/// # Errors
///
/// Returns an error if the string contains any part that cannot be parsed into an f64.
pub fn floats<'de, D>(deserializer: D) -> Result<Option<Vec<f64>>, D::Error>
where
    D: Deserializer<'de>,
{
    /// A visitor to handle deserializing a space-separated string of floats.
    struct SpaceSeparatedFloatsVisitor;

    impl<'de> Visitor<'de> for SpaceSeparatedFloatsVisitor {
        type Value = Option<Vec<f64>>;

        /// Specifies what this visitor is expecting to parse.
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string containing space-separated floating-point numbers")
        }

        /// Handles the case where the input is a string.
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .split_whitespace()
                .map(f64::from_str)
                .collect::<Result<Vec<f64>, _>>()
                .map(Some)
                .map_err(de::Error::custom)
        }

        /// Handles the case where the input is `None`.
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        /// Handles the case where the input is `Some(T)`.
        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = Option::<String>::deserialize(deserializer)?;
            s.map_or_else(
                || Ok(None),
                |s| {
                    let result: Result<Vec<f64>, _> =
                        s.split_whitespace().map(f64::from_str).collect();
                    result.map(Some).map_err(de::Error::custom)
                },
            )
        }
    }

    deserializer.deserialize_option(SpaceSeparatedFloatsVisitor)
}