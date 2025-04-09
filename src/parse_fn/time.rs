use chrono::NaiveTime;
use serde::{self, Deserialize, Deserializer};

pub fn timestamp<'de, D>(deserializer: D) -> Result<Option<NaiveTime>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize an optional string
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = s {
        // Attempt to parse the string into a NaiveTime using the "%H:%M:%S" format.
        if let Ok(time) = NaiveTime::parse_from_str(&s, "%H:%M:%S") {
            return Ok(Some(time));
        }
        // Try parsing as a naive time with subseconds
        if let Ok(naive) = NaiveTime::parse_from_str(&s, "%H:%M:%S%.f") {
            return Ok(Some(naive));
        }
        // If parsing fails, log it, and set the field to None (this is not great).
        dbg!("Parsing time failed: {}", &s);
        Ok(None)
    } else {
        Ok(None)
    }
}
