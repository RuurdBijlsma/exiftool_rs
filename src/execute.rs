use crate::output_type::ExifOutput;
use std::error::Error;
use tokio::process::Command;

/// Executes ExifTool with the given arguments, using JSON output mode,
/// and returns the parsed Exif metadata as `ExifOutput`.
pub async fn execute(args: &[&str]) -> Result<ExifOutput, Box<dyn Error>> {
    // Build the argument list: first add the JSON flag, then the rest.
    let mut cmd_args = vec!["-j"];
    cmd_args.extend_from_slice(args);

    // Execute ExifTool asynchronously.
    let output = Command::new("exiftool").args(&cmd_args).output().await?;

    if !output.status.success() {
        return Err(format!("ExifTool exited with non-zero status: {}", output.status).into());
    }

    // Parse the stdout into our ExifOutput type.
    let exif: ExifOutput = serde_json::from_slice(&output.stdout)?;
    Ok(exif)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Run the async test with Tokio.
    #[tokio::test]
    async fn test_execute() {
        let result = execute(&["test_data/IMG_20170801_162043.jpg"]).await;
        assert!(result.is_ok());
        if let Ok(exif) = result {
            println!("Parsed Exif Data: {:#?}", exif);
            // You could add further assertions here based on expected values.
        }
    }
}
