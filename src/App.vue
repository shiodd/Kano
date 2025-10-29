<script setup>
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Settings from './pages/Settings.vue';
import About from './pages/About.vue';
import TagManagement from './pages/TagManagement.vue';
import Toolbox from './pages/Toolbox.vue';
import Notes from './pages/Notes.vue';
import Sidebar from './components/Sidebar.vue';
import GameLibrary from './pages/GameLibrary.vue';
import GameDetailModal from './components/GameDetailModal.vue';
import ReplaceModal from './components/ReplaceModal.vue';
import { useCache } from './composables/useCache';
import { useGameLibrary } from './composables/useGameLibrary';
import { useImageFetch } from './composables/useImageFetch';
import gameService from './services/gameService';

const greetMsg = ref("");
const name = ref("");

// 使用 composables / services
const { detailCache, getDetailCache, setDetailCache, removeDetailCache, clearAllCache, loadCacheFromFile, saveCacheToFile } = useCache();
const { games, isLoadingGames, projectRoot, loadProjectRoot, listGames: loadGames, addGame, removeGame: removeGameService, updateGameInfo, pickExe: pickExeService, pickFolderAndScan: pickFolderAndScanService, launchExe: launchExeService, killGame: killGameService, listExes: listExesService, updateGamePlaytime: updateGamePlaytimeService } = useGameLibrary();
const { imageFetchRunning, fetchImageForGame, autoFetchImages, loadedGamesCount, totalGamesCount } = useImageFetch(games);

