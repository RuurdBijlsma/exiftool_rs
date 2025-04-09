//! Concise example: Read multiple specific tags into a custom struct.
use exiftool::ExifTool;
use serde::Deserialize;
use std::path::Path;

const IMAGE_PATH: &str = "data/IMG_20170801_162043.jpg";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")] // Match ExifTool JSON keys
struct CameraInfo {
    make: String,
    model: String,
    #[serde(rename = "ISO")] // Example of rename
    iso: Option<u32>, // Optional tag
}

fn main() -> Result<(), exiftool::ExifToolError> {
    let mut et = ExifTool::new()?;
    let path = Path::new(IMAGE_PATH);

    // Tags to extract (without leading '-')
    let tags = ["Make", "Model", "ISO"];

    // Read only these tags and deserialize
    let info: CameraInfo = et.read_tags(path, &tags)?;

    println!("Read specific tags into struct:\n{:#?}", info);
    // Output:
    // Read specific tags into struct:
    // CameraInfo {
    //     make: "Huawei",
    //     model: "Nexus 6P",
    //     iso: Some(121),
    // }

    Ok(())
}