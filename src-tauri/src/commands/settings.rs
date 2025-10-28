use crate::config::{load_config, save_config, load_tools_file, save_tools_file, load_token, save_token};
use crate::models::ToolEntry;
use uuid::Uuid;

#[tauri::command]
pub fn set_access_token(token: &str) -> Result<(), String> {
    if token.is_empty() {
        // clear token file by writing empty string
        return save_token("");
    }
    save_token(token)
}

#[tauri::command]
pub fn get_access_token() -> Result<Option<String>, String> {
    Ok(load_token())
}

#[tauri::command]
pub fn get_tools() -> Result<Vec<ToolEntry>, String> {
    Ok(load_tools_file())
}

#[tauri::command]
pub fn add_tool(name: &str, path: &str) -> Result<ToolEntry, String> {
    let mut tools = load_tools_file();
    let id = Uuid::new_v4().to_string();
    let entry = ToolEntry { id: id.clone(), name: name.to_string(), path: path.to_string() };
    tools.push(entry.clone());
    save_tools_file(&tools)?;
    Ok(entry)
}

#[tauri::command]
pub fn remove_tool(id: &str) -> Result<(), String> {
    let mut tools = load_tools_file();
    tools.retain(|t| t.id != id);
    save_tools_file(&tools)?;
    Ok(())
}

#[tauri::command]
pub fn launch_tool(id: &str) -> Result<(), String> {
    let tools = load_tools_file();
    if let Some(tool) = tools.iter().find(|t| t.id == id) {
        let mut cmd = std::process::Command::new(&tool.path);
        if let Some(parent) = std::path::Path::new(&tool.path).parent() {
            cmd.current_dir(parent);
        }
        cmd.spawn().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("tool not found".into())
    }
}

// Get the project root directory (parent of src-tauri)
#[tauri::command]
pub fn get_project_root() -> Result<String, String> {
    // Use the app-local project root (prefer executable parent). This keeps
    // behavior consistent with functions in `config.rs` such as
    // `games_db_path()` / `images_dir_path()` which use `app_base_dir()`.
    let root = crate::config::project_root();
    Ok(root.to_string_lossy().to_string())
}

// Test network connectivity to bgm.tv and return latency
#[tauri::command]
pub fn test_network_connection() -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("testGalManager/1.0")
        .build()
        .map_err(|e| e.to_string())?;
    
    // 获取 access token
    let token = load_token();
    
    // 测试基础连接
    let start = std::time::Instant::now();
    let base_result = client.get("https://bgm.tv").send();
    let base_ok = base_result.is_ok();
    let base_latency = if base_ok { Some(start.elapsed().as_millis() as u64) } else { None };
    
    // 测试普通游戏 API (ID: 2288)
    let start = std::time::Instant::now();
    let mut normal_request = client.get("https://api.bgm.tv/v0/subjects/2288")
        .header("User-Agent", "testGalManager/1.0");
    
    if let Some(ref t) = token {
        if !t.is_empty() {
            normal_request = normal_request.header("Authorization", format!("Bearer {}", t));
        }
    }
    
    let normal_game_result = normal_request.send();
    let normal_game_ok = match normal_game_result {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false
    };
    let normal_game_latency = if normal_game_ok { Some(start.elapsed().as_millis() as u64) } else { None };
    
    // 测试 NSFW 游戏 API (ID: 165894) - 需要 token
    let start = std::time::Instant::now();
    let mut nsfw_request = client.get("https://api.bgm.tv/v0/subjects/165894")
        .header("User-Agent", "testGalManager/1.0");
    
    if let Some(ref t) = token {
        if !t.is_empty() {
            nsfw_request = nsfw_request.header("Authorization", format!("Bearer {}", t));
        }
    }
    
    let nsfw_game_result = nsfw_request.send();
    let nsfw_game_ok = match nsfw_game_result {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false
    };
    let nsfw_game_latency = if nsfw_game_ok { Some(start.elapsed().as_millis() as u64) } else { None };
    
    Ok(serde_json::json!({
        "success": base_ok,
        "latency": base_latency,
        "normalGameApi": normal_game_ok,
        "normalGameLatency": normal_game_latency,
        "nsfwGameApi": nsfw_game_ok,
        "nsfwGameLatency": nsfw_game_latency
    }))
}