// 帮助函数：获取图片源（将本地路径转换为 Tauri 可访问的文件 URL）
function getImageSrc(imagePath) {
  if (!imagePath) return null;
  // 如果是 http/https URL，直接返回
  if (imagePath.startsWith('http://') || imagePath.startsWith('https://')) {
    return imagePath;
  }
  // 支持旧的 game_data/ 前缀，同时优先识别新的 kano_data/
  if (imagePath.startsWith('kano_data/') || imagePath.startsWith('game_data/')) {
    // 兼容：如果是旧前缀 game_data/，将其视为 kano_data/
    const fixed = imagePath.startsWith('game_data/') ? imagePath.replace(/^game_data\//, 'kano_data/') : imagePath;
    const absolutePath = `${projectRoot.value}\\${fixed.replace(/\//g, '\\\\')}`;
    return convertFileSrc(absolutePath);
  }
  // 其他情况（绝对路径）直接转换
  return convertFileSrc(imagePath);
}

// 详情缓存由 useCache composable 管理（getDetailCache / setDetailCache / 等方法）

async function greet() {
  // 关于 Tauri 命令的更多信息，参见：https://tauri.app/develop/calling-rust/
  greetMsg.value = await gameService.greet(name.value);
}

// 最小化示例：用户选择 EXE 并启动它
const selectedExe = ref("");
const runningGames = ref(new Set()); // 追踪正在运行的游戏路径
const gameStartTimes = ref(new Map()); // 记录游戏启动时间 path -> timestamp

// 多选状态
const selectedGames = ref(new Set());
const isMultiSelectMode = ref(false);

// 菜单状态
const showMenu = ref(false);

// Toast（提示） 状态
const toastMessage = ref('');
const toastVisible = ref(false);

// 过滤器状态
const selectedFilter = ref('全部'); // '全部', 'ADV', 'RPG'
const searchKeyword = ref(''); // 搜索关键词
const selectedTag = ref(null); // 选中的标签

// 计算属性：过滤后的游戏列表
const filteredGames = computed(() => {
  let result = games.value;
  
  // Apply tag filter (优先级最高)
  if (selectedTag.value) {
    result = result.filter(game => {
      return game.tags && game.tags.includes(selectedTag.value);
    });
  }
  
  // Apply type filter
  if (selectedFilter.value !== '全部') {
    result = result.filter(game => {
      if (!game.subject_id) return false;
      
      const cacheData = getDetailCache(game.subject_id);
      if (!cacheData || !cacheData.meta_tags) return false;
      
      return cacheData.meta_tags.includes(selectedFilter.value);
    });
  }
  
  // Apply search filter
  if (searchKeyword.value.trim()) {
    const keyword = searchKeyword.value.toLowerCase().trim();
    result = result.filter(game => {
      // Search in game name
      if (game.name && game.name.toLowerCase().includes(keyword)) {
        return true;
      }
      
      // Search in cache data (Chinese name, original name)
      if (game.subject_id) {
        const cacheData = getDetailCache(game.subject_id);
        if (cacheData) {
          if (cacheData.name_cn && cacheData.name_cn.toLowerCase().includes(keyword)) {
            return true;
          }
          if (cacheData.name && cacheData.name.toLowerCase().includes(keyword)) {
            return true;
          }
        }
      }
      
      return false;
    });
  }
  
  return result;
});

function setFilter(filter) {
  selectedFilter.value = filter;
}

// Toast 通知
function showToast(message) {
  toastMessage.value = message;
  toastVisible.value = true;
  setTimeout(() => {
    toastVisible.value = false;
  }, 2000);
}

function toggleMultiSelect() {
  isMultiSelectMode.value = !isMultiSelectMode.value;
  if (!isMultiSelectMode.value) {
    selectedGames.value.clear();
  }
}

function toggleGameSelection(game) {
  if (selectedGames.value.has(game.path)) {
    selectedGames.value.delete(game.path);
  } else {
    selectedGames.value.add(game.path);
  }
}

function selectAllGames() {
  filteredGames.value.forEach(g => selectedGames.value.add(g.path));
}

function deselectAllGames() {
  selectedGames.value.clear();
}

async function deleteSelectedGames() {
  const count = selectedGames.value.size;
  if (count === 0) {
    alert('请先选择要删除的游戏');
    return;
  }
  
  if (!confirm(`确定要删除选中的 ${count} 个游戏吗？`)) {
    return;
  }
  
  try {
    const pathsToDelete = Array.from(selectedGames.value);
    for (const path of pathsToDelete) {
      const game = games.value.find(g => g.path === path);
      if (game) {
        if (game.subject_id) {
          removeDetailCache(game.subject_id);
          // 同时删除缓存的图片文件
          try {
            await gameService.deleteCachedImage({ subjectId: game.subject_id });
          } catch (err) {
            console.error('删除缓存图片失败:', err);
            // 不阻止删除流程
          }
        }
        await removeGameService(path);
      }
    }
    selectedGames.value.clear();
  await loadGames();
    isMultiSelectMode.value = false; // 自动退出多选模式
  } catch (e) {
    alert('批量删除失败: ' + e);
  }
}

// `listGames` 功能已迁移到 useGameLibrary composable（对外以 loadGames 暴露）


onMounted(async () => {
  try {
    await loadProjectRoot();
  } catch (e) {
    console.error('Failed to get project root:', e);
  }
  await loadCacheFromFile(); // Load cache first
  await loadGames();
  // 启动后台任务，为没有图片的游戏抓取封面/详情
  try { await autoFetchImages(); } catch (e) { /* ignore */ }
  
  // 监听游戏退出事件
  listen('game-exited', async (event) => {
    const gamePath = event.payload;
    await updatePlaytime(gamePath);
    runningGames.value.delete(gamePath);
  });
});


// UI: which pane is shown in main area. 'library', 'test', or 'settings'
const activeTab = ref('library');
const bangumiLoading = ref(false);
// imageFetchRunning、isLoadingGames、loadedGamesCount、totalGamesCount 由 composable 提供

function formatSeconds(s) {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  return `${h}h ${m}m ${sec}s`;
}

async function scan() {
  // 已移除（占位）
}

async function startGame(g) {
  // 已移除（占位）
}

async function stopGame(g) {
  // 已移除（占位）
}

async function pickExe() {
  try {
  const path = await pickExeService(null);
    if (!path) return;
    selectedExe.value = String(path);
  } catch (e) {
    // 用户取消时不提示错误
    if (e === 'cancelled' || e === 'canceled') return;
    alert('选择失败: ' + e);
  }
}

const isScanningFolder = ref(false);
const scannedGames = ref([]);
const selectedScannedGames = ref(new Set());

async function scanFolder() {
  try {
    isScanningFolder.value = true;
    selectedScannedGames.value = new Set(); // 重置选中状态
    const games = await pickFolderAndScanService();
    if (!games || games.length === 0) {
      alert('文件夹中没有找到游戏');
      return;
    }
    scannedGames.value = games;
    // 显示扫描结果，让用户选择要添加的游戏
  } catch (e) {
    if (e !== 'cancelled') {
      alert('扫描失败: ' + e);
    }
  } finally {
    isScanningFolder.value = false;
  }
}

async function addScannedGames(selectedGames) {
  try {
    for (const game of selectedGames) {
      await addGame({ path: game.path, name: game.name, folderPath: game.folder_path });
    }
    scannedGames.value = [];
    selectedScannedGames.value = new Set();
  await loadGames();
    // Try to fetch images/details for newly added games immediately
    try { await autoFetchImages(); } catch (e) { /* ignore */ }
  } catch (e) {
    alert('添加失败: ' + e);
  }
}

function toggleScannedGame(index) {
  if (selectedScannedGames.value.has(index)) {
    selectedScannedGames.value.delete(index);
  } else {
    selectedScannedGames.value.add(index);
  }
}

function addSelectedScannedGames() {
  const selected = scannedGames.value.filter((_, i) => selectedScannedGames.value.has(i));
  if (selected.length === 0) {
    alert('请至少选择一个游戏');
    return;
  }
  addScannedGames(selected);
}

async function launchSelected() {
  if (!selectedExe.value) return alert('请先选择一个 EXE');
  try {
    await launchExeService({ path: selectedExe.value });
  } catch (e) {
    alert('启动失败: ' + e);
  }
}

async function addSelectedToLibrary() {
  if (!selectedExe.value) return showToast('请先选择一个 EXE');
  try {
    await addGame({ path: selectedExe.value, name: null, folderPath: null });
    await loadGames();
    // Fetch images/details for the new game
    try { await autoFetchImages(); } catch (e) { /* ignore */ }
    selectedExe.value = ''; // 清空选中状态
    showToast('加入成功');
  } catch (e) {
    showToast('加入失败');
  }
}

async function removeGame(g) {
  // 添加确认对话框
  if (!confirm(`确定要删除游戏"${g.name}"吗？`)) {
    return false; // 用户取消，返回 false
  }
  
  try {
    // if image fetching is running, wait briefly for it to finish to avoid race
    let attempts = 0;
    while (imageFetchRunning.value && attempts < 50) {
      // wait up to ~5s (50 * 100ms)
      await new Promise((r) => setTimeout(r, 100));
      attempts++;
    }
    // Remove cache for this game's detail if it exists
    if (g.subject_id) {
      removeDetailCache(g.subject_id);
      // 同时删除缓存的图片文件
      try {
        await gameService.deleteCachedImage({ subjectId: g.subject_id });
      } catch (err) {
        console.error('删除缓存图片失败:', err);
        // 不阻止删除游戏流程
      }
    }
    await removeGameService(g.path);
    await loadGames();
    return true; // 删除成功，返回 true
  } catch (e) {
    alert('删除失败: ' + e);
    return false; // 删除失败，返回 false
  }
}

// image fetch logic moved to useImageFetch composable (fetchImageForGame / autoFetchImages)

async function launchFromLibrary(g) {
  // 检查游戏是否已在运行中
  if (runningGames.value.has(g.path)) {
    alert('该游戏已在运行中');
    return;
  }
  
  try {
    runningGames.value.add(g.path); // 标记为运行中
    gameStartTimes.value.set(g.path, Date.now()); // 记录启动时间
    await launchExeService(g.path);
    // 进程监控会在游戏关闭时自动移除运行状态
  } catch (e) {
    runningGames.value.delete(g.path); // 启动失败时移除状态
    gameStartTimes.value.delete(g.path);
    alert('启动失败: ' + e);
  }
}

// 手动关闭游戏（终止进程）
async function closeGame(gamePath) {
  try {
    await killGameService(gamePath);
    await updatePlaytime(gamePath);
    runningGames.value.delete(gamePath);
  } catch (e) {
    // 即使终止失败，也移除运行状态（可能进程已经不存在了）
    await updatePlaytime(gamePath);
    runningGames.value.delete(gamePath);
    alert('关闭游戏失败: ' + e);
  }
}

// 更新游戏时长
async function updatePlaytime(gamePath) {
  if (!gameStartTimes.value.has(gamePath)) return;
  
  const startTime = gameStartTimes.value.get(gamePath);
  const playedSeconds = Math.floor((Date.now() - startTime) / 1000);
  const lastPlayed = new Date().toISOString();
  
  try {
    const totalPlaytime = await updateGamePlaytimeService({ 
      path: gamePath, 
      additionalSeconds: playedSeconds,
      lastPlayed: lastPlayed
    });
    
    // 更新本地游戏列表中的时长和最后游玩时间
    const game = games.value.find(g => g.path === gamePath);
    if (game) {
      game.playtime = totalPlaytime;
      game.last_played = lastPlayed;
    }
  } catch (e) {
    // 更新游戏时长失败
  } finally {
    gameStartTimes.value.delete(gamePath);
  }
}

// 格式化游戏时长显示
function formatPlaytime(seconds) {
  if (!seconds || seconds === 0) return '0分钟';
  
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}小时${minutes}分钟`;
  }
  return `${minutes}分钟`;
}

// 处理图片下载完成事件
async function handleImageDownloaded({ path, localPath }) {
  const game = games.value.find(g => g.path === path);
  if (game) {
    game.image = localPath;
    // 重新加载游戏列表以确保数据一致
    await loadGames();
  }
}

// 格式化最后游玩时间
function formatLastPlayed(isoString) {
  if (!isoString) return '';
  
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now - date;
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  
  // 如果是今天
  if (diffDays === 0) {
    return `今天 ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  }
  // 如果是昨天
  if (diffDays === 1) {
    return `昨天 ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  }
  // 如果是一周内
  if (diffDays < 7) {
    return `${diffDays}天前`;
  }
  // 否则显示完整日期
  return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')}`;
}

