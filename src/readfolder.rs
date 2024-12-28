use std::{
    fs, path::PathBuf
};

pub fn read_folder(path: &PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                files.extend(read_folder(&entry_path));
            } else if let Some(ext) = entry_path.extension() {
                if ext == "html" || ext == "js" || ext == "css" {
                    files.push(entry_path);
                }
            }
        }
    }

    files
}


pub fn get_files(path: &PathBuf) -> Vec<PathBuf> {
    read_folder(path)
}