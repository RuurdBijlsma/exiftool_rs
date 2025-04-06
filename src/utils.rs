use std::{fs, io};
use std::path::PathBuf;
use serde_json::Value;

/// Get all files in a directory (non-recursive)
pub fn get_files_in_dir(dir: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            files.push(path);
        }
    }

    // Sort for consistent test order
    files.sort();

    Ok(files)
}

pub fn value_to_clean_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        _ => val.to_string(),
    }
}