async function launch(g) {
  try {
    const list = await listExesService(g.path);
    // list may be [] or array
    exeList.value = list || [];
    if (exeList.value.length === 0) {
      alert('未找到可执行文件');
      return;
    }
    if (exeList.value.length === 1) {
      await launchExeService(exeList.value[0]);
      return;
    }
    // multiple: show modal
    modalGame.value = g;
    chosenExe.value = exeList.value[0];
    showExeModal.value = true;
  } catch (e) {
    alert('启动失败: ' + e);
  }
}

async function confirmLaunch() {
  try {
    await launchExeService(chosenExe.value);
    showExeModal.value = false;
  } catch (e) {
    alert('启动失败: ' + e);
  }
}

// Replace-from-Bangumi UI state & functions
const replaceModalVisible = ref(false);
const replaceSearchKeyword = ref('');
const replaceResults = ref([]);
const replaceLoading = ref(false);
const replaceTargetGame = ref(null);

function openReplaceModal(g) {
  replaceTargetGame.value = g;
  replaceSearchKeyword.value = '';
  replaceResults.value = [];
  replaceModalVisible.value = true;
}

function closeReplaceModal() {
  replaceModalVisible.value = false;
  replaceTargetGame.value = null;
}

async function replaceExeFile() {
  if (!replaceTargetGame.value) return;
  
  try {
    const currentPath = replaceTargetGame.value.path;
    const pathParts = currentPath.split(/[\\/]/);
    pathParts.pop();
    const initialDir = pathParts.join('\\');
    
  const newPath = await pickExeService(initialDir);
    if (!newPath) return;
    
    const oldPath = replaceTargetGame.value.path;
    
    // 更新游戏的 exe 路径（保持其他信息不变）
    await updateGameInfo({ path: oldPath, name: null, image: null, imageUrl: null, subjectId: null });
    
    // 从数据库中删除旧路径的游戏
  await removeGameService(oldPath);
    
    // 添加新路径的游戏（复制原有信息，保留 folder_path）
    await addGame({ path: newPath, name: replaceTargetGame.value.name, folderPath: replaceTargetGame.value.folder_path || null });
    
    // 更新新游戏的完整信息
    await updateGameInfo({ path: newPath, name: replaceTargetGame.value.name, image: replaceTargetGame.value.image, imageUrl: replaceTargetGame.value.image_url, subjectId: replaceTargetGame.value.subject_id });
    
    // 在本地游戏列表中找到并更新该游戏
    const gameIndex = games.value.findIndex(g => g.path === oldPath);
    if (gameIndex !== -1) {
      games.value[gameIndex].path = newPath;
    }
    
    // 更新本地游戏对象引用
    replaceTargetGame.value.path = newPath;
    
    alert('EXE 文件已更换');
    closeReplaceModal();
    // Refresh list and try to fetch images/details for the replaced game
    try { await loadGames(); await autoFetchImages(); } catch (e) { /* ignore */ }
  } catch (e) {
    if (e === 'cancelled' || e === 'canceled') return;
    alert('更换失败: ' + e);
  }
}

