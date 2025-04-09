//! Concise example: Write string/numeric and binary tags.
use exiftool::ExifTool;
use std::{fs, path::{Path, PathBuf}};

const SOURCE_IMAGE_PATH: &str = "data/IMG_20170801_162043.jpg";

// Helper to create a temp copy (avoids modifying original data)
fn setup_temp_copy(src: &Path) -> std::io::Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let filename = format!("exiftool_write_test_{}.jpg", rand::random::<u32>());
    let temp_path = temp_dir.join(filename);
    fs::copy(src, &temp_path)?;
    Ok(temp_path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut et = ExifTool::new()?;
    let source_path = Path::new(SOURCE_IMAGE_PATH);
    let temp_path = setup_temp_copy(source_path)?;

    println!("Working with temporary file: {}", temp_path.display());

    // Use -overwrite_original to prevent backup file creation
    let args = ["-overwrite_original"];

    // 1. Write String tag
    et.write_tag(&temp_path, "UserComment", "Hello from exiftool-rs!", &args)?;
    let comment: String = et.read_tag(&temp_path, "UserComment")?;
    println!("Wrote and read back comment: '{}'", comment);
    assert_eq!(comment, "Hello from exiftool-rs!");

    // 2. Write Numeric tag
    et.write_tag(&temp_path, "Rating", 5u8, &args)?;
    let rating: u8 = et.read_tag(&temp_path, "Rating")?;
    println!("Wrote and read back rating: {}", rating);
    assert_eq!(rating, 5);

    // 3. Write Binary tag (tiny dummy JPEG)
    let dummy_thumb = b"\xFF\xD8\xFF\xD9";
    et.write_tag_binary(&temp_path, "ThumbnailImage", &dummy_thumb[..], &args)?;
    let thumb = et.read_tag_binary(&temp_path, "ThumbnailImage")?;
    println!("Wrote and read back thumbnail: {} bytes", thumb.len());
    assert_eq!(thumb, dummy_thumb);

    // Clean up
    fs::remove_file(&temp_path)?;
    println!("Cleaned up temporary file.");

    Ok(())
}