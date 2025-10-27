use std::fs;
use crate::config::{load_config, cache_path, images_dir_path};

#[tauri::command]
pub fn search_bangumi(query: &str, filter: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
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
pub fn get_bangumi_subject(id: i64) -> Result<serde_json::Value, String> {
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
pub fn load_cache() -> Result<serde_json::Value, String> {
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
pub fn save_cache(cache: serde_json::Value) -> Result<(), String> {
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
pub async fn download_image(url: &str, subject_id: i64) -> Result<String, String> {
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
    
    // Return the relative path (relative to exe directory)
    Ok(format!("game_data/images/{}", filename))
}

// Delete a cached image by subject_id
#[tauri::command]
pub fn delete_cached_image(subject_id: i64) -> Result<(), String> {
    let images_dir = images_dir_path();
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
    
    Ok(())
}
