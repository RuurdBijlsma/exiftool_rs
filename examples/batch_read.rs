//! Concise example: Read metadata from multiple files in batch.
use exiftool::{ExifTool, ExifToolError};
use std::path::{Path, PathBuf};

const IMAGE_DIR: &str = "data/valid"; // Adjust if needed

// Helper to find image files (basic)
fn find_images_in_dir(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| matches!(ext.to_str(), Some("jpg") | Some("jpeg") | Some("png") | Some("tif"))))
        .collect())
}

fn main() -> Result<(), ExifToolError> {
    let mut et = ExifTool::new()?;
    let img_dir_path = Path::new(IMAGE_DIR);

    let paths = find_images_in_dir(img_dir_path).unwrap_or_else(|e| {
        eprintln!("Failed to read image dir '{}': {}", IMAGE_DIR, e);
        Vec::new()
    });

    if paths.is_empty() {
        println!("No images found in '{}'. Exiting.", IMAGE_DIR);
        return Ok(());
    }

    println!("Found {} images in '{}'. Reading batch...", paths.len(), IMAGE_DIR);

    // Read FileName and ImageSize for all files in the list
    let results = et.json_batch(&paths, &["-FileName", "-ImageSize"])?;

    println!("\n--- Batch Results ---");
    for metadata in results {
        println!(
            "File: {:?}, Size: {:?}",
            metadata.get("FileName").and_then(|v| v.as_str()),
            metadata.get("ImageSize").and_then(|v| v.as_str()) // ImageSize is often string "WxH"
        );
    }

    Ok(())
}