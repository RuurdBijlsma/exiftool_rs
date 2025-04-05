use exiftool_wrapper::executors::single::execute_json;
use rand::seq::SliceRandom;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Hardcoded directory path
    let dir_path = PathBuf::from("E:/Backup/Photos/photos/photos");

    // Number of random files to sample
    let sample_size = 2;

    // Read directory and collect all regular files
    let mut files: Vec<PathBuf> = fs::read_dir(&dir_path)
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();

    if files.is_empty() {
        println!("No files found in the directory.");
        return;
    }

    // Shuffle the files and take the first N items
    let mut rng = rand::rng();
    files.shuffle(&mut rng);
    let sampled_files: Vec<PathBuf> = files.into_iter().take(sample_size).collect();
    dbg!(&sampled_files);

    // Convert sampled_files to a Vec<String> for owned paths
    let file_paths: Vec<String> = sampled_files
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    // Start with the argument
    let mut args: Vec<&str> = vec!["-g2"];

    // Add file paths
    args.extend(file_paths.iter().map(|s| s.as_str()));

    // Execute exiftool on the sampled files, now as &[&str]
    let result = execute_json(&args).unwrap();
    dbg!(result);
}
