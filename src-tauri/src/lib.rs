// Tauri backend with modular structure
mod models;
mod config;
mod state;
mod utils;
mod commands;

use state::RunningProcesses;
use commands::*;

use std::sync::Mutex;
use std::collections::HashMap;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(RunningProcesses {
            processes: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            launch_exe,
            kill_game,
            pick_exe,
            pick_folder_and_scan,
            add_game,
            list_games,
            remove_game,
            search_bangumi,
            get_bangumi_subject,
            update_game_image,
            update_game_info,
            update_game_playtime,
            set_access_token,
            get_access_token,
            load_cache,
            save_cache,
            download_image,
            get_project_root,
            delete_cached_image,
            test_network_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