async function runReplaceSearch(keyword) {
  const searchKeyword = keyword || replaceSearchKeyword.value;
  if (!searchKeyword) return;
  replaceSearchKeyword.value = searchKeyword;
  replaceLoading.value = true;
  try {
    const filter = { type: [4], nsfw: true };
  const res = await gameService.searchBangumi({ query: searchKeyword, filter });
    
    let list = [];
    if (res) {
      if (Array.isArray(res)) list = res;
      else if (res.data && Array.isArray(res.data)) list = res.data;
      else if (res.results && Array.isArray(res.results)) list = res.results;
      else if (res.subjects && Array.isArray(res.subjects)) list = res.subjects;
      else if (res.items && Array.isArray(res.items)) list = res.items;
      else { for (const k in res) if (Array.isArray(res[k])) { list = res[k]; break; } }
    }
    replaceResults.value = list;
  } catch (e) {
    console.error('replace search failed', e);
    replaceResults.value = [];
  } finally {
    replaceLoading.value = false;
  }
}

async function selectReplaceItem(item) {
  if (!replaceTargetGame.value) return;
  const image = item.image || (item.subject && item.subject.image) || (item.images && (item.images.large || item.images.medium || item.images.small || item.images.common || item.images.grid)) || (item.subject && item.subject.images && (item.subject.images.large || subject.images.medium || subject.images.small));
  const title = item.name_cn || item.name || item.title || (item.subject && (item.subject.name_cn || item.subject.name || item.subject.title)) || null;
  // Use the same getId helper as in fetchImageForGame
  const getId = (node) => {
    if (!node) return null;
    if (node.id) return node.id;
    if (node.data) {
      if (Array.isArray(node.data) && node.data.length > 0 && node.data[0].id) return node.data[0].id;
      if (node.data.id) return node.data.id;
    }
    if (node.subject && node.subject.id) return node.subject.id;
    return null;
  };
  const sid = getId(item);
  
  // 保存旧的 subject_id 用于删除旧缓存和图片
  const oldSubjectId = replaceTargetGame.value.subject_id;
  
  try {
    const updRes = await updateGameInfo({ path: replaceTargetGame.value.path, name: title, image, imageUrl: image, subjectId: sid });
    // update local copy from backend result
    if (updRes && updRes.name) replaceTargetGame.value.name = updRes.name;
    if (updRes && updRes.image) replaceTargetGame.value.image = updRes.image;
    if (updRes && updRes.image_url) replaceTargetGame.value.image_url = updRes.image_url;
    if (updRes && updRes.subject_id) replaceTargetGame.value.subject_id = updRes.subject_id;
    
    // 如果更换了条目，删除旧的缓存和图片
    if (oldSubjectId && oldSubjectId !== sid) {
      try {
  removeDetailCache(oldSubjectId);
  await gameService.deleteCachedImage({ subjectId: oldSubjectId });
      } catch (err) {
        console.error('删除旧缓存失败:', err);
      }
    }
    
    // 立即下载新图片并缓存详情信息
    if (sid && image) {
      try {
        // 下载并保存图片到本地
        const localImagePath = await gameService.downloadImage({ url: image, subjectId: sid });
        
        // 更新游戏使用本地图片路径
        if (localImagePath) {
          await updateGameInfo({ path: replaceTargetGame.value.path, name: null, image: localImagePath, imageUrl: null, subjectId: null });
          replaceTargetGame.value.image = localImagePath;
        }
        
        // 获取并缓存完整的详情信息
        const detailData = await gameService.getBangumiSubject(sid);
        if (detailData) setDetailCache(sid, detailData);
      } catch (err) {
        console.error('下载图片或缓存详情失败:', err);
        // 不阻止流程
      }
    }
    
    closeReplaceModal();
    // After selecting a replacement item, ensure UI reflects changes and try to fetch missing images
    try { await loadGames(); await autoFetchImages(); } catch (e) { /* ignore */ }
  } catch (e) {
    console.error('替换失败:', e);
    alert('替换失败: ' + e);
  }
}

