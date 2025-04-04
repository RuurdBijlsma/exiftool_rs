use serde_json::Value;
use std::error::Error;
use tokio::process::Command;

/// Executes ExifTool with the given arguments, using JSON output mode,
/// and returns the parsed Exif metadata as `ExifOutput`.
pub async fn execute(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    // Build the argument list: first add the JSON flag, then the rest.
    let mut cmd_args = vec!["-json"];
    cmd_args.extend_from_slice(args);

    let output = Command::new("exiftool").args(&cmd_args).output().await?;

    if !output.status.success() {
        return Err(format!("ExifTool exited with non-zero status: {}", output.status).into());
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Run the async test with Tokio.
    #[tokio::test]
    async fn test_execute() {
        let result = execute(&["test_data/IMG_20170801_162043.jpg"]).await;
        assert!(result.is_ok());
        if let Ok(exif_str) = result {
            println!("Json Exif Data: {:#?}", exif_str);
            // You could add further assertions here based on expected values.
        }
    }
}
