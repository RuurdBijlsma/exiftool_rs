use exiftool_wrapper::executors::stay_open::ExifTool;
use exiftool_wrapper::parse::parse_output::parse_output;
use exiftool_wrapper::structs::poging_drie::ExifOutput;
use rand::seq::SliceRandom;
use std::fs::{self};
use std::path::PathBuf;

// Using Result<(), Box<dyn std::error::Error>> for main to easily handle errors
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Hardcoded directory path
    let dir_path = PathBuf::from("E:/Backup/Photos/photos/photos");

    // Number of random files to sample
    let sample_size = 5000;

    // Read directory and collect all regular files
    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();

    if files.is_empty() {
        println!("No files found in the directory: {}", dir_path.display());
        return Ok(()); // Exit cleanly
    }

    if files.len() < sample_size {
        println!(
            "Warning: Found only {} files, sampling all of them.",
            files.len()
        );
    } else {
        // Shuffle the files
        // Use thread_rng() for simplicity unless specific seeding is needed
        let mut rng = rand::thread_rng();
        files.shuffle(&mut rng);
    }

    // Take the first N items (or all if fewer than N exist)
    let sampled_files: Vec<PathBuf> = files.into_iter().take(sample_size).collect();

    if sampled_files.is_empty() {
        println!("No files were sampled.");
        return Ok(());
    }

    println!("Sampling {} files:", sampled_files.len());
    for file in &sampled_files {
        println!("  - {}", file.display());
    }

    // Convert sampled_files to a Vec<String> for owned paths
    let file_paths: Vec<String> = sampled_files
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    // Start with the arguments for exiftool
    // -g2: Group tags by family 2 (more specific groups like Camera, Image, Location)
    let mut args: Vec<&str> = vec!["-g2"];

    // Add file paths
    args.extend(file_paths.iter().map(|s| s.as_str()));

    // Execute exiftool on the sampled files
    println!("Running exiftool...");
    let mut tool = ExifTool::new()?;
    let exif_json = tool.execute_json(&args)?;
    let parsed = parse_output::<ExifOutput>(&exif_json)?;
    dbg!(&parsed.len());

    Ok(())
}
