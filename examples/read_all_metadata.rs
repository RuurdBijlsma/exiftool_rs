use exiftool::{ExifData, ExifTool, ExifToolError};
use std::path::Path;

const IMAGE_PATH: &str = "data/image.jpg";

fn main() -> Result<(), ExifToolError> {
    let mut et = ExifTool::new()?;
    let path = Path::new(IMAGE_PATH);

    // Option 1. Read all metadata as raw JSON Value
    let json_val = et.json(path, &[])?;
    println!("{}", serde_json::to_string_pretty(&json_val)?);

    // Option 2. Read all metadata and deserialize into ExifData struct
    // The provided ExifData struct requires `-g2`
    println!("\n--- Reading all metadata into ExifData (-g2) ---");
    let exif_data: ExifData = et.read_metadata(path, &["-g2"])?;
    println!("{:#?}", exif_data);

    Ok(())
}