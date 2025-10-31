use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

/// Deserializes a string, number, boolean, or sequence of strings into a single String, returning `None` on failure.
///
/// # Errors
///
/// Returns an error if the underlying deserializer encounters an I/O or syntax error.
pub fn string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    /// A visitor that can handle strings, numbers, booleans, and sequences, converting them to a String.
    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = String;

        /// Specifies what this visitor is expecting to parse.
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string, number, or sequence")
        }

        /// Handles boolean values by converting them to a string.
        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
            Ok(value.to_string())
        }

        /// Handles i64 values by converting them to a string.
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        /// Handles u64 values by converting them to a string.
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        /// Handles f64 values by converting them to a string.
        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        /// Handles string slices by creating an owned String.
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_owned())
        }

        /// Handles owned Strings by passing them through.
        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        /// Handles `None` values by returning an empty string.
        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(String::new())
        }

        /// Handles a sequence by joining its elements into a single comma-separated string.
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut elements = Vec::new();
            while let Some(element) = seq.next_element::<String>()? {
                elements.push(element);
            }
            Ok(elements.join(", "))
        }
    }

    // Attempt deserialization but return None instead of erroring on type mismatches.
    deserializer
        .deserialize_any(StringOrNumberVisitor)
        .map_or_else(|_| Ok(None), |s| Ok(Some(s)))
}
