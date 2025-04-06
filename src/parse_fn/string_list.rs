use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    Vec(Vec<String>),
    String(String),
}

pub fn string_list<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let helper: Option<StringOrVec> = Option::deserialize(deserializer)?;
    match helper {
        Some(StringOrVec::Vec(v)) => Ok(Some(v)),
        Some(StringOrVec::String(s)) => Ok(Some(vec![s])),
        None => Ok(None),
    }
}