// Game detail modal state
const detailModalVisible = ref(false);
const detailData = ref(null);
const detailGame = ref(null);

async function openGameDetail(g) {
  if (!g.subject_id) return alert('该游戏没有关联 Bangumi 条目');
  try {
    // Check permanent cache first
    let res = getDetailCache(g.subject_id);
    if (!res) {
      res = await gameService.getBangumiSubject(g.subject_id);
      if (res) setDetailCache(g.subject_id, res);
    }
    detailData.value = res;
    detailGame.value = g;
    detailModalVisible.value = true;
  } catch (e) {
    alert('获取条目详情失败: ' + e);
  }
}

function closeDetailModal() {
  detailModalVisible.value = false;
  detailData.value = null;
  detailGame.value = null;
}

function getInfoboxValue(key) {
  if (!detailData.value || !detailData.value.infobox) return null;
  const item = detailData.value.infobox.find(i => i.key === key);
  if (!item) return null;
  if (Array.isArray(item.value)) {
    return item.value.map(v => v.v || v).join(', ');
  }
  return item.value;
}

// 标签相关处理
const sidebarRef = ref(null);
const tagManagementRef = ref(null);

function handleTagSelected(tag) {
  // 点击标签时，如果已选中该标签则取消，否则选中
  if (selectedTag.value === tag) {
    selectedTag.value = null;
  } else {
    selectedTag.value = tag;
  }
  // 确保在游戏库页面
  if (activeTab.value !== 'library') {
    activeTab.value = 'library';
  }
}

