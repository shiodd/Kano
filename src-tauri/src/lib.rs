// Minimal Tauri backend: expose a greet and launch_exe command.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
// reqwest Client is used inline; no extra import required here

// A simple greeting kept for compatibility
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Launch an executable by path
#[tauri::command]
fn launch_exe(path: &str) -> Result<(), String> {
    let p = PathBuf::from(path);
    if !p.exists() || !p.is_file() {
        return Err("executable not found".into());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(&p)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new(path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameEntry {
    name: String,
    path: String,
    image: Option<String>,
    subject_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct GamesDB {
    games: Vec<GameEntry>,
}

fn games_db_path() -> PathBuf {
    // prefer parent directory (project root) when available to avoid writing inside src-tauri
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    let mut p = if let Some(parent) = cwd.parent() {
        parent.to_path_buf()
    } else {
        cwd
    };
    p.push("game_data");
    p.push("games_db.json");
    p
}

fn config_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    let mut p = if let Some(parent) = cwd.parent() {
        parent.to_path_buf()
    } else {
        cwd
    };
    p.push("tauri_config.json");
    p
}

fn cache_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    let mut p = if let Some(parent) = cwd.parent() {
        parent.to_path_buf()
    } else {
        cwd
    };
    p.push("game_data");
    p.push("bangumi_cache.json");
    p
}

fn images_dir_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    let mut p = if let Some(parent) = cwd.parent() {
        parent.to_path_buf()
    } else {
        cwd
    };
    p.push("game_data");
    p.push("images");
    p
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct AppConfig {
    access_token: Option<String>,
}

fn load_config() -> AppConfig {
    let path = config_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(c) = serde_json::from_str::<AppConfig>(&s) {
            return c;
        }
    }
    AppConfig { access_token: None }
}

fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("failed to create config dir: {}", e);
        }
    }
    let s = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    // simple lock file path
    let lock_path = path.with_extension("lock");
    // try to acquire lock by creating the lock file atomically
    let mut acquired = false;
    let mut attempts = 0;
    while !acquired && attempts < 50 {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(_) => acquired = true,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                attempts += 1;
            }
        }
    }
    if !acquired {
        return Err("failed to acquire config lock".into());
    }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    let res = (|| {
        fs::write(&tmp, s).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(&lock_path);
    res
}

fn load_games_db() -> GamesDB {
    let path = games_db_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(db) = serde_json::from_str::<GamesDB>(&s) {
            return db;
        }
    }
    GamesDB { games: Vec::new() }
}

