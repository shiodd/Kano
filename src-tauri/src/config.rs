use std::fs;
use std::path::PathBuf;
use crate::models::{AppConfig, GamesDB};

// Get the directory where the exe file is located
fn get_app_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir()))
}

pub fn games_db_path() -> PathBuf {
    let mut p = get_app_dir();
    p.push("game_data");
    p.push("games_db.json");
    p
}

pub fn config_path() -> PathBuf {
    let mut p = get_app_dir();
    p.push("tauri_config.json");
    p
}

pub fn cache_path() -> PathBuf {
    let mut p = get_app_dir();
    p.push("game_data");
    p.push("bangumi_cache.json");
    p
}

pub fn images_dir_path() -> PathBuf {
    let mut p = get_app_dir();
    p.push("game_data");
    p.push("images");
    p
}

pub fn project_root() -> PathBuf {
    get_app_dir()
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(c) = serde_json::from_str::<AppConfig>(&s) {
            return c;
        }
    }
    AppConfig::default()
}

pub fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("failed to create config dir: {}", e);
        }
    }
    let s = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
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

pub fn load_games_db() -> GamesDB {
    let path = games_db_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(db) = serde_json::from_str::<GamesDB>(&s) {
            return db;
        }
    }
    GamesDB::default()
}

pub fn save_games_db(db: &GamesDB) -> Result<(), String> {
    let path = games_db_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("create db dir: {}", e));
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
        return Err("failed to acquire db lock".into());
    }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    let res = (|| {
        fs::write(&tmp, s).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(&lock_path);
    res
}

