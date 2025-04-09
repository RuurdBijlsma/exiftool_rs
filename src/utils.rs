#[cfg(test)]
pub(crate) mod test_helpers {
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    pub fn list_files_recursive(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| Ok(e.into_path()))
            .collect()
    }

    pub fn test_image_path() -> PathBuf {
        PathBuf::from("data/valid/IMG_20170801_162043.jpg")
    }
}
