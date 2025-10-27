use crate::config::{games_db_path, config_path, cache_path, images_dir_path};

#[tauri::command]
pub fn get_app_data_paths() -> serde_json::Value {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_string_lossy().to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    
    serde_json::json!({
        "exe_path": exe_path,
        "exe_dir": exe_dir,
        "games_db_path": games_db_path().to_string_lossy().to_string(),
        "config_path": config_path().to_string_lossy().to_string(),
        "cache_path": cache_path().to_string_lossy().to_string(),
        "images_dir": images_dir_path().to_string_lossy().to_string(),
    })
}
