use crate::error::ExifToolError;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_path_to_error;

/// Parses JSON output from ExifTool into the specified type
pub fn parse_output<T>(output: &Value) -> Result<T, ExifToolError>
where
    T: DeserializeOwned,
{
    let exif = serde_path_to_error::deserialize(output)?;
    Ok(exif)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execute::ExifTool;
    use crate::structs::structs::ExifOutput;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct TestError {
        wrong_field: String,
    }

    #[tokio::test]
    async fn test_successful_deserialization() {
        let mut exiftool = ExifTool::new().unwrap();
        let filename = "IMG_20170801_162043.jpg";
        let json = exiftool
            .execute_json(&[&format!("test_data/{}", filename)])
            .unwrap();

        let result: Result<ExifOutput, _> = parse_output(&json);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);
        let item = &parsed[0];
        assert_eq!(item.file_name.clone().unwrap(), filename);
        assert_eq!(item.mime_type.clone().unwrap(), "image/jpeg");
        assert_eq!(
            item.blue_matrix_column.clone().unwrap(),
            vec![0.14307, 0.06061, 0.7141]
        );

        println!("{:#?}", item);
        let result = item.blue_trc.clone().unwrap().extract("BlueTRC").unwrap();
        println!("{:#?}", result);
    }

    #[test]
    fn test_deserialization_error() {
        let json = serde_json::json!({
            "existing_field": "value"
        });

        let result: Result<TestError, _> = parse_output(&json);
        assert!(result.is_err());

        if let Err(ExifToolError::Deserialization { path, source }) = result {
            assert_eq!(path, ".");
            assert!(source.to_string().contains("missing field"));
        }
    }
}
