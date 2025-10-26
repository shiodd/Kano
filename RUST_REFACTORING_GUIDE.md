# Rust 后端代码重构指南

## 当前状态
当前所有命令都在 `lib.rs` 中，文件有 800+ 行代码。

## 推荐的模块化结构

```
src-tauri/src/
├── main.rs
├── lib.rs                 # 主入口，只包含 run() 函数和命令注册
├── state.rs              # 全局状态管理
├── config.rs             # 配置文件读写
├── utils.rs              # 工具函数
├── models.rs             # 数据结构定义
└── commands/
    ├── mod.rs            # 命令模块入口
    ├── game_library.rs   # 游戏库相关命令
    ├── bangumi.rs        # Bangumi API 相关命令
    └── settings.rs       # 设置相关命令
```

## 各模块职责

### `state.rs` - 全局状态
- `RunningProcesses` 结构体
- 进程跟踪相关逻辑

### `config.rs` - 配置管理
- `AppConfig` 结构体
- `load_config()` / `save_config()`
- 各种路径函数：`config_path()`, `cache_path()`, `games_db_path()`

### `models.rs` - 数据模型
- `GameEntry` 结构体
- `GamesDB` 结构体
- 其他数据结构

### `utils.rs` - 工具函数
- `find_exe_with_folder_path()` - 查找 exe 文件
- `collect_exe_files_with_path()` - 递归收集 exe
- 其他辅助函数

### `commands/game_library.rs` - 游戏库命令
```rust
// 这个文件包含：
- greet()
- launch_exe()
- kill_game()
- pick_exe()
- pick_folder_and_scan()
- add_game()
- list_games()
- remove_game()
- update_game_image()
- update_game_info()
- update_game_playtime()
```

### `commands/bangumi.rs` - Bangumi 命令
```rust
// 这个文件包含：
- search_bangumi()
- get_bangumi_subject()
- load_cache()
- save_cache()
- download_image()
- delete_cached_image()
```

### `commands/settings.rs` - 设置命令
```rust
// 这个文件包含：
- set_access_token()
- get_access_token()
- get_project_root()
- test_network_connection()
```

### `lib.rs` - 主入口
```rust
// lib.rs 应该简洁，只包含：
mod state;
mod config;
mod models;
mod utils;
mod commands;

use state::RunningProcesses;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(RunningProcesses::new())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::launch_exe,
            commands::kill_game,
            commands::pick_exe,
            commands::pick_folder_and_scan,
            commands::add_game,
            commands::list_games,
            commands::remove_game,
            commands::search_bangumi,
            commands::get_bangumi_subject,
            commands::update_game_image,
            commands::update_game_info,
            commands::update_game_playtime,
            commands::set_access_token,
            commands::get_access_token,
            commands::load_cache,
            commands::save_cache,
            commands::download_image,
            commands::get_project_root,
            commands::delete_cached_image,
            commands::test_network_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## 重构步骤建议

1. **创建新文件** - 先创建所有新模块文件
2. **移动数据结构** - 将 `GameEntry`, `GamesDB`, `AppConfig` 移到 `models.rs`
3. **移动配置函数** - 将路径和配置相关函数移到 `config.rs`
4. **移动工具函数** - 将 `find_exe_with_folder_path` 等移到 `utils.rs`
5. **分离命令** - 按功能将命令函数移到对应的 commands 文件
6. **更新 lib.rs** - 添加模块导入和简化 run() 函数
7. **测试** - 每移动一个模块就测试一次编译

## 优势

1. **代码组织清晰** - 每个文件职责单一
2. **易于维护** - 修改某个功能只需要找到对应文件
3. **团队协作** - 多人可以同时编辑不同模块而不冲突
4. **易于测试** - 模块化后可以单独测试每个模块
5. **可扩展性** - 添加新功能时只需要新增模块文件

## 参考项目

许多 Tauri 项目都采用这种模块化结构，例如：
- [tauri-examples](https://github.com/tauri-apps/tauri/tree/dev/examples)
- 大型开源 Tauri 应用

## 注意事项

- 保持向后兼容，确保前端调用的命令名称不变
- 使用 `pub use` 重新导出命令，简化 lib.rs 中的导入
- 每个命令函数前需要保留 `#[tauri::command]` 宏
