mod commands;
mod delta;
mod diff;
#[cfg(test)]
mod test_diff;

use commands::{check_delta, get_app_args, get_diff, get_file_tree, read_file_content};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_file_tree,
            get_diff,
            read_file_content,
            check_delta,
            get_app_args,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
