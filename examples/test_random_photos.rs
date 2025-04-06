use exiftool_wrapper::exiftool::ExifTool;
use exiftool_wrapper::parse::parse_output::parse_output;
use exiftool_wrapper::structs::g2::ExifData;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn list_files_recursive(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok()) // Ignore errors during traversal
        .filter(|e| e.file_type().is_file())
    // Only include files
    {
        files.push(entry.into_path());
    }

    Ok(files)
}

fn subtract_vecs(vec1: Vec<PathBuf>, vec2: Vec<PathBuf>) -> Vec<PathBuf> {
    let set2: HashSet<_> = vec2.into_iter().collect();
    vec1.into_iter().filter(|p| !set2.contains(p)).collect()
}

// Using Result<(), Box<dyn std::error::Error>> for main to easily handle errors
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let success_files_path = Path::new("examples/example_output/successes.txt");
    let success_files: Vec<PathBuf> = if success_files_path.exists() {
        let file = File::open(success_files_path)?;
        let reader = io::BufReader::new(file);
        reader
            .lines()
            .map(|line| Path::new(&line.unwrap()).to_path_buf())
            .collect()
    } else {
        Vec::<PathBuf>::new()
    };

    // Hardcoded directory path
    let dir_path = PathBuf::from("test_data/other_images");
    let files = list_files_recursive(&dir_path)?;
    let todo_files = subtract_vecs(files, success_files);

    let mut file_handle = OpenOptions::new()
        .append(true)
        .create(true)
        .open(success_files_path)?;

    let mut tool = ExifTool::new()?;
    for file in todo_files {
        // Start with the arguments for exiftool
        // -g2: Group tags by family 2 (more specific groups like Camera, Image, Location)
        println!("Running exiftool... {}", &file.display());
        let exif_json = tool.file_metadata(&file, &["-g2"])?;
        parse_output::<ExifData>(&exif_json)?;
        writeln!(file_handle, "{}", &file.display())?;
    }

    Ok(())
}
