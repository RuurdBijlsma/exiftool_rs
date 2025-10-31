use serde::de::{self, Deserializer, SeqAccess, Unexpected, Visitor};
use std::fmt;

/// Deserializes a number or a potentially nested array of numbers into a flattened array of u64.
///
/// # Errors
///
/// Returns an error if the deserialized value is not a number, an array of numbers, or a nested array of numbers.
pub fn to_array<'de, D>(deserializer: D) -> Result<Option<Vec<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    /// A visitor to deserialize directory item lengths which can be a single number or an array.
    struct DirectoryItemLengthVisitor;

    impl<'de> Visitor<'de> for DirectoryItemLengthVisitor {
        type Value = Vec<u64>;

        /// Specifies what this visitor is expecting to parse.
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a number or an array of numbers (or nested arrays) representing directory item lengths")
        }

        /// Handles the case where the JSON is a single u64 number.
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value])
        }

        /// Handles the case where the JSON is a sequence of numbers or nested arrays of numbers.
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values = Vec::new();

            // Iterate over each element in the top-level array.
            while let Some(elem) = seq.next_element::<serde_json::Value>()? {
                match elem {
                    // If the element is a number, extract it.
                    serde_json::Value::Number(n) => {
                        let num = n.as_u64().ok_or_else(|| {
                            de::Error::invalid_value(Unexpected::Other("non-u64 number"), &self)
                        })?;
                        values.push(num);
                    }
                    // If the element is an array, iterate over it.
                    serde_json::Value::Array(arr) => {
                        for inner in arr {
                            let num = match inner {
                                serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| {
                                    de::Error::invalid_value(
                                        Unexpected::Other("non-u64 number in nested array"),
                                        &self,
                                    )
                                })?,
                                _ => {
                                    return Err(de::Error::invalid_type(
                                        Unexpected::Other("non-number in nested array"),
                                        &self,
                                    ))
                                }
                            };
                            values.push(num);
                        }
                    }
                    // Any other type is unexpected.
                    _ => {
                        return Err(de::Error::invalid_type(
                            Unexpected::Other("non-number/non-array element"),
                            &self,
                        ))
                    }
                }
            }
            Ok(values)
        }
    }

    Ok(Some(
        deserializer.deserialize_any(DirectoryItemLengthVisitor)?,
    ))
}