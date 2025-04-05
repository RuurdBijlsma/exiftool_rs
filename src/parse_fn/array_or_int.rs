use serde::{
    de::{self, Deserializer, SeqAccess, Unexpected, Visitor},
    Deserialize,
};
use std::fmt;

pub fn to_array<'de, D>(deserializer: D) -> Result<Option<Vec<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct DirectoryItemLengthVisitor;

    impl<'de> Visitor<'de> for DirectoryItemLengthVisitor {
        type Value = Vec<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number or an array of numbers (or nested arrays) representing directory item lengths")
        }

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

        // Optionally, if the JSON was just a single number (not wrapped in an array),
        // handle that case as well.
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value])
        }
    }

    Ok(Some(deserializer.deserialize_any(DirectoryItemLengthVisitor)?))
}
