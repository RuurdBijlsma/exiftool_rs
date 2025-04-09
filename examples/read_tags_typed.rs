use exiftool::ExifTool;
use serde::Deserialize;
use std::path::Path;

const IMAGE_PATH: &str = "data/image.jpg";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")] // Match ExifTool JSON keys
struct CameraInfo {
    make: String,
    model: String,
    #[serde(rename = "ISO")]
    iso: Option<u32>,
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
    //     make: "LG Electronics",
    //     model: "LG-H815",
    //     iso: Some(400),
    // }

    Ok(())
}