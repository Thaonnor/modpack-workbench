use std::fs::{self, File};
use std::path::Path;
use serde::Serialize;
use zip::ZipArchive;

#[derive(Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
}

pub fn scan_directory(path: &str) -> Result<Vec<FileInfo>, String> {
    let dir_path = Path::new(path);

    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }

    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut files = Vec::new();

    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let metadata = entry.metadata()
            .map_err(|e| format!("Failed to read metadata: {}", e))?;

        let name = entry.file_name().to_string_lossy().to_string();

        // Only include .jar files
        if metadata.is_file() && name.to_lowercase().ends_with(".jar") {
            files.push(FileInfo {
                name,
                path: entry.path().to_string_lossy().to_string(),
            });
        }
    }

    // Sort by name
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(files)
}

#[derive(Serialize)]
pub struct JarEntry {
    pub name: String,
    pub is_dir: bool,
}

pub fn read_jar_contents(path: &str) -> Result<Vec<JarEntry>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open jar: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read jar: {}", e))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| format!("Failed to read entry: {}", e))?;
        let name = entry.name().to_string();

        // Only include entries within data/*/recipe/ or data/*/recipes/
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() < 3 || parts[0] != "data" || (parts[2] != "recipe" && parts[2] != "recipes") {
            continue;
        }

        entries.push(JarEntry {
            name,
            is_dir: entry.is_dir(),
        });
    }

    // Sort entries alphabetically
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(entries)
}

