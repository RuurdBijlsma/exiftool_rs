use crate::core::exiftool::ExifTool;
use crate::error::ExifToolError;
use serde_json::Value;
use std::path::Path;

impl ExifTool {
    /// Get JSON metadata for a single file. This will return a single json object.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {file_path} {...extra_args}`
    ///
    /// You can tell exiftool to structure the output by grouping into categories with `-g1` or `-g2`.
    pub fn file_metadata(
        &mut self,
        file_path: &Path,
        extra_args: &[&str],
    ) -> Result<Value, ExifToolError> {
        let path_str = file_path.to_string_lossy();
        let mut args = vec![path_str.as_ref()];
        args.extend_from_slice(extra_args);

        let result = self.execute_json(&args)?;
        if let Some(single) = result.as_array().and_then(|a| a.first()) {
            Ok(single.clone())
        } else {
            Err(ExifToolError::UnexpectedFormat {
                command: args.join(" "),
            })
        }
    }

    /// Extract bytes from a binary field.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool {file_path} -b -{field_name}`
    pub fn read_tag_binary(
        &mut self,
        file_path: &Path,
        field_name: &str,
    ) -> Result<Vec<u8>, ExifToolError> {
        let path_str = file_path.to_string_lossy();
        self.execute_raw(&[path_str.as_ref(), "-b", &format!("-{}", field_name)])
    }

    /// Read contents of a tag as a json value.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {file_path} -{field}`
    pub fn read_tag_json(&mut self, file_path: &Path, field: &str) -> Result<Value, ExifToolError> {
        let field_str = format!("-{}", field);
        let value = self.file_metadata(file_path, &[&field_str])?;
        if let Some(field_value) = value.get(field) {
            return Ok(field_value.to_owned());
        }
        Err(ExifToolError::FieldDoesNotExist {
            file: file_path.to_string_lossy().to_string(),
            field: field.to_string(),
        })
    }

    /// Write a metadata field to a file.
    /// Keep in mind that ExifTool will never overwrite a file, so a copy of the file will
    /// be made, called {filename}_original
    ///
    /// The command executed by this function is:
    /// `exiftool -{field}={value} {...extra_args} {file_path}`
    pub fn write_tag<T: ToString>(
        &mut self,
        file_path: &Path,
        field: &str,
        value: T,
        extra_args: &[&str],
    ) -> Result<(), ExifToolError> {
        let value_str = value.to_string();
        let field_arg = format!("-{}={}", field, value_str);
        let path_str = file_path.to_string_lossy();

        let mut args = vec![field_arg.as_str()];
        args.extend_from_slice(extra_args);
        args.push(path_str.as_ref());

        let result = self.execute_str(&args)?;
        dbg!(&result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_helpers::{list_files_recursive, test_image};
    use image::ImageReader;
    use std::fs;
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn test_file_metadata() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let binding = test_image();
        let file = Path::new(&binding);

        // First query
        let result = exiftool.file_metadata(file, &[])?;
        assert!(result.is_object());

        // Second query with same process
        let result2 = exiftool.file_metadata(file, &["-createdate"])?;
        assert!(result2.is_object());
        Ok(())
    }

    #[test]
    fn test_read_binary_tag() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let binding = test_image();
        let file = Path::new(&binding);
        let result = exiftool.read_tag_binary(file, "ThumbnailImage");

        match result {
            Ok(data) => {
                dbg!(data.len());
                // Verify it's a valid JPEG
                let cursor = Cursor::new(&data);
                let format = ImageReader::new(cursor)
                    .with_guessed_format()
                    .expect("Cursor never fails")
                    .format();

                assert_eq!(format, Some(image::ImageFormat::Jpeg));

                // decode to check that it's readable
                let img = image::load_from_memory(&data).unwrap();
                println!("Thumbnail dimensions: {}x{}", img.width(), img.height());

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_write_tag() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let temp_path = Path::new("test_data/temp_img.jpg");

        // Copy test file
        fs::copy(test_image(), temp_path)?;

        // Write new value with overwrite
        exiftool.write_tag(temp_path, "Author", "TestValue", &[])?;

        // Read back verification
        let value: String = exiftool.read_tag(temp_path, "Author")?;
        assert_eq!(value, "TestValue");

        fs::remove_file(temp_path)?;
        fs::remove_file(format!("{}_original", temp_path.display()))?;
        Ok(())
    }

    #[test]
    fn test_all_exif_files() -> Result<(), ExifToolError> {
        let test_dir = "test_data";

        // Collect all files in directory (non-recursive)
        let files = list_files_recursive(test_dir.as_ref())?;
        assert!(!files.is_empty());

        let mut exiftool = ExifTool::new()?;

        for file in files {
            // Single full metadata extraction per file
            let result = exiftool.file_metadata(&file, &[])?;

            // Basic validation
            assert!(
                result.is_object(),
                "Expected JSON array for file {}",
                &file.display()
            );
            assert!(
                !result.as_object().unwrap().is_empty(),
                "Empty result for file {}",
                &file.display()
            );
        }

        Ok(())
    }
}
