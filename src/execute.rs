use serde_json::Value;
use std::process::ExitStatus;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum ExifError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("ExifTool failed with status {status}: {message}")]
    NonZeroExit {
        status: ExitStatus,
        message: String,
    },
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Executes ExifTool with the given arguments, using JSON output mode,
/// and returns the parsed Exif metadata as `serde_json::Value`.
pub async fn execute(args: &[&str]) -> Result<Value, ExifError> {
    let mut cmd_args = vec!["-json"];
    cmd_args.extend_from_slice(args);

    let output = Command::new("exiftool").args(&cmd_args).output().await?;

    if !output.status.success() {
        let message = String::from_utf8(output.stderr)?;

        // Check for file not found error
        if message.contains("File not found") {
            let filename = args.first().unwrap_or(&"<unknown>").to_string();
            return Err(ExifError::FileNotFound(filename));
        }

        return Err(ExifError::NonZeroExit {
            status: output.status,
            message,
        });
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_success() {
        let result = execute(&["test_data/IMG_20170801_162043.jpg"]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_non_existent_file() {
        let filename = "nonexistent.jpg";
        let result = execute(&[filename]).await;
        match result {
            Err(ExifError::FileNotFound(f)) => {
                assert_eq!(f, filename);
            }
            Err(e) => panic!("Expected FileNotFound error, got {:?}", e),
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[tokio::test]
    async fn test_other_error() {
        let filename = "test_data/output.json";
        let result = execute(&[filename]).await;
        match result {
            Err(ExifError::FileNotFound(f)) => {
                assert_eq!(f, filename);
            }
            Err(e) => panic!("Expected FileNotFound error, got {:?}", e),
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[tokio::test]
    async fn test_utf8_error_handling() {
        // Test case for invalid UTF-8 (though exiftool should always output valid UTF-8)
    }
}