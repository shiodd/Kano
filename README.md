# Kano

<div align="center">
  <img src="src-tauri/icons/128x128.png" alt="Kano Logo" width="64" height="64">
  
  一个轻量简洁的Galgame管理工具
  
  ![Version](https://img.shields.io/badge/version-0.1.0-blue)
  ![License](https://img.shields.io/badge/license-MIT-green)
  ![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
</div>

##  简介
因市面上大部分相关工具皆会获取到游戏的评分/评价，使得自身主观判断受到影响，再无法沉浸体验某个游戏，于是开发此工具。

##  快速开始

### 安装

1. 从 [Releases](https://github.com/shiodd/testGalManager/releases) 页面下载最新版本的安装包
2. 运行 `kano_x.x.x_x64-setup.exe`
3. 按照安装向导完成安装

### 使用

#### 添加游戏

**方式一：单个添加**
1. 点击"选择 EXE"按钮
2. 选择游戏的可执行文件
3. 游戏自动添加到库中

**方式二：批量扫描**
1. 点击"扫描文件夹"按钮
2. 选择包含多个游戏的父文件夹
3. 应用会自动识别每个子文件夹中的游戏
4. 勾选需要添加的游戏，点击"添加选中游戏"

#### 配置 Bangumi Token

为了访问完整的 Bangumi API：

1. 进入"设置"页面
2. 访问 [Bangumi 个人令牌](https://next.bgm.tv/demo/access-token) 创建 Access Token
3. 将 Token 粘贴到设置中并保存


## 功能说明

### 游戏库功能

- 添加/删除游戏
- 游戏搜索和筛选
- 游戏详情
- 游戏时长统计

### 设置功能

- Bangumi Access Token 配置
- 网络连接测试


## 已知问题

- 部分游戏可能无法正确识别执行文件
- Bangumi API 在未配置 Token 时无法访问完整内容

## 计划

- [ ] 游戏分类/标签功能
- [ ] 更多游戏信息源（如VNDB）
- [ ] 游戏启动参数配置
- [ ] 云同步功能
- [ ] 主题切换
- [ ] 多语言支持

=======
# Kano
>>>>>>> 3cb86f9d88bb9b23f0c629fec7a0ac65dcb2e050
