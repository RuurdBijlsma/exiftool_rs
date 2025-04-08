use crate::core::exiftool::ExifTool;
use crate::error::ExifToolError;
use serde_json::Value;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

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
                file: path_str.to_string(),
                args: args.join(" "),
            })
        }
    }
    /// Get JSON metadata for a multiple files. This will return a single json object.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {...extra_args} {...file_paths} `
    ///
    /// You can tell exiftool to structure the output by grouping into categories with `-g1` or `-g2`.
    pub fn batch_file_metadata<I, P>(
        &mut self,
        file_paths: I,
        extra_args: &[&str],
    ) -> Result<Vec<Value>, ExifToolError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let file_paths: Vec<String> = file_paths
            .into_iter()
            .map(|p| p.as_ref().to_string_lossy().into_owned())
            .collect();
        let mut args = extra_args.to_vec();
        args.extend(file_paths.iter().map(String::as_str));
        let result = self.execute_json(&args)?;
        if let Value::Array(array) = result {
            Ok(array)
        } else {
            Err(ExifToolError::UnexpectedFormat {
                file: file_paths.join(", "),
                args: args.join(" "),
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

    /// Write binary data to a metadata field.
    /// This helper writes binary data into a field by first writing the data to a
    /// temporary file and then instructing ExifTool to read the value from that file.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -{field}<=temp_file {...extra_args} {file_path}`
    pub fn write_tag_binary(
        &mut self,
        file_path: &Path,
        field: &str,
        data: impl AsRef<[u8]>,
        extra_args: &[&str],
    ) -> Result<(), ExifToolError> {
        // Create a temporary file to hold the binary data
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(data.as_ref())?;

        // Get the temporary file's path as a string.
        let temp_path = temp_file.path().to_string_lossy();

        // Construct the field argument with the '<=' operator.
        let field_arg = format!("-{}<={}", field, temp_path);

        let file_path_str = file_path.to_string_lossy();
        let mut args = vec![field_arg.as_str()];
        args.extend_from_slice(extra_args);
        args.push(file_path_str.as_ref());

        let result = self.execute_str(&args)?;
        dbg!(&result);

        // The temporary file is automatically removed when temp_file goes out of scope.
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
    fn test_write_binary() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let temp_path = Path::new("test_data/temp_img2.jpg");

        // Copy test file to temporary location
        fs::copy(test_image(), temp_path)?;

        // Get thumbnail bytes to embed
        let thumb_image_to_embed = Path::new("test_data/other_images/jpg/gps/DSCN0010.jpg");
        let thumb_bytes = fs::read(thumb_image_to_embed)?;

        // Write binary data to thumbnail tag
        exiftool.write_tag_binary(temp_path, "ThumbnailImage", &thumb_bytes, &[])?;

        // Read back the binary tag to verify
        let read_bytes = exiftool.read_tag_binary(temp_path, "ThumbnailImage")?;
        assert_eq!(read_bytes, thumb_bytes);

        // Clean up temporary files
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
        let array = exiftool.batch_file_metadata(&files, &[])?;
        for file in array {
            if let Some(source) = file.get("SourceFile").map(|t| t.as_str()).flatten() {
                assert!(source.len() > 0);
            }
        }

        Ok(())
    }
}
