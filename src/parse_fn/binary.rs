use serde::{Deserialize, Deserializer};

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryDataField {
    bytes: usize,
    field_name: String,
}

impl BinaryDataField {
    pub fn extract(&self) -> Result<Vec<u8>, std::io::Error> {
        println!("[EXTRACT] bytes: {}, field_name: {}", self.bytes, self.field_name);
        Ok(vec![1, 2, 3])
    }
}

pub fn binary<'de, D>(deserializer: D) -> Result<Option<BinaryDataField>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let re = regex::Regex::new(r"\(Binary data (\d+) bytes, use -b option to extract\)")
        .map_err(serde::de::Error::custom)?;

    let caps = match re.captures(&s) {
        Some(caps) => caps,
        None => return Ok(None),
    };

    let bytes = caps[1].parse::<usize>().map_err(serde::de::Error::custom)?;

    Ok(Some(BinaryDataField {
        bytes,
        field_name: "BlueTRC".to_string(),
    }))
}
