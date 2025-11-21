mod scanner;

#[tauri::command]
fn scan_folder(path: String) -> Result<Vec<scanner::FileInfo>, String> {
    scanner::scan_directory(&path)
}

#[tauri::command]
fn get_jar_contents(path: String) -> Result<Vec<scanner::JarEntry>, String> {
    scanner::read_jar_contents(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![scan_folder, get_jar_contents])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
