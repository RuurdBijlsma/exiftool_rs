use serde_json::Value;

pub fn value_to_clean_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        _ => val.to_string(),
    }
}
#[cfg(test)]
pub mod test_helpers {
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    pub fn list_files_recursive(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
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
}
