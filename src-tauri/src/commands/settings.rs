use crate::config::{load_config, save_config};

#[tauri::command]
pub fn set_access_token(token: &str) -> Result<(), String> {
    let mut cfg = load_config();
    cfg.access_token = if token.is_empty() { None } else { Some(token.to_string()) };
    save_config(&cfg)
}

#[tauri::command]
pub fn get_access_token() -> Result<Option<String>, String> {
    Ok(load_config().access_token)
}

// Get the project root directory (exe directory)
#[tauri::command]
pub fn get_project_root() -> Result<String, String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
        .ok_or("Failed to get exe directory")?;
    
    Ok(exe_dir.to_string_lossy().to_string())
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
    let token = load_config().access_token;
    
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
