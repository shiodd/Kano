use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameEntry {
    pub name: String,
    pub path: String,
    pub image: Option<String>,       // 本地图片路径
    #[serde(default)]
    pub image_url: Option<String>,   // 网络图片URL
    pub subject_id: Option<i64>,
    #[serde(default)]
    pub playtime: i64, // 总游戏时长（秒）
    #[serde(default)]
    pub last_played: Option<String>, // 上次游玩时间 (ISO 8601 格式)
    #[serde(default)]
    pub folder_path: Option<Vec<String>>, // 从用户选择的文件夹开始的文件夹路径
    #[serde(default)]
    pub tags: Vec<String>, // 游戏标签
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GamesDB {
    pub games: Vec<GameEntry>,
    #[serde(default)]
    pub custom_tags: Vec<String>, // 用户自定义标签
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AppConfig {
    pub access_token: Option<String>,
}
