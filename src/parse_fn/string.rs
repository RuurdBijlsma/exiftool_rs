use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;

// Helper function to deserialize either a string, number, or sequence into a String
pub fn string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string, number, or sequence")
        }

        // Handle other unexpected types gracefully
        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
            Ok(value.to_string())
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_owned())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(String::new())
        }

        // New: Handle sequences by joining elements with commas
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

    // Attempt deserialization but return None instead of erroring
    match deserializer.deserialize_any(StringOrNumberVisitor) {
        Ok(s) => Ok(Some(s)),
        Err(_) => Ok(None), // Fallback to None on any error
    }
}
