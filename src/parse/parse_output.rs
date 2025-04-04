use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_path_to_error as spte;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExifParseError {
    #[error("Deserialization error at path '{path}': {source}")]
    Deserialization {
        path: String,
        source: serde_json::Error,
    },
}

impl From<spte::Error<serde_json::Error>> for ExifParseError {
    fn from(err: spte::Error<serde_json::Error>) -> Self {
        ExifParseError::Deserialization {
            path: err.path().to_string(),
            source: err.into_inner(),
        }
    }
}

/// Parses JSON output from ExifTool into the specified type
pub fn parse_output<T>(output: &Value) -> Result<T, ExifParseError>
where
    T: DeserializeOwned,
{
    let exif = spte::deserialize(output)?;
    Ok(exif)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execute::execute;
    use crate::parse::output_type::ExifOutput;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestError {}

    #[tokio::test]
    async fn test_successful_deserialization() {
        let filename = "IMG_20170801_162043.jpg";
        let json = execute(&[&format!("test_data/{}", filename)])
            .await
            .unwrap();

        let result: Result<ExifOutput, _> = parse_output(&json);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);
        let item = &parsed[0];
        assert_eq!(item.file_name.clone().unwrap(), filename);
        assert_eq!(item.mime_type.clone().unwrap(), "image/jpeg");
    }

    #[test]
    fn test_deserialization_error() {
        let json = serde_json::json!({
            "existing_field": "value"
        });

        let result: Result<TestError, _> = parse_output(&json);
        assert!(result.is_err());

        if let Err(ExifParseError::Deserialization { path, source }) = result {
            assert_eq!(path, "non_existent_field");
            assert!(source.to_string().contains("missing field"));
        }
    }
}
