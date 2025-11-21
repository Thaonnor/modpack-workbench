mod database;
mod recipe_parser;
mod scanner;

use database::{Database, ExtractionResult, Recipe};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use std::fs::File;
use std::io::Read;
use std::sync::OnceLock;
use zip::ZipArchive;

#[derive(Clone, Serialize)]
struct ExtractionProgress {
    current: usize,
    total: usize,
    current_mod: String,
}

static DATABASE: OnceLock<Database> = OnceLock::new();

fn get_db() -> &'static Database {
    DATABASE.get().expect("Database not initialized")
}

#[tauri::command]
fn scan_folder(path: String) -> Result<Vec<scanner::FileInfo>, String> {
    scanner::scan_directory(&path)
}

#[tauri::command]
fn get_jar_contents(path: String) -> Result<Vec<scanner::JarEntry>, String> {
    scanner::read_jar_contents(&path)
}

#[tauri::command]
async fn extract_all_recipes(app: AppHandle, paths: Vec<String>) -> Result<ExtractionResult, String> {
    // Run extraction in a background thread using tauri's async runtime
    tauri::async_runtime::spawn_blocking(move || {
        let db = get_db();

        // Clear existing data for fresh extraction
        db.clear_all().map_err(|e| format!("Failed to clear database: {}", e))?;

        let mut mods_processed = 0;
        let mut recipes_extracted = 0;
        let mut errors = Vec::new();
        let total = paths.len();

        for (index, jar_path) in paths.iter().enumerate() {
            // Extract mod name from jar filename
            let mod_name = std::path::Path::new(jar_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| jar_path.clone());

            // Emit progress event
            let _ = app.emit("extraction-progress", ExtractionProgress {
                current: index + 1,
                total,
                current_mod: mod_name.clone(),
            });

            let file = match File::open(jar_path) {
                Ok(f) => f,
                Err(e) => {
                    errors.push(format!("{}: {}", jar_path, e));
                    continue;
                }
            };

            let mut archive = match ZipArchive::new(file) {
                Ok(a) => a,
                Err(e) => {
                    errors.push(format!("{}: {}", jar_path, e));
                    continue;
                }
            };

            let mod_id = match db.insert_mod(&mod_name, jar_path) {
                Ok(id) => id,
                Err(e) => {
                    errors.push(format!("{}: Failed to insert mod: {}", mod_name, e));
                    continue;
                }
            };

            mods_processed += 1;

            // Find and process recipe files
            let entry_names: Vec<String> = (0..archive.len())
                .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
                .collect();

            for entry_name in entry_names {
                // Check if it's a recipe JSON file
                let parts: Vec<&str> = entry_name.split('/').collect();
                if parts.len() < 4 || parts[0] != "data" {
                    continue;
                }
                if parts[2] != "recipe" && parts[2] != "recipes" {
                    continue;
                }
                if !entry_name.ends_with(".json") {
                    continue;
                }

                // Read the file contents
                let mut entry = match archive.by_name(&entry_name) {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let mut contents = String::new();
                if entry.read_to_string(&mut contents).is_err() {
                    continue;
                }

                // Parse the recipe
                let parsed = match recipe_parser::parse_recipe(&contents) {
                    Ok(p) => p,
                    Err(e) => {
                        errors.push(format!("{}:{}: {}", mod_name, entry_name, e));
                        continue;
                    }
                };

                // Insert into database
                match db.insert_recipe(
                    mod_id,
                    &entry_name,
                    &parsed.recipe_type,
                    parsed.result_item.as_deref(),
                    parsed.result_count,
                    &contents,
                    &parsed.ingredients,
                ) {
                    Ok(_) => recipes_extracted += 1,
                    Err(e) => {
                        errors.push(format!("{}:{}: {}", mod_name, entry_name, e));
                    }
                }
            }
        }

        Ok(ExtractionResult {
            mods_processed,
            recipes_extracted,
            errors,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
}

#[tauri::command]
fn search_recipes_by_output(item: String) -> Result<Vec<Recipe>, String> {
    get_db()
        .search_by_output(&item)
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
fn search_recipes_by_ingredient(item: String) -> Result<Vec<Recipe>, String> {
    get_db()
        .search_by_ingredient(&item)
        .map_err(|e| format!("Search failed: {}", e))
}

#[tauri::command]
fn list_recipes(offset: i64, limit: i64) -> Result<Vec<Recipe>, String> {
    get_db()
        .list_recipes(offset, limit)
        .map_err(|e| format!("List failed: {}", e))
}

#[tauri::command]
fn get_recipe_count() -> Result<i64, String> {
    get_db()
        .get_recipe_count()
        .map_err(|e| format!("Count failed: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize database in app data directory
            let app_data = app.path().app_data_dir().expect("Failed to get app data dir");
            let db_path = app_data.join("recipes.db");

            let db = Database::new(db_path).expect("Failed to initialize database");
            DATABASE.set(db).expect("Database already initialized");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            get_jar_contents,
            extract_all_recipes,
            search_recipes_by_output,
            search_recipes_by_ingredient,
            list_recipes,
            get_recipe_count
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