async function handleTagsUpdated() {
  // 刷新游戏列表以更新标签数据
  await loadGames();
  // 刷新侧边栏标签列表
  if (sidebarRef.value && sidebarRef.value.loadTags) {
    await sidebarRef.value.loadTags();
  }
  // 刷新标签管理页面（如果正在该页面）
  if (tagManagementRef.value && tagManagementRef.value.loadTags) {
    await tagManagementRef.value.loadTags();
  }
}
</script>

<template>
  <div @click="showMenu = false" style="display:flex; min-height:100vh;">
    <!-- 左侧固定侧边栏 -->
    <Sidebar v-model="activeTab" :selected-tag="selectedTag" @tag-selected="handleTagSelected" @tags-updated="handleTagsUpdated" ref="sidebarRef" />

    <!-- 右侧主内容区域 -->
    <main style="flex:1; margin-left:180px; background:#fff; min-height:100vh;">
      <section style="padding:20px; width:100%;">
        <!-- 游戏库页面 -->
      <div v-if="activeTab === 'library'">
        <GameLibrary 
          :games="games"
          :filtered-games="filteredGames"
          :running-games="runningGames"
          :selected-games="selectedGames"
          :project-root="projectRoot"
          :selected-tag="selectedTag"
          v-model:search-keyword="searchKeyword"
          v-model:selected-filter="selectedFilter"
          :is-multi-select-mode="isMultiSelectMode"
          :selected-exe="selectedExe"
          :is-scanning-folder="isScanningFolder"
          :is-loading-games="isLoadingGames"
          :is-fetching-images="imageFetchRunning"
          :loaded-games-count="loadedGamesCount"
          :total-games-count="totalGamesCount"
          @pick-exe="pickExe"
          @scan-folder="scanFolder"
          @toggle-multi-select="toggleMultiSelect"
          @select-all="selectAllGames"
          @deselect-all="deselectAllGames"
          @delete-selected="deleteSelectedGames"
          @add-to-library="addSelectedToLibrary"
          @toggle-selection="toggleGameSelection"
          @open-detail="openGameDetail"
          @launch="launchFromLibrary"
          @close-game="closeGame"
          @open-replace="openReplaceModal"
          @delete-game="removeGame"
          @clear-tag-filter="selectedTag = null"
          @image-downloaded="handleImageDownloaded"
        />
      </div>

      <!-- 设置页面 -->
      <div v-else-if="activeTab === 'settings'">
        <Settings />
      </div>

      <!-- 标签管理页面 -->
        <div v-else-if="activeTab === 'tag-management'">
          <TagManagement @tags-updated="handleTagsUpdated" ref="tagManagementRef" />
        </div>

        <!-- 工具箱页面 -->
        <div v-else-if="activeTab === 'toolbox'">
          <Toolbox />
        </div>

        <!-- 记录页面 -->
        <div v-else-if="activeTab === 'notes'">
          <Notes />
        </div>

      <!-- 关于页面 -->
      <div v-else-if="activeTab === 'about'">
        <About />
      </div>
    </section>

    <!-- 替换搜索模态框 -->
    <ReplaceModal 
      :visible="replaceModalVisible"
      :target-game="replaceTargetGame"
      :results="replaceResults"
      :loading="replaceLoading"
      @close="closeReplaceModal"
      @search="runReplaceSearch"
      @select="selectReplaceItem"
      @replace-exe="replaceExeFile"
    />
   
    <!-- 详情模态框 -->
    <GameDetailModal 
      :visible="detailModalVisible"
      :detail-data="detailData"
      :game="detailGame"
      :is-running="detailGame && runningGames.has(detailGame.path)"
      :image-src="detailData && detailData.images?.large ? getImageSrc(detailData.images.large) : ''"
      @close="closeDetailModal"
      @launch-game="launchFromLibrary(detailGame)"
      @close-game="closeGame(detailGame.path)"
      @delete-game="async () => { if (await removeGame(detailGame)) closeDetailModal() }"
      @tags-updated="handleTagsUpdated"
    />

    <!-- 扫描结果模态框 -->
    <div v-if="scannedGames.length > 0" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.3); z-index:90; padding:20px;">
      <div style="width:90%; max-width:800px; max-height:90%; overflow:auto; background:#fff; border-radius:4px; border:1px solid #ddd; display:flex; flex-direction:column;">
        <div style="padding:20px; border-bottom:1px solid #e0e0e0; display:flex; justify-content:space-between; align-items:center;">
          <h3 style="margin:0; font-size:18px; font-weight:500; color:#333;">扫描到 {{ scannedGames.length }} 个游戏</h3>
          <button @click="scannedGames = []" 
                  style="padding:6px 12px; font-size:13px; background:#fff; border:1px solid #ddd; color:#666; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#e8e8e8'; $event.target.style.borderColor='#999'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">关闭</button>
        </div>
        
        <div style="padding:20px; overflow-y:auto; max-height:60vh;">
          <div style="margin-bottom:16px; font-size:13px; color:#666;">从文件夹扫描到以下游戏（每个子文件夹识别为一个游戏）：</div>
          <div style="display:flex; flex-direction:column; gap:8px;">
            <label v-for="(game, index) in scannedGames" :key="index" 
                   style="display:flex; align-items:center; padding:10px; border:1px solid #e0e0e0; border-radius:4px; cursor:pointer; transition:all 0.2s;"
                   @mouseenter="$event.currentTarget.style.backgroundColor='#f8f9fa'"
                   @mouseleave="$event.currentTarget.style.backgroundColor='#fff'">
              <input type="checkbox" 
                     :checked="selectedScannedGames.has(index)"
                     @change="toggleScannedGame(index)"
                     style="margin-right:10px; cursor:pointer;" />
              <div style="flex:1; overflow:hidden;">
                <div style="font-size:13px; font-weight:500; color:#333; margin-bottom:2px;">{{ game.name }}</div>
                <div style="font-size:11px; color:#999; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" :title="game.path">
                  {{ game.path }}
                </div>
              </div>
            </label>
          </div>
        </div>
        
        <div style="padding:16px 20px; border-top:1px solid #e0e0e0; display:flex; gap:8px; justify-content:flex-end; background:#f8f9fa;">
          <button @click="addSelectedScannedGames" 
                  style="padding:8px 20px; font-size:13px; background:#fff; border:1px solid #999; color:#333;">
            添加选中的游戏
          </button>
          <button @click="addScannedGames(scannedGames)" 
                  style="padding:8px 20px; font-size:13px; background:#fff; border:1px solid #ddd; color:#666;">
            添加全部
          </button>
        </div>
      </div>
    </div>
  </main>
  
  <!-- Toast 提示 -->
  <div v-if="toastVisible" 
       style="position:fixed; bottom:40px; left:50%; transform:translateX(-50%); background:rgba(0,0,0,0.8); color:#fff; padding:12px 24px; border-radius:4px; font-size:14px; z-index:9999; animation:fadeIn 0.3s;">
    {{ toastMessage }}
  </div>
  </div>
</template>

<style scoped>
/* Toast 动画 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}
</style>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
  font-size: 16px;
  line-height: 1.5;
  font-weight: 400;

  color: #333;
  background-color: #fff;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  min-height: 100vh;
  background-color: #fff;
}

a {
  font-weight: 500;
  color: #666;
  text-decoration: none;
}

a:hover {
  color: #333;
  text-decoration: underline;
}

input,
button {
  border-radius: 4px;
  border: 1px solid #ddd;
  padding: 0.6em 1em;
  font-size: 0.9em;
  font-weight: 400;
  font-family: inherit;
  color: #333;
  background-color: #fff;
  transition: all 0.2s;
}

button {
  cursor: pointer;
  border: 1px solid #ccc;
  background-color: #fff;
  color: #333;
}

button:hover {
  background-color: #e8e8e8;
  border-color: #999;
}

button:active {
  transform: translateY(1px);
  background-color: #ddd;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

input {
  border: 1px solid #ddd;
  background-color: #fff;
  color: #333;
}

input:focus {
  outline: none;
  border-color: #999;
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.05);
}

pre {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', monospace;
}
</style>
