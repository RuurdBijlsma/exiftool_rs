use serde::de::{Deserializer, Error, Visitor};
use std::fmt;

pub fn permissive<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    struct PermissiveU32Visitor;

    impl Visitor<'_> for PermissiveU32Visitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("any value that can be loosely interpreted as a u32")
        }

        // For any other type, return None
        fn visit_bool<E: Error>(self, _: bool) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
            Ok(if value >= 0 { Some(value as u32) } else { None })
        }

        fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
            Ok(Some(value as u32))
        }

        fn visit_f64<E: Error>(self, value: f64) -> Result<Self::Value, E> {
            Ok(if value >= 0.0 && value <= u32::MAX as f64 {
                Some(value as u32)
            } else {
                None
            })
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            // First try parsing directly
            if let Ok(num) = value.parse::<u32>() {
                return Ok(Some(num));
            }

            // Then try splitting and taking first part
            let first_part = value.split_whitespace().next().unwrap_or("");
            Ok(first_part.parse::<u32>().ok())
        }

        fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
            self.visit_str(&value)
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(PermissiveU32Visitor)
}
