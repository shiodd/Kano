use std::fs;
use std::path::PathBuf;
use std::env;
use crate::models::{AppConfig, GamesDB};

fn app_base_dir() -> PathBuf {
    // Prefer the executable's parent directory (installed case). Fall back to the
    // previous behavior (parent of current working dir) and finally to temp dir.
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.to_path_buf();
        }
    }
    let cwd = env::current_dir().unwrap_or_else(|_| env::temp_dir());
    if let Some(parent) = cwd.parent() {
        parent.to_path_buf()
    } else {
        cwd
    }
}

pub fn games_db_path() -> PathBuf {
    let base = app_base_dir();
    let new_dir = base.join("game_data");

    // Try to migrate existing data from the old location (parent of cwd) if present.
    let old_cwd = env::current_dir().unwrap_or_else(|_| env::temp_dir());
    let old_base = if let Some(parent) = old_cwd.parent() { parent.to_path_buf() } else { old_cwd };
    let old_dir = old_base.join("game_data");
    if old_dir.exists() && !new_dir.exists() {
        // Best-effort move; ignore errors to avoid blocking startup
        let _ = fs::rename(&old_dir, &new_dir);
    }

    new_dir.join("games_db.json")
}

pub fn config_path() -> PathBuf {
    let base = app_base_dir();
    base.join("tauri_config.json")
}

pub fn cache_path() -> PathBuf {
    let base = app_base_dir();
    base.join("game_data").join("bangumi_cache.json")
}

pub fn images_dir_path() -> PathBuf {
    let base = app_base_dir();
    base.join("game_data").join("images")
}

pub fn project_root() -> PathBuf {
    app_base_dir()
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

