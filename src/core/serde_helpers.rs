use crate::core::exiftool::ExifTool;
use crate::error::ExifToolError;
use serde::de::DeserializeOwned;
use std::path::Path;

impl ExifTool {
    /// Read contents of a tag and deserialize to a generic type.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {file_path} -{field}`
    pub fn read_tag<T: DeserializeOwned>(
        &mut self,
        file_path: &Path,
        field: &str,
    ) -> Result<T, ExifToolError> {
        let value = self.read_tag_json(file_path, field)?;
        serde_json::from_value(value).map_err(|e| ExifToolError::FieldDeserializationError {
            field: field.to_string(),
            file: file_path.to_string_lossy().to_string(),
            error: e.to_string(),
        })
    }

    /// Get metadata from a single file, deserialized to the specified type.
    /// Note that this adds te -g2 tag to structure the exiftool output in groups.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {file_path} -g2 {...extra_args}`
    pub fn file_metadata_parsed<T>(
        &mut self,
        file_path: &Path,
        extra_args: &[&str],
    ) -> Result<T, ExifToolError>
    where
        T: DeserializeOwned,
    {
        let mut args = vec!["-g2"];
        args.extend_from_slice(extra_args);
        let output = self.file_metadata(file_path, &args)?;
        Ok(serde_path_to_error::deserialize(output)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_helpers::test_image;
    use crate::{ExifData, ExifTool};
    use serde::Deserialize;
    use std::path::Path;

    #[test]
    fn test_read_tag() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let binding = test_image();
        let file = Path::new(&binding);

        let result: String = exiftool.read_tag(file, "Author")?;
        assert_eq!(result, "Ruurd");
        Ok(())
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct TestError {
        wrong_field: String,
    }

    #[test]
    fn test_successful_deserialization() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let binding = test_image();
        let file_path = Path::new(&binding);
        let filename = file_path.file_name().map(|f| f.to_string_lossy()).unwrap();

        let result: ExifData = exiftool.file_metadata_parsed(file_path, &[])?;
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
        let bytes = exiftool.read_tag_binary(file_path, "BlueTRC")?;
        dbg!(bytes.len());

        Ok(())
    }
}
