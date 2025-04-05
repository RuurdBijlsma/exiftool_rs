use serde::{
    de::{self, Deserializer, Unexpected, Visitor},
    Deserialize,
};
use std::fmt;

// Helper function to deserialize either a string or a number into a String
pub fn string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumberVisitor;

    impl<'de> Visitor<'de> for StringOrNumberVisitor {
        type Value = String; // We want to produce a String

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a number")
        }

        // Handle JSON strings
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_owned())
        }

        // Handle JSON strings (owned)
        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        // Handle JSON numbers (integers) - Convert to String
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        // Handle JSON numbers (unsigned integers) - Convert to String
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }

        // Handle JSON numbers (floats) - Convert to String
        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // Decide on float formatting if needed, simple .to_string() is often fine
            Ok(value.to_string())
        }

        // Optional: Handle booleans if they might occur, and you want them as strings
        // fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        // where
        //     E: de::Error,
        // {
        //     Ok(value.to_string())
        // }

        // You might add more visit_... methods if other JSON types are possible (like bool)
    }

    // Tell the deserializer to use our visitor to process the data
    Ok(Some(deserializer.deserialize_any(StringOrNumberVisitor)?))
}
