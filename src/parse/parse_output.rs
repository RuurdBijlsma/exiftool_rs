use crate::parse::output_type::ExifOutput;
use serde_path_to_error as spte;
use std::error::Error;
use serde_json::Value;

/// Executes ExifTool with the given arguments, using JSON output mode,
/// and returns the parsed Exif metadata as `ExifOutput`.
pub fn parse_output(output: &Value) -> Result<ExifOutput, Box<dyn std::error::Error>> {
    let exif = spte::deserialize(output).map_err(|e| {
        let path = e.path().to_string();
        println!("Failed at field: {}", path);
        println!("Problematic value: {:#?}", e.inner());
        e
    })?;

    Ok(exif)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execute::execute;

    #[tokio::test]
    async fn test_parse_output() {
        let out_result = execute(&["test_data/IMG_20170801_162043.jpg"]).await;
        assert!(out_result.is_ok());
        let parsed_result = parse_output(&out_result.unwrap());
        if let Err(e) = &parsed_result {
            println!("PARSE ERROR: {:#?}", e);
        }
        assert!(parsed_result.is_ok());
        if let Ok(exif) = parsed_result {
            println!("Parsed Exif Data: {:#?}", exif);
        }
    }
}
