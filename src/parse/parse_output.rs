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
    use crate::exiftool::ExifTool;
    use crate::structs::g2::ExifData;
    use crate::utils::test_helpers::list_files_recursive;
    use serde::Deserialize;
    use std::path::Path;

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct TestError {
        wrong_field: String,
    }

    #[test]
    fn test_successful_deserialization() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let filename = "IMG_20170801_162043.jpg";
        let file_path_str = format!("test_data/{}", filename);
        let file_path = Path::new(&file_path_str);
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
    #[test]
    fn test_deserialize_all() -> Result<(), ExifToolError> {
        let test_dir = "test_data";

        // Collect all files in directory (non-recursive)

        let binding = list_files_recursive(test_dir.as_ref())?;
        let mut args: Vec<&str> = binding.iter()
            .map(|p| p.to_str())
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect();
        assert!(!args.is_empty());
        args.insert(0, "-g2");

        let mut exiftool = ExifTool::new()?;
        let result = exiftool.execute_json(&args)?;
        let parsed = parse_output::<Vec<ExifData>>(&result)?;

        for item in parsed{
            println!("{:?}", item.source_file);

        }

        Ok(())
    }
}
