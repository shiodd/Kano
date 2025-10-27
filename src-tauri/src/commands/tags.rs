use crate::config::{load_games_db, save_games_db};

// 获取所有可用标签
#[tauri::command]
pub fn get_all_tags() -> Result<Vec<String>, String> {
    let mut db = load_games_db();
    
    // 如果数据库中没有标签，初始化默认标签
    if db.custom_tags.is_empty() {
        db.custom_tags = vec![
            "未玩".to_string(),
            "正在玩".to_string(),
            "已通关".to_string(),
        ];
        let _ = save_games_db(&db);
    }
    
    Ok(db.custom_tags.clone())
}

// 添加自定义标签
#[tauri::command]
pub fn add_custom_tag(tag: &str) -> Result<(), String> {
    if tag.trim().is_empty() {
        return Err("标签名称不能为空".to_string());
    }
    
    let mut db = load_games_db();
    
    // 检查是否已存在
    if db.custom_tags.contains(&tag.to_string()) {
        return Err("标签已存在".to_string());
    }
    
    db.custom_tags.push(tag.to_string());
    save_games_db(&db)?;
    Ok(())
}

// 删除标签（包括默认标签和自定义标签）
#[tauri::command]
pub fn remove_custom_tag(tag: &str) -> Result<(), String> {
    let mut db = load_games_db();
    
    // 从自定义标签列表中移除（如果存在）
    db.custom_tags.retain(|t| t != tag);
    
    // 从所有游戏中移除这个标签
    for game in db.games.iter_mut() {
        game.tags.retain(|t| t != tag);
    }
    
    save_games_db(&db)?;
    Ok(())
}

// 为游戏添加标签
#[tauri::command]
pub fn add_tag_to_game(path: &str, tag: &str) -> Result<(), String> {
    let mut db = load_games_db();
    
    for game in db.games.iter_mut() {
        if game.path == path {
            if !game.tags.contains(&tag.to_string()) {
                game.tags.push(tag.to_string());
                save_games_db(&db)?;
                return Ok(());
            }
            return Ok(());
        }
    }
    
    Err("游戏不存在".to_string())
}

// 从游戏移除标签
#[tauri::command]
pub fn remove_tag_from_game(path: &str, tag: &str) -> Result<(), String> {
    let mut db = load_games_db();
    
    for game in db.games.iter_mut() {
        if game.path == path {
            game.tags.retain(|t| t != tag);
            save_games_db(&db)?;
            return Ok(());
        }
    }
    
    Err("游戏不存在".to_string())
}

// 获取指定标签的游戏数量
#[tauri::command]
pub fn get_games_count_by_tag(tag: &str) -> Result<usize, String> {
    let db = load_games_db();
    let count = db.games.iter()
        .filter(|game| game.tags.contains(&tag.to_string()))
        .count();
    Ok(count)
}