fn save_games_db(db: &GamesDB) -> Result<(), String> {
    let path = games_db_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("failed to create games db dir: {}", e);
        }
    }
    let s = serde_json::to_string_pretty(db).map_err(|e| e.to_string())?;
    let lock_path = path.with_extension("lock");
    let mut acquired = false;
    let mut attempts = 0;
    while !acquired && attempts < 50 {
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&lock_path) {
            Ok(_) => acquired = true,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                attempts += 1;
            }
        }
    }
    if !acquired {
        return Err("failed to acquire games db lock".into());
    }
    let res = (|| {
        let mut tmp = path.clone();
        tmp.set_extension(format!("tmp.{}", std::process::id()));
        fs::write(&tmp, s).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(&lock_path);
    res
}

#[tauri::command]
fn add_game(path: &str, name: Option<&str>) -> Result<GameEntry, String> {
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
        subject_id: None,
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
fn update_game_image(path: &str, image: &str) -> Result<(), String> {
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
fn update_game_info(path: &str, name: Option<&str>, image: Option<&str>, subject_id: Option<i64>) -> Result<serde_json::Value, String> {
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
fn list_games() -> Result<Vec<GameEntry>, String> {
    let db = load_games_db();
    Ok(db.games)
}

#[tauri::command]
fn remove_game(path: &str) -> Result<(), String> {
    let mut db = load_games_db();
    db.games.retain(|g| g.path != path);
    save_games_db(&db)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            launch_exe,
            pick_exe,
            pick_folder_and_scan,
            add_game,
            list_games,
            remove_game,
            search_bangumi,
            get_bangumi_subject,
            update_game_image,
            update_game_info,
            set_access_token,
            get_access_token,
            load_cache,
            save_cache,
            download_image,
            get_project_root,
            delete_cached_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Open a native file dialog (blocking) to pick a single .exe file and return its path.
#[tauri::command]
fn pick_exe(initial_dir: Option<String>) -> Result<String, String> {
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
fn pick_folder_and_scan() -> Result<Vec<serde_json::Value>, String> {
    match rfd::FileDialog::new().pick_folder() {
        Some(parent_folder) => {
            let mut games = Vec::new();
            
            // Scan each direct subfolder
            if let Ok(entries) = fs::read_dir(&parent_folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Find the first .exe file in this folder
                        if let Some(exe_path) = find_first_exe_in_folder(&path) {
                            let folder_name = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            games.push(serde_json::json!({
                                "path": exe_path,
                                "name": folder_name
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

fn find_first_exe_in_folder(folder: &PathBuf) -> Option<String> {
    let mut all_exes = Vec::new();
    
    // Collect all exe files recursively
    collect_exe_files(folder, &mut all_exes);
    
    if all_exes.is_empty() {
        return None;
    }
    
    // Prioritize exe files with Chinese keywords
    let chinese_keywords = ["ch", "chs", "cn", "中文", "chinese", "简体", "繁体", "汉化"];
    
    for exe_path in &all_exes {
        let file_name = exe_path.to_lowercase();
        for keyword in &chinese_keywords {
            if file_name.contains(keyword) {
                return Some(exe_path.clone());
            }
        }
    }
    
    // If no Chinese version found, return the first exe
    all_exes.into_iter().next()
}

fn collect_exe_files(folder: &PathBuf, result: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("exe") {
                        result.push(path.to_string_lossy().to_string());
                    }
                }
            } else if path.is_dir() {
                collect_exe_files(&path, result);
            }
        }
    }
}

#[tauri::command]
fn search_bangumi(query: &str, filter: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
    let url = "https://api.bgm.tv/v0/search/subjects";
    let mut body = serde_json::Map::new();
    body.insert("keyword".to_string(), serde_json::Value::String(query.to_string()));
    if let Some(f) = filter {
        body.insert("filter".to_string(), f);
    }
    let client = reqwest::blocking::Client::new();
    let mut req = client
        .post(url)
        .json(&serde_json::Value::Object(body))
        .header("User-Agent", "shiodd/my-private-project");
    if let Some(tok) = load_config().access_token {
        if !tok.is_empty() {
            req = req.bearer_auth(tok);
        }
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    let v: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    Ok(v)
}

#[tauri::command]
fn get_bangumi_subject(id: i64) -> Result<serde_json::Value, String> {
    // Use the v0 subjects endpoint (plural) and include Accept header.
    let url = format!("https://api.bgm.tv/v0/subjects/{}", id);
    let client = reqwest::blocking::Client::new();
    let mut req = client
        .get(&url)
        .header("User-Agent", "shiodd/my-private-project")
        .header("Accept", "application/json");

    // attach bearer token from saved config if present
    if let Some(tok) = load_config().access_token {
        if !tok.is_empty() {
            req = req.bearer_auth(tok);
        }
    }

    let resp = req.send().map_err(|e| e.to_string())?;
    let status = resp.status();
    // read the response body as text so we can return useful errors when non-2xx
    let body_text = resp.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status.as_u16(), body_text));
    }
    let v: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| e.to_string())?;
    Ok(v)
}

#[tauri::command]
fn set_access_token(token: &str) -> Result<(), String> {
    let mut cfg = load_config();
    cfg.access_token = if token.is_empty() { None } else { Some(token.to_string()) };
    save_config(&cfg)
}

#[tauri::command]
fn get_access_token() -> Result<Option<String>, String> {
    Ok(load_config().access_token)
}

#[tauri::command]
fn load_cache() -> Result<serde_json::Value, String> {
    let path = cache_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(cache) = serde_json::from_str::<serde_json::Value>(&s) {
            return Ok(cache);
        }
    }
    // return empty object if no cache file or parse error
    Ok(serde_json::json!({}))
}

#[tauri::command]
fn save_cache(cache: serde_json::Value) -> Result<(), String> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("create dir failed: {}", e));
        }
    }
    let s = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
    fs::write(&path, s).map_err(|e| e.to_string())
}

// Download image from URL and save it locally with the given subject_id as filename
#[tauri::command]
async fn download_image(url: &str, subject_id: i64) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client.get(url)
        .header("User-Agent", "shiodd/my-private-project")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status().as_u16()));
    }
    
    // Get the image bytes
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    
    // Determine file extension from URL or content-type
    let ext = if url.contains(".jpg") || url.contains(".jpeg") {
        "jpg"
    } else if url.contains(".png") {
        "png"
    } else if url.contains(".webp") {
        "webp"
    } else {
        "jpg" // default
    };
    
    // Create images directory
    let images_dir = images_dir_path();
    if let Err(e) = fs::create_dir_all(&images_dir) {
        return Err(format!("create images dir failed: {}", e));
    }
    
    // Save image as {subject_id}.{ext}
    let filename = format!("{}.{}", subject_id, ext);
    let mut image_path = images_dir.clone();
    image_path.push(&filename);
    
    fs::write(&image_path, bytes).map_err(|e| e.to_string())?;
    
    // Return the relative path for easier portability
    Ok(format!("game_data/images/{}", filename))
}

// Get the project root directory (parent of src-tauri)
#[tauri::command]
fn get_project_root() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let root = if let Some(parent) = cwd.parent() {
        parent.to_path_buf()
    } else {
        cwd
    };
    Ok(root.to_string_lossy().to_string())
}

// Delete cached image file for a subject_id
#[tauri::command]
fn delete_cached_image(subject_id: i64) -> Result<(), String> {
    let images_dir = images_dir_path();
    
    // Try common image extensions
    let extensions = ["jpg", "jpeg", "png", "webp"];
    for ext in &extensions {
        let filename = format!("{}.{}", subject_id, ext);
        let mut image_path = images_dir.clone();
        image_path.push(&filename);
        
        if image_path.exists() {
            fs::remove_file(&image_path).map_err(|e| e.to_string())?;
            return Ok(());
        }
    }
    
    // Image not found, but that's okay
    Ok(())
}
