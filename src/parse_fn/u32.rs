use serde::de::{Deserializer, Error, Visitor};
use std::fmt;

/// Deserializes various JSON types (numbers, strings) into a `u32` where possible, returning `None` otherwise.
///
/// # Errors
///
/// Returns an error if the underlying deserializer encounters a syntax or I/O error.
pub fn permissive<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    /// A visitor that permissively deserializes various types into an `Option<u32>`.
    struct PermissiveU32Visitor;

    impl Visitor<'_> for PermissiveU32Visitor {
        type Value = Option<u32>;

        /// Specifies what this visitor is expecting to parse.
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("any value that can be loosely interpreted as a u32")
        }

        /// Handles boolean values by returning `None`.
        fn visit_bool<E: Error>(self, _: bool) -> Result<Self::Value, E> {
            Ok(None)
        }

        /// Handles i64 values by converting them to `u32` if they are non-negative.
        fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
            Ok(if value >= 0 { Some(value as u32) } else { None })
        }

        /// Handles u64 values by casting them to `u32`.
        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(Some(value as u32))
        }

        /// Handles f64 values by converting them to `u32` if they are within the valid range.
        fn visit_f64<E: Error>(self, value: f64) -> Result<Self::Value, E> {
            Ok(if value >= 0.0 && value <= f64::from(u32::MAX) {
                Some(value as u32)
            } else {
                None
            })
        }

        /// Handles string slices by attempting to parse them as `u32`.
        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            // First try parsing directly
            if let Ok(num) = value.parse::<u32>() {
                return Ok(Some(num));
            }

            // Then try splitting and taking first part
            let first_part = value.split_whitespace().next().unwrap_or("");
            Ok(first_part.parse::<u32>().ok())
        }

        /// Handles owned Strings by delegating to `visit_str`.
        fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
            self.visit_str(&value)
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(PermissiveU32Visitor)
}
