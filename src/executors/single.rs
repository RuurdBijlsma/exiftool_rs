use crate::error::ExifToolError;
use serde_json::Value;
use std::process::Command;

pub fn execute_bytes(args: &[&str]) -> Result<Vec<u8>, ExifToolError> {
    let output = Command::new("exiftool").args(args).output()?;

    let stdout = output.stdout;
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut errors = Vec::new();
    for line in stderr.lines() {
        if line.starts_with("Error: ") {
            if let Some(filename) = line.strip_prefix("Error: File not found - ") {
                return Err(ExifToolError::FileNotFound(filename.trim().to_string()));
            }
            errors.push(line.to_string());
        }
    }

    if !errors.is_empty() {
        return Err(ExifToolError::ExifToolError(errors.join("\n")));
    }

    // Check exit status
    if !output.status.success() && stdout.is_empty() {
        return Err(ExifToolError::ExifToolError(format!(
            "ExifTool exited with status: {}",
            output.status
        )));
    }

    // Empty output check
    if stdout.is_empty() {
        return Err(ExifToolError::EmptyResponse);
    }

    Ok(stdout)
}

pub fn execute_json(args: &[&str]) -> Result<Value, ExifToolError> {
    let mut cmd_args = vec!["-json"];
    cmd_args.extend_from_slice(args);

    let output_bytes = execute_bytes(&cmd_args)?;
    let json_str = String::from_utf8(output_bytes)?;
    let value: Value = serde_json::from_str(&json_str)?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::get_files_in_dir;
    use image::ImageReader;
    use rayon::prelude::*;
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn test_basic_functionality() -> Result<(), ExifToolError> {
        let file = "test_data/IMG_20170801_162043.jpg";

        assert!(Path::new(file).exists(), "Test file doesn't exist");

        // First query
        let result = execute_json(&["-g2", file])?;
        assert!(result.is_array());
        println!("First result: {:#?}", result);

        // Second query with same process
        let result2 = execute_json(&["-createdate", file])?;
        assert!(result2.is_array());
        println!("Second result: {:#?}", result2);
        Ok(())
    }

    #[test]
    fn test_file_not_found() -> Result<(), ExifToolError> {
        let filename = "nonexistent.jpg";
        let result = execute_bytes(&[filename]);
        assert!(!result.is_ok());

        match result {
            Err(ExifToolError::FileNotFound(f)) => {
                assert_eq!(f, filename);
                Ok(())
            }
            other => panic!("Expected FileNotFound error, got {:?}", other),
        }
    }

    #[test]
    fn test_binary_response() -> Result<(), ExifToolError> {
        let file = "test_data/IMG_20170801_162043.jpg";
        let result = execute_bytes(&["-b", "-ThumbnailImage", file]);

        match result {
            Ok(data) => {
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
    fn test_all_exif_files() -> Result<(), ExifToolError> {
        let test_dir = "test_data/exiftool_images";

        // Collect all files in directory (non-recursive)
        let files = get_files_in_dir(test_dir)?;
        assert!(!files.is_empty());

        let results: Vec<Result<(), String>> = files
            .par_iter()
            .map(|file| {
                let file_path = file.to_string_lossy();
                println!("\nTesting: {}", file_path);

                match execute_json(&[file_path.as_ref()]) {
                    Ok(result) => {
                        if !result.is_array() {
                            return Err(format!("Expected JSON array for file {}", file_path));
                        }

                        if result.as_array().unwrap().is_empty() {
                            return Err(format!("Empty result for file {}", file_path));
                        }

                        println!("Metadata for {}: {:#?}", file_path, result);
                        Ok(())
                    }
                    Err(e) => Err(format!("Error processing {}: {:?}", file_path, e)),
                }
            })
            .collect();

        // Aggregate errors
        let errors: Vec<_> = results.into_iter().filter_map(Result::err).collect();
        if !errors.is_empty() {
            panic!("Errors occurred:\n{}", errors.join("\n"));
        }

        Ok(())
    }
}
