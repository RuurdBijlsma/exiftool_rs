use exiftool_wrapper::executors::stay_open::ExifTool;
use exiftool_wrapper::parse::parse_output::parse_output;
use exiftool_wrapper::structs::g2::ExifOutput;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

fn subtract_vecs(vec1: Vec<PathBuf>, vec2: Vec<PathBuf>) -> Vec<PathBuf> {
    let set2: HashSet<_> = vec2.into_iter().collect();
    vec1.into_iter().filter(|p| !set2.contains(p)).collect()
}

// Using Result<(), Box<dyn std::error::Error>> for main to easily handle errors
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let success_files_path = Path::new("examples/successes.txt");
    let success_files: Vec<PathBuf> = if success_files_path.exists() {
        let file = File::open(success_files_path)?;
        let reader = io::BufReader::new(file);
        reader.lines().map(|line| Path::new(&line.unwrap()).to_path_buf()).collect()
    } else {
        Vec::<PathBuf>::new()
    };

    // Hardcoded directory path
    let dir_path = PathBuf::from("C:/Users/Ruurd/Pictures/photos");

    // Number of random files to sample
    let sample_size = 500;

    // Read directory and collect all regular files
    let all_files: Vec<PathBuf> = fs::read_dir(&dir_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();

    dbg!(&all_files.len());
    dbg!(&success_files.len());

    let mut files = subtract_vecs(all_files, success_files);

    dbg!(&files.len());

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
        let mut rng = rand::rng();
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

    let mut file_handle = OpenOptions::new()
        .append(true)
        .create(true)
        .open(success_files_path)?;

    let mut tool = ExifTool::new()?;
    for file in file_paths {
        // Start with the arguments for exiftool
        // -g2: Group tags by family 2 (more specific groups like Camera, Image, Location)
        let mut args: Vec<&str> = vec!["-g2"];

        // Add file paths
        args.push(&file);

        // Execute exiftool on the sampled files
        println!("Running exiftool... {}", file);
        let exif_json = tool.execute_json(&args)?;
        parse_output::<ExifOutput>(&exif_json)?;
        writeln!(file_handle, "{}", &file)?;
    }

    Ok(())
}
