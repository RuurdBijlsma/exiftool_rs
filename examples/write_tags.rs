use exiftool::ExifTool;
use std::path::PathBuf;
use std::{fs, path::Path};
use tempfile::Builder;

const SOURCE_IMAGE_PATH: &str = "data/image.jpg";

// Helper to create a temp copy (avoids modifying original data)
fn setup_temp_file(src: &Path) -> std::io::Result<PathBuf> {
    let (_, pb) = Builder::new().suffix(".jpg").tempfile_in("data")?.keep()?;
    fs::copy(src, &pb)?;
    Ok(pb)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut et = ExifTool::new()?;
    let source_path = Path::new(SOURCE_IMAGE_PATH);
    let temp_file = setup_temp_file(source_path)?;

    println!("Working with temporary file: {}", temp_file.display());

    // Use -overwrite_original to prevent backup file creation
    let args = ["-overwrite_original"];

    // 1. Write String tag
    et.write_tag(&temp_file, "UserComment", "Hello from exiftool-rs!", &args)?;
    let comment: String = et.read_tag(&temp_file, "UserComment")?;
    println!("Wrote and read back comment: '{}'", comment);
    assert_eq!(comment, "Hello from exiftool-rs!");

    // 2. Write Numeric tag
    et.write_tag(&temp_file, "Rating", 5, &args)?;
    let rating: u8 = et.read_tag(&temp_file, "Rating")?;
    println!("Wrote and read back rating: {}", rating);
    assert_eq!(rating, 5);

    // 3. Write Binary tag (tiny dummy JPEG)
    let dummy_thumb = b"\xFF\xD8\xFF\xD9";
    et.write_tag_binary(&temp_file, "ThumbnailImage", &dummy_thumb[..], &args)?;
    let thumb = et.read_tag_binary(&temp_file, "ThumbnailImage")?;
    println!("Wrote and read back thumbnail: {} bytes", thumb.len());
    assert_eq!(thumb, dummy_thumb);

    fs::remove_file(temp_file)?;

    Ok(())
}
