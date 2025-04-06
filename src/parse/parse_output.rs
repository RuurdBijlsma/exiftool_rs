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
    use crate::executors::stay_open::ExifTool;
    use crate::structs::g2::ExifData;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct TestError {
        wrong_field: String,
    }

    #[test]
    fn test_successful_deserialization() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let filename = "IMG_20170801_162043.jpg";
        let file_path = format!("test_data/{}", filename);
        let json = exiftool.file_metadata(&file_path, &["-g2"])?;

        let result: ExifData = parse_output(&json)?;
        dbg!(&result);
        assert_eq!(
            result.other.clone().and_then(|o| o.file_name).unwrap(),
            filename
        );
        assert_eq!(
            result.other.clone().and_then(|o| o.mime_type).unwrap(),
            "image/jpeg"
        );

        assert_eq!(
            result
                .image
                .clone()
                .and_then(|i| i.blue_matrix_column)
                .unwrap(),
            vec![0.14307, 0.06061, 0.7141]
        );
        let bytes = exiftool.binary_field(&file_path, "BlueTRC")?;
        dbg!(bytes.len());

        Ok(())
    }

    #[test]
    fn test_deserialization_error() -> Result<(), ExifToolError> {
        let json = serde_json::json!({
            "existing_field": "value"
        });

        let result: Result<TestError, _> = parse_output(&json);
        assert!(result.is_err());

        if let Err(ExifToolError::Deserialization { path, source }) = result {
            assert_eq!(path, ".");
            assert!(source.to_string().contains("missing field"));
        }

        Ok(())
    }
}
