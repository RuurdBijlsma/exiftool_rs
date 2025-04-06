use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;

pub fn floats<'de, D>(deserializer: D) -> Result<Option<Vec<f64>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct SpaceSeparatedFloatsVisitor;

    impl<'de> Visitor<'de> for SpaceSeparatedFloatsVisitor {
        type Value = Option<Vec<f64>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing space-separated floating-point numbers")
        }

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

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = Option::<String>::deserialize(deserializer)?;
            match s {
                Some(s) => {
                    let result: Result<Vec<f64>, _> =
                        s.split_whitespace().map(f64::from_str).collect();
                    result.map(Some).map_err(de::Error::custom)
                }
                None => Ok(None),
            }
        }
    }

    deserializer.deserialize_option(SpaceSeparatedFloatsVisitor)
}
