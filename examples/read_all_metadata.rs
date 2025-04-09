//! Concise example: Read all metadata (as JSON Value or into ExifData struct).
use exiftool::{ExifData, ExifTool, ExifToolError};
use std::path::Path;

const IMAGE_PATH: &str = "data/IMG_20170801_162043.jpg";

fn main() -> Result<(), ExifToolError> {
    let mut et = ExifTool::new()?;
    let path = Path::new(IMAGE_PATH);

    // 1. Read all metadata as raw JSON Value (grouped by -g1)
    println!("--- Reading all metadata as JSON (-g1) ---");
    let json_val = et.json(path, &["-g1"])?;
    println!(
        "Make from JSON: {:?}",
        json_val.get("EXIF").and_then(|g| g.get("Make")) // Access nested value
    );
    // Optionally print the whole JSON (can be large)
    // println!("{}", serde_json::to_string_pretty(&json_val).unwrap());

    // 2. Read all metadata and deserialize into ExifData struct (requires -g2)
    println!("\n--- Reading all metadata into ExifData (-g2) ---");
    let exif_data: ExifData = et.read_metadata(path, &["-g2"])?;
    println!(
        "Make from ExifData: {:?}",
        exif_data.camera.as_ref().and_then(|c| c.make.as_ref())
    );
    println!(
        "FileName from ExifData: {:?}",
        exif_data.other.as_ref().and_then(|o| o.file_name.as_ref())
    );
    // Optionally print the whole struct (can be large)
    // println!("{:#?}", exif_data);

    Ok(())
}