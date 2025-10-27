use std::fs;
use std::path::PathBuf;
use tauri::{Emitter, Manager, State};
use crate::models::GameEntry;
use crate::config::{load_games_db, save_games_db};
use crate::state::RunningProcesses;
use crate::utils::find_exe_with_folder_path;

// A simple greeting kept for compatibility
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Launch an executable by path and monitor the process
#[tauri::command]
pub async fn launch_exe(path: String, app: tauri::AppHandle, state: State<'_, RunningProcesses>) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() || !p.is_file() {
        return Err("executable not found".into());
    }
    
    // Spawn the process
    let mut child = std::process::Command::new(&p)
        .spawn()
        .map_err(|e| e.to_string())?;
    
    // Get process ID
    let pid = child.id();
    
    // Store process ID
    {
        let mut processes = state.processes.lock().unwrap();
        processes.insert(path.clone(), pid);
    }
    
    // Clone app handle for the async task
    let app_clone = app.clone();
    let path_clone = path.clone();
    
    // Monitor the process in a separate thread
    tauri::async_runtime::spawn(async move {
        // Wait for the process to exit
        match child.wait() {
            Ok(_) => {
                // Remove from tracking
                {
                    let state = app_clone.state::<RunningProcesses>();
                    let mut processes = state.processes.lock().unwrap();
                    processes.remove(&path_clone);
                }
                // Process has exited, emit event to frontend
                let _ = app_clone.emit("game-exited", path_clone);
            }
            Err(e) => {
                eprintln!("Error waiting for process: {}", e);
            }
        }
    });
    
    Ok(())
}

// Kill a running game process
#[tauri::command]
pub fn kill_game(path: String, state: State<'_, RunningProcesses>) -> Result<(), String> {
    let processes = state.processes.lock().unwrap();
    
    if let Some(&pid) = processes.get(&path) {
        #[cfg(target_os = "windows")]
        {
            // Use taskkill on Windows
            std::process::Command::new("taskkill")
                .args(&["/F", "/PID", &pid.to_string()])
                .output()
                .map_err(|e| e.to_string())?;
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Use kill on Unix-like systems
            use std::process::Command;
            Command::new("kill")
                .args(&["-9", &pid.to_string()])
                .output()
                .map_err(|e| e.to_string())?;
        }
        
        Ok(())
    } else {
        Err("Process not found".into())
    }
}

#[tauri::command]
pub fn add_game(path: &str, name: Option<&str>, folder_path: Option<Vec<String>>) -> Result<GameEntry, String> {
    let p = PathBuf::from(path);
    if !p.exists() || !p.is_file() {
        return Err("path not found or not a file".into());
    }
    let nm = name.map(|s| s.to_string()).unwrap_or_else(|| {
        p.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".into())
    });
    let entry = GameEntry {
        name: nm,
        path: p.to_string_lossy().to_string(),
        image: None,
        image_url: None,
        subject_id: None,
        playtime: 0,
        last_played: None,
        folder_path,
        tags: vec![],
    };
    let mut db = load_games_db();
    // avoid duplicates
    if db.games.iter().any(|g| g.path == entry.path) {
        return Ok(entry);
    }
    db.games.push(entry.clone());
    save_games_db(&db)?;
    Ok(entry)
}

#[tauri::command]
pub fn update_game_image(path: &str, image: &str) -> Result<(), String> {
    let mut db = load_games_db();
    let mut changed = false;
    for g in db.games.iter_mut() {
        if g.path == path {
            g.image = Some(image.to_string());
            changed = true;
            break;
        }
    }
    if changed {
        save_games_db(&db)?;
        Ok(())
    } else {
        Err("game not found".into())
    }
}

#[tauri::command]
pub fn update_game_info(path: &str, name: Option<&str>, image: Option<&str>, image_url: Option<&str>, subject_id: Option<i64>) -> Result<serde_json::Value, String> {
    let mut db = load_games_db();
    let mut found = false;
    let mut updated_entry: Option<GameEntry> = None;
    
    for g in db.games.iter_mut() {
        if g.path == path {
            found = true;
            
            // Always apply all updates if provided
            if let Some(n) = name {
                g.name = n.to_string();
            }
            if let Some(img) = image {
                g.image = Some(img.to_string());
            }
            if let Some(url) = image_url {
                g.image_url = Some(url.to_string());
            }
            if let Some(sid) = subject_id {
                g.subject_id = Some(sid);
            }
            
            // Important: Only clone AFTER all updates are applied
            updated_entry = Some(g.clone());
            break;
        }
    }
    
    if found {
        // Always save if we found the game, regardless of whether values changed
        save_games_db(&db)?;
        match updated_entry {
            Some(entry) => {
                let v = serde_json::to_value(entry).map_err(|e| e.to_string())?;
                Ok(v)
            }
            None => Err("failed to get updated entry".into())
        }
    } else {
        Err("game not found".into())
    }
}

#[tauri::command]
pub fn update_game_playtime(path: &str, additional_seconds: i64, last_played: &str) -> Result<i64, String> {
    let mut db = load_games_db();
    let mut found = false;
    let mut total_playtime = 0i64;
    
    for g in db.games.iter_mut() {
        if g.path == path {
            g.playtime += additional_seconds;
            g.last_played = Some(last_played.to_string());
            total_playtime = g.playtime;
            found = true;
            break;
        }
    }
    
    if found {
        save_games_db(&db)?;
        Ok(total_playtime)
    } else {
        Err("game not found".into())
    }
}

#[tauri::command]
pub fn list_games() -> Result<Vec<GameEntry>, String> {
    let db = load_games_db();
    Ok(db.games)
}

#[tauri::command]
pub fn remove_game(path: &str) -> Result<(), String> {
    let mut db = load_games_db();
    db.games.retain(|g| g.path != path);
    save_games_db(&db)?;
    Ok(())
}

// Open a native file dialog (blocking) to pick a single .exe file and return its path.
#[tauri::command]
pub fn pick_exe(initial_dir: Option<String>) -> Result<String, String> {
    // Use rfd (cross-platform) to present a native file dialog
    let mut dialog = rfd::FileDialog::new().add_filter("Executable", &["exe"]);
    
    // Set initial directory if provided
    if let Some(dir) = initial_dir {
        dialog = dialog.set_directory(&dir);
    }
    
    match dialog.pick_file() {
        Some(p) => Ok(p.to_string_lossy().to_string()),
        None => Err("cancelled".into()),
    }
}

// Open a native folder dialog and scan for game folders (each subfolder is treated as one game)
#[tauri::command]
pub fn pick_folder_and_scan() -> Result<Vec<serde_json::Value>, String> {
    match rfd::FileDialog::new().pick_folder() {
        Some(parent_folder) => {
            let mut games = Vec::new();
            
            // Scan each direct subfolder
            if let Ok(entries) = fs::read_dir(&parent_folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Recursively find exe in this folder and its subfolders
                        if let Some((exe_path, mut folder_names)) = find_exe_with_folder_path(&path) {
                            let folder_name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            // 在路径前面插入游戏文件夹自己的名字（用户选择文件夹下的第一层）
                            folder_names.insert(0, folder_name.clone());
                            
                            games.push(serde_json::json!({
                                "path": exe_path,
                                "name": folder_name,
                                "folder_path": folder_names  // 从用户选择的文件夹开始的完整路径
                            }));
                        }
                    }
                }
            }
            
            Ok(games)
        }
        None => Err("cancelled".into()),
    }
}
