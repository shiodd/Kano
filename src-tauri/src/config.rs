use std::fs;
use std::path::PathBuf;
use crate::models::{AppConfig, GamesDB, ToolEntry};

fn app_base_dir() -> PathBuf {
    // Prefer the executable's parent directory (works for packaged app).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            return parent.to_path_buf();
        }
    }
    // Fallback to parent(current_dir()) as before
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    if let Some(parent) = cwd.parent() {
        parent.to_path_buf()
    } else {
        cwd
    }
}

pub fn games_db_path() -> PathBuf {
    let mut p = app_base_dir();
    p.push("kano_data");
    p.push("games_db.json");

    // Migrate from old locations:
    // 1. previous app-local `game_data/games_db.json`
    // 2. previous parent(current_dir())/game_data/games_db.json
    let mut old1 = app_base_dir();
    old1.push("game_data");
    old1.push("games_db.json");
    if old1.exists() && !p.exists() {
        if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
        let _ = fs::rename(&old1, &p);
    }
    let old_cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    if let Some(old_parent) = old_cwd.parent() {
        let mut old2 = old_parent.to_path_buf();
        old2.push("game_data");
        old2.push("games_db.json");
        if old2.exists() && !p.exists() {
            if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
            let _ = fs::rename(&old2, &p);
        }
    }
    p
}

pub fn config_path() -> PathBuf {
    let mut p = app_base_dir();
    p.push("kano_data");
    p.push("tauri_config.json");
    // migrate old config if present (from previous app-local or parent current dir)
    let mut old1 = app_base_dir();
    old1.push("tauri_config.json");
    if old1.exists() && !p.exists() {
        if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
        let _ = fs::rename(&old1, &p);
    }
    let old_cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    if let Some(old_parent) = old_cwd.parent() {
        let mut old2 = old_parent.to_path_buf();
        old2.push("tauri_config.json");
        if old2.exists() && !p.exists() {
            if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
            let _ = fs::rename(&old2, &p);
        }
    }
    p
}

pub fn cache_path() -> PathBuf {
    let mut p = app_base_dir();
    p.push("kano_data");
    p.push("bangumi_cache.json");
    p
}

pub fn images_dir_path() -> PathBuf {
    let mut p = app_base_dir();
    p.push("kano_data");
    p.push("images");
    p
}

pub fn project_root() -> PathBuf {
    app_base_dir()
}

pub fn tools_path() -> PathBuf {
    let mut p = app_base_dir();
    p.push("kano_data");
    p.push("tools.json");

    // migrate from previous app-local or old parent(current_dir()) locations
    let mut old1 = app_base_dir();
    old1.push("tools.json");
    if old1.exists() && !p.exists() {
        if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
        let _ = fs::rename(&old1, &p);
    }
    let old_cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    if let Some(old_parent) = old_cwd.parent() {
        let mut old2 = old_parent.to_path_buf();
        old2.push("tools.json");
        if old2.exists() && !p.exists() {
            if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
            let _ = fs::rename(&old2, &p);
        }
    }
    p
}

pub fn notes_path() -> PathBuf {
    let mut p = app_base_dir();
    p.push("kano_data");
    p.push("notes.json");

    // migrate from old locations (app-local notes.json)
    let mut old1 = app_base_dir();
    old1.push("notes.json");
    if old1.exists() && !p.exists() {
        if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
        let _ = fs::rename(&old1, &p);
    }
    let old_cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    if let Some(old_parent) = old_cwd.parent() {
        let mut old2 = old_parent.to_path_buf();
        old2.push("notes.json");
        if old2.exists() && !p.exists() {
            if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
            let _ = fs::rename(&old2, &p);
        }
    }
    p
}

pub fn load_tools_file() -> Vec<ToolEntry> {
    let path = tools_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<ToolEntry>>(&s) {
            return v;
        }
    }

    // Fallback / migration: if tools are present in tauri_config (AppConfig), migrate them
    let cfg = load_config();
    if !cfg.tools.is_empty() {
        let tools = cfg.tools.clone();
        let _ = save_tools_file(&tools);
        // clear migrated tools from main config to avoid duplication
        let mut newcfg = cfg;
        newcfg.tools = Vec::new();
        let _ = save_config(&newcfg);
        return tools;
    }

    Vec::new()
}

pub fn save_tools_file(tools: &Vec<ToolEntry>) -> Result<(), String> {
    let path = tools_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("create tools dir: {}", e));
        }
    }
    let s = serde_json::to_string_pretty(tools).map_err(|e| e.to_string())?;
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
        return Err("failed to acquire tools lock".into());
    }
    let tmp = path.with_extension(format!("tmp.{}", std::process::id()));
    let res = (|| {
        fs::write(&tmp, s).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(&lock_path);
    res
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

pub fn token_path() -> PathBuf {
    // Use kano_data inside app base dir
    let mut p = app_base_dir();
    p.push("kano_data");
    p.push("tokens.json");

    // migrate from old locations (app-local tokens.json or parent(current_dir())/tokens.json)
    let mut old1 = app_base_dir();
    old1.push("tokens.json");
    if old1.exists() && !p.exists() {
        if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
        let _ = fs::rename(&old1, &p);
    }
    let old_cwd = std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir());
    if let Some(old_parent) = old_cwd.parent() {
        let mut old2 = old_parent.to_path_buf();
        old2.push("tokens.json");
        if old2.exists() && !p.exists() {
            if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
            let _ = fs::rename(&old2, &p);
        }
    }

    p
}

/// Load token from dedicated token file. If not present, try migrating from AppConfig.access_token.
pub fn load_token() -> Option<String> {
    let path = token_path();
    if let Ok(s) = fs::read_to_string(&path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&s) {
            if let Some(tok) = json.get("access_token") {
                if !tok.is_null() {
                    if let Some(tstr) = tok.as_str() {
                        let trimmed = tstr.trim().to_string();
                        if !trimmed.is_empty() {
                            return Some(trimmed);
                        }
                    }
                }
            }
        }
    }

    // fallback / migration from tauri_config.json
    let mut cfg = load_config();
    if let Some(tok) = cfg.access_token.clone() {
        // save to tokens.json
        let _ = save_token(&tok);
        // clear from main config and persist
        cfg.access_token = None;
        let _ = save_config(&cfg);
        return Some(tok);
    }

    None
}

pub fn save_token(token: &str) -> Result<(), String> {
    let path = token_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("create token dir: {}", e));
        }
    }
    // write as JSON { "access_token": ... }
    let obj = if token.trim().is_empty() {
        serde_json::json!({ "access_token": null })
    } else {
        serde_json::json!({ "access_token": token })
    };
    let s = serde_json::to_string_pretty(&obj).map_err(|e| e.to_string())?;
    fs::write(&path, s).map_err(|e| e.to_string())
}

