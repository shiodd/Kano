<script setup>
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import BangumiTest from './components/BangumiTest.vue';

const greetMsg = ref("");
const name = ref("");

// Project root directory path
const projectRoot = ref("");

// Cache for Bangumi API responses - permanent cache for game details
const detailCache = ref(new Map());

// Helper function to get image source (convert local paths to Tauri asset URLs)
function getImageSrc(imagePath) {
  if (!imagePath) return null;
  // 如果是 http/https URL，直接返回
  if (imagePath.startsWith('http://') || imagePath.startsWith('https://')) {
    return imagePath;
  }
  // 如果是相对路径（game_data/images/...），转换为绝对路径
  if (imagePath.startsWith('game_data/')) {
    const absolutePath = `${projectRoot.value}\\${imagePath.replace(/\//g, '\\\\')}`;
    return convertFileSrc(absolutePath);
  }
  // 其他情况（绝对路径）直接转换
  return convertFileSrc(imagePath);
}

function getDetailCacheKey(subjectId) {
  return `subject:${subjectId}`;
}

function setDetailCache(subjectId, data) {
  const key = getDetailCacheKey(subjectId);
  
  // 只保存面板上显示的字段，减少缓存文件大小
  // 同时将图片 URL 替换为本地相对路径
  const filteredData = {
    id: data.id,
    name: data.name,
    name_cn: data.name_cn,
    date: data.date,
    summary: data.summary,
    meta_tags: data.meta_tags,
    images: data.images ? { 
      large: `game_data/images/${data.id}.jpg` // 使用本地相对路径
    } : null,
    infobox: data.infobox ? data.infobox.filter(item => 
      ['中文名', '别名', '平台', '游戏类型', '开发', '发行'].includes(item.key)
    ) : null
  };
  
  detailCache.value.set(key, filteredData);
  saveCacheToFile(); // Save to file when cache changes
}

function getDetailCache(subjectId) {
  const key = getDetailCacheKey(subjectId);
  return detailCache.value.get(key) || null;
}

function removeDetailCache(subjectId) {
  if (!subjectId) return;
  const key = getDetailCacheKey(subjectId);
  detailCache.value.delete(key);
  saveCacheToFile(); // Save to file when cache changes
}

function clearAllCache() {
  detailCache.value.clear();
  saveCacheToFile(); // Save to file when cache changes
}

// Load cache from file
async function loadCacheFromFile() {
  try {
    const cacheObj = await invoke('load_cache');
    if (cacheObj && typeof cacheObj === 'object') {
      detailCache.value = new Map(Object.entries(cacheObj));
    }
  } catch (e) {
    console.error('Failed to load cache:', e);
  }
}

// Save cache to file (debounced to avoid too frequent writes)
let saveTimer = null;
async function saveCacheToFile() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    try {
      const cacheObj = Object.fromEntries(detailCache.value);
      await invoke('save_cache', { cache: cacheObj });
    } catch (e) {
      console.error('Failed to save cache:', e);
    }
  }, 500); // Debounce 500ms
}

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}

// Minimal: user selects an exe and we launch it
const selectedExe = ref("");
const games = ref([]);
const runningGames = ref(new Set()); // 追踪正在运行的游戏路径
const gameStartTimes = ref(new Map()); // 记录游戏启动时间 path -> timestamp

// Multi-select state
const selectedGames = ref(new Set());
const isMultiSelectMode = ref(false);

// Menu state
const showMenu = ref(false);

// Toast state
const toastMessage = ref('');
const toastVisible = ref(false);

// Filter state
const selectedFilter = ref('全部'); // '全部', 'ADV', 'RPG'
const searchKeyword = ref(''); // 搜索关键词

// Computed filtered games
const filteredGames = computed(() => {
  let result = games.value;
  
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

// Toast notification
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
            await invoke('delete_cached_image', { subjectId: game.subject_id });
          } catch (err) {
            console.error('删除缓存图片失败:', err);
            // 不阻止删除流程
          }
        }
        await invoke('remove_game', { path });
      }
    }
    selectedGames.value.clear();
    await listGames();
    isMultiSelectMode.value = false; // 自动退出多选模式
  } catch (e) {
    alert('批量删除失败: ' + e);
  }
}

async function listGames() {
  try {
    const res = await invoke('list_games');
    games.value = res || [];
    // after loading games, automatically try to fetch images for games without images
    autoFetchImages();
  } catch (e) {
    console.error('list_games failed', e);
    games.value = [];
  }
}

onMounted(async () => {
  try {
    projectRoot.value = await invoke('get_project_root');
  } catch (e) {
    console.error('Failed to get project root:', e);
  }
  loadCacheFromFile(); // Load cache first
  listGames();
  loadToken();
  
  // Listen for game exit events
  listen('game-exited', async (event) => {
    const gamePath = event.payload;
    await updatePlaytime(gamePath); // 更新游戏时长
    runningGames.value.delete(gamePath);
    console.log('Game exited:', gamePath);
  });
});

const accessToken = ref('');
const showToken = ref(false); // 控制 token 显示/隐藏

async function loadToken() {
  try {
    const t = await invoke('get_access_token');
    accessToken.value = t || '';
  } catch (e) {
    console.error('get_access_token failed', e);
  }
}

async function saveToken() {
  try {
    await invoke('set_access_token', { token: accessToken.value });
    alert('已保存 access token');
  } catch (e) {
    alert('保存失败: ' + e);
  }
}

// (Bangumi search on main page removed — use the BangumiTest component instead)

// UI: which pane is shown in main area. 'library', 'test', or 'settings'
const activeTab = ref('library');
const bangumiLoading = ref(false);
const imageFetchRunning = ref(false);

function formatSeconds(s) {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  return `${h}h ${m}m ${sec}s`;
}

async function scan() {
  // removed
}

async function startGame(g) {
  // removed
}

async function stopGame(g) {
  // removed
}

async function pickExe() {
  try {
    const path = await invoke('pick_exe', { initialDir: null });
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
    const games = await invoke('pick_folder_and_scan');
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
      await invoke('add_game', { path: game.path, name: game.name });
    }
    scannedGames.value = [];
    selectedScannedGames.value = new Set();
    await listGames();
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
    await invoke('launch_exe', { path: selectedExe.value });
  } catch (e) {
    alert('启动失败: ' + e);
  }
}

async function addSelectedToLibrary() {
  if (!selectedExe.value) return showToast('请先选择一个 EXE');
  try {
    await invoke('add_game', { path: selectedExe.value, name: null });
    await listGames();
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
        await invoke('delete_cached_image', { subjectId: g.subject_id });
      } catch (err) {
        console.error('删除缓存图片失败:', err);
        // 不阻止删除游戏流程
      }
    }
    await invoke('remove_game', { path: g.path });
    await listGames();
    return true; // 删除成功，返回 true
  } catch (e) {
    alert('删除失败: ' + e);
    return false; // 删除失败，返回 false
  }
}

// Try to fetch image for a single game entry.
// Returns true if image was found and saved.
async function fetchImageForGame(g) {
  try {
    // prefer containing folder name first, then fallback to exe basename
    const p = g.path || '';
    const parts = p.split(/[\\/]/);
    let folder = '';
    if (parts.length >= 2) folder = parts[parts.length - 2];
    const exeName = parts.pop() || p;
    let res = null;
    const filter = { type: [4], nsfw: true };
    
    // Search without cache (temporary searches don't need caching)
    if (folder) {
      res = await invoke('search_bangumi', { query: folder, filter });
    }
    if (!res) {
      res = await invoke('search_bangumi', { query: exeName, filter });
    }
    
    let list = [];
    if (res) {
      if (Array.isArray(res)) list = res;
      else if (res.data && Array.isArray(res.data)) list = res.data;
      else if (res.results && Array.isArray(res.results)) list = res.results;
      else if (res.subjects && Array.isArray(res.subjects)) list = res.subjects;
      else if (res.items && Array.isArray(res.items)) list = res.items;
      else {
        for (const k in res) if (Array.isArray(res[k])) { list = res[k]; break; }
      }
    }
    // fallback: try containing folder name
    if ((!list || list.length === 0) && p) {
      const parts = p.split(/[\\/]/);
      if (parts.length >= 2) {
        const folder = parts[parts.length - 2];
        res = await invoke('search_bangumi', { query: folder, filter });
        if (res) {
          if (Array.isArray(res)) list = res;
          else if (res.data && Array.isArray(res.data)) list = res.data;
          else if (res.results && Array.isArray(res.results)) list = res.results;
          else if (res.subjects && Array.isArray(res.subjects)) list = res.subjects;
          else if (res.items && Array.isArray(res.items)) list = res.items;
          else { for (const k in res) if (Array.isArray(res[k])) { list = res[k]; break; } }
        }
      }
    }

  // If we have results, try to get image and save it to game
    if (list && list.length > 0) {
      // if folder or exe name contains 'FD' (case-insensitive), prefer second result
      const fdFlag = ((folder && /fd/i.test(folder)) || (/fd/i.test(exeName)));
      const idx = (fdFlag && list.length > 1) ? 1 : 0;
      const first = list[idx];
      
      const image = first.image || (first.subject && first.subject.image) || (first.images && (first.images.large || first.images.medium || first.images.small || first.images.common || first.images.grid)) || (first.subject && first.subject.images && (first.subject.images.large || first.subject.images.medium || first.subject.images.small));
      const title = first.name_cn || first.name || first.title || (first.subject && (first.subject.name_cn || first.subject.name || first.subject.title)) || null;
      
      if (image || title || first.id) {
        try {
          // robust id extraction: prefers id directly, then data array, then subject
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
          
          const sid = getId(first);
          const numericSid = sid ? Number(sid) : null;
          if (sid && isNaN(numericSid)) {
            return false;
          }
          
          const updRes = await invoke('update_game_info', {
            path: g.path,
            name: title,
            image: image,
            subjectId: numericSid
          });
          
          // update local copy from backend result
          if (updRes && updRes.name) g.name = updRes.name;
          if (updRes && updRes.image) g.image = updRes.image;
          if (updRes && updRes.subject_id) g.subject_id = updRes.subject_id;
          
          // 立即下载图片并缓存详情信息
          if (numericSid && image) {
            try {
              // 下载并保存图片到本地，文件名为 subject_id
              const localImagePath = await invoke('download_image', { 
                url: image, 
                subjectId: numericSid 
              });
              
              // 更新游戏使用本地图片路径
              if (localImagePath) {
                await invoke('update_game_info', {
                  path: g.path,
                  name: null,
                  image: localImagePath,
                  subjectId: null
                });
                g.image = localImagePath;
              }
              
              // 获取并缓存完整的详情信息
              const detailData = await invoke('get_bangumi_subject', { id: numericSid });
              if (detailData) {
                setDetailCache(numericSid, detailData);
              }
            } catch (err) {
              console.error('下载图片或缓存详情失败:', err);
              // 不阻止流程，继续使用在线图片
            }
          }
          
          return true;
        } catch (e) {
          console.error('update_game_info failed', e);
        }
      }
    }
    return false;
  } catch (e) {
    console.error('fetchImageForGame failed', e);
    return false;
  }
}

// Iterate through games and fetch images for those without image.
async function autoFetchImages() {
  if (imageFetchRunning.value) return;
  imageFetchRunning.value = true;
  try {
    for (const g of games.value) {
      if (!g.image) {
        await fetchImageForGame(g);
        // small delay to avoid hammering the API
        await new Promise((r) => setTimeout(r, 200));
      }
    }
  } finally {
    imageFetchRunning.value = false;
  }
}

async function launchFromLibrary(g) {
  // 检查游戏是否已在运行中
  if (runningGames.value.has(g.path)) {
    alert('该游戏已在运行中');
    return;
  }
  
  try {
    runningGames.value.add(g.path); // 标记为运行中
    gameStartTimes.value.set(g.path, Date.now()); // 记录启动时间
    await invoke('launch_exe', { path: g.path });
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
    await invoke('kill_game', { path: gamePath });
    // 进程终止后，game-exited 事件会自动触发，移除运行状态
    // 但为了即时反馈，我们也可以立即移除
    await updatePlaytime(gamePath);
    runningGames.value.delete(gamePath);
  } catch (e) {
    console.error('关闭游戏失败:', e);
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
    const totalPlaytime = await invoke('update_game_playtime', { 
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
    
    console.log(`游戏时长已更新: ${formatPlaytime(totalPlaytime)}`);
  } catch (e) {
    console.error('更新游戏时长失败:', e);
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
    const list = await invoke('list_exes', { game_dir: g.path });
    // list may be [] or array
    exeList.value = list || [];
    if (exeList.value.length === 0) {
      alert('未找到可执行文件');
      return;
    }
    if (exeList.value.length === 1) {
      await invoke('launch_exe', { path: exeList.value[0] });
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
    await invoke('launch_exe', { path: chosenExe.value });
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
    // 获取当前 exe 文件的目录
    const currentPath = replaceTargetGame.value.path;
    const pathParts = currentPath.split(/[\\/]/);
    pathParts.pop(); // 移除文件名
    const initialDir = pathParts.join('\\'); // 重新组合为目录路径
    
    const newPath = await invoke('pick_exe', { initialDir });
    if (!newPath) return; // 用户取消选择
    
    const oldPath = replaceTargetGame.value.path;
    
    // 更新游戏的 exe 路径（保持其他信息不变）
    await invoke('update_game_info', {
      path: oldPath,
      name: null,
      image: null,
      subjectId: null
    });
    
    // 从数据库中删除旧路径的游戏
    await invoke('remove_game', { path: oldPath });
    
    // 添加新路径的游戏（复制原有信息）
    await invoke('add_game', { path: newPath, name: replaceTargetGame.value.name });
    
    // 更新新游戏的完整信息
    await invoke('update_game_info', {
      path: newPath,
      name: replaceTargetGame.value.name,
      image: replaceTargetGame.value.image,
      subjectId: replaceTargetGame.value.subject_id
    });
    
    // 在本地游戏列表中找到并更新该游戏
    const gameIndex = games.value.findIndex(g => g.path === oldPath);
    if (gameIndex !== -1) {
      games.value[gameIndex].path = newPath;
    }
    
    // 更新本地游戏对象引用
    replaceTargetGame.value.path = newPath;
    
    alert('EXE 文件已更换');
    closeReplaceModal();
  } catch (e) {
    // 用户取消时不提示错误
    if (e === 'cancelled' || e === 'canceled') return;
    console.error('更换 EXE 文件失败:', e);
    alert('更换失败: ' + e);
  }
}

async function runReplaceSearch() {
  if (!replaceSearchKeyword.value) return;
  replaceLoading.value = true;
  try {
    const filter = { type: [4], nsfw: true };
    const res = await invoke('search_bangumi', { query: replaceSearchKeyword.value, filter });
    
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
    const updRes = await invoke('update_game_info', { path: replaceTargetGame.value.path, name: title, image, subjectId: sid });
    // update local copy from backend result
    if (updRes && updRes.name) replaceTargetGame.value.name = updRes.name;
    if (updRes && updRes.image) replaceTargetGame.value.image = updRes.image;
    if (updRes && updRes.subject_id) replaceTargetGame.value.subject_id = updRes.subject_id;
    
    // 如果更换了条目，删除旧的缓存和图片
    if (oldSubjectId && oldSubjectId !== sid) {
      try {
        removeDetailCache(oldSubjectId);
        await invoke('delete_cached_image', { subjectId: oldSubjectId });
      } catch (err) {
        console.error('删除旧缓存失败:', err);
      }
    }
    
    // 立即下载新图片并缓存详情信息
    if (sid && image) {
      try {
        // 下载并保存图片到本地
        const localImagePath = await invoke('download_image', { 
          url: image, 
          subjectId: sid 
        });
        
        // 更新游戏使用本地图片路径
        if (localImagePath) {
          await invoke('update_game_info', {
            path: replaceTargetGame.value.path,
            name: null,
            image: localImagePath,
            subjectId: null
          });
          replaceTargetGame.value.image = localImagePath;
        }
        
        // 获取并缓存完整的详情信息
        const detailData = await invoke('get_bangumi_subject', { id: sid });
        if (detailData) {
          setDetailCache(sid, detailData);
        }
      } catch (err) {
        console.error('下载图片或缓存详情失败:', err);
        // 不阻止流程
      }
    }
    
    closeReplaceModal();
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
      res = await invoke('get_bangumi_subject', { id: g.subject_id });
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
</script>

<template>
  <div @click="showMenu = false" style="display:flex; min-height:100vh;">
    <!-- 左侧固定侧边栏 -->
    <aside style="width:180px; background:#f8f9fa; border-right:1px solid #e0e0e0; padding:0; position:fixed; left:0; top:0; bottom:0; display:flex; flex-direction:column;">
      <div style="padding:20px 16px; border-bottom:1px solid #e0e0e0;">
        <h2 style="margin:0; font-size:16px; font-weight:600; color:#333;">游戏管理器</h2>
      </div>
      <nav style="flex:1; padding:12px 0; display:flex; flex-direction:column; gap:2px;">
        <button :class="{active: activeTab === 'library'}" @click="activeTab = 'library'" 
                style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent;">
          游戏库
        </button>
        <button :class="{active: activeTab === 'test'}" @click="activeTab = 'test'" 
                style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent;">
          Bangumi 测试
        </button>
        <button :class="{active: activeTab === 'settings'}" @click="activeTab = 'settings'" 
                style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent;">
          设置
        </button>
      </nav>
    </aside>

    <!-- 右侧主内容区域 -->
    <main style="flex:1; margin-left:180px; background:#fff; min-height:100vh;">
      <section style="padding:20px; width:100%;">
        <!-- 游戏库页面 -->
      <div v-if="activeTab === 'library'">
        <!-- 标题栏 -->
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">
          <div style="display:flex; gap:12px; align-items:center; flex:1;">
            <h1 style="margin:0; font-size:20px; font-weight:500; color:#333;">游戏库</h1>
            
            <!-- 搜索框 -->
            <div style="flex:1; max-width:300px;">
              <input v-model="searchKeyword" 
                     placeholder="搜索游戏..." 
                     style="width:100%; padding:6px 12px; font-size:13px; border:1px solid #e0e0e0; border-radius:4px; outline:none;"
                     @focus="$event.target.style.borderColor='#999'"
                     @blur="$event.target.style.borderColor='#e0e0e0'" />
            </div>
            
            <!-- 筛选按钮 -->
            <div style="display:flex; gap:4px; align-items:center; padding:4px; background:#fff; border-radius:4px; border:1px solid #e0e0e0;">
              <button @click="setFilter('全部')" 
                      :style="{
                        padding: '4px 12px',
                        fontSize: '12px',
                        background: selectedFilter === '全部' ? '#e8e8e8' : '#fff',
                        border: selectedFilter === '全部' ? '1px solid #999' : '1px solid #ddd',
                        color: selectedFilter === '全部' ? '#333' : '#666',
                        fontWeight: selectedFilter === '全部' ? '500' : '400',
                        borderRadius: '3px',
                        cursor: 'pointer'
                      }">
                全部
              </button>
              <button @click="setFilter('ADV')" 
                      :style="{
                        padding: '4px 12px',
                        fontSize: '12px',
                        background: selectedFilter === 'ADV' ? '#e8e8e8' : '#fff',
                        border: selectedFilter === 'ADV' ? '1px solid #999' : '1px solid #ddd',
                        color: selectedFilter === 'ADV' ? '#333' : '#666',
                        fontWeight: selectedFilter === 'ADV' ? '500' : '400',
                        borderRadius: '3px',
                        cursor: 'pointer'
                      }">
                ADV
              </button>
              <button @click="setFilter('RPG')" 
                      :style="{
                        padding: '4px 12px',
                        fontSize: '12px',
                        background: selectedFilter === 'RPG' ? '#e8e8e8' : '#fff',
                        border: selectedFilter === 'RPG' ? '1px solid #999' : '1px solid #ddd',
                        color: selectedFilter === 'RPG' ? '#333' : '#666',
                        fontWeight: selectedFilter === 'RPG' ? '500' : '400',
                        borderRadius: '3px',
                        cursor: 'pointer'
                      }">
                RPG
              </button>
            </div>
            <div style="font-size:12px; color:#999;">
              {{ filteredGames.length }} / {{ games.length }}
            </div>
          </div>
          
          <!-- 右上角菜单 -->
          <div style="position:relative;">
            <button @click.stop="showMenu = !showMenu" 
                    style="padding:8px 16px; font-size:13px; white-space:nowrap; display:flex; align-items:center; gap:4px;"
                    @mouseenter="$event.target.style.backgroundColor='#f0f0f0'"
                    @mouseleave="$event.target.style.backgroundColor='#fff'">
              操作 ▼
            </button>
            
            <!-- 下拉菜单 -->
            <div v-if="showMenu" 
                 @click.stop
                 style="position:absolute; top:100%; right:0; margin-top:4px; background:#fff; border:1px solid #e0e0e0; border-radius:4px; box-shadow:0 2px 8px rgba(0,0,0,0.1); z-index:100; min-width:160px;">
              <button @click="pickExe(); showMenu = false" 
                      style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                      @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                      @mouseleave="$event.target.style.backgroundColor='#fff'">
                选择游戏
              </button>
              <button @click="scanFolder(); showMenu = false" 
                      :disabled="isScanningFolder"
                      style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                      @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                      @mouseleave="$event.target.style.backgroundColor='#fff'">
                {{ isScanningFolder ? '扫描中...' : '扫描文件夹' }}
              </button>
              <div style="height:1px; background:#e0e0e0; margin:4px 0;"></div>
              <button @click="toggleMultiSelect(); showMenu = false" 
                      style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                      @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                      @mouseleave="$event.target.style.backgroundColor='#fff'">
                {{ isMultiSelectMode ? '退出多选模式' : '多选模式' }}
              </button>
              <template v-if="isMultiSelectMode">
                <button @click="selectAllGames(); showMenu = false" 
                        style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                        @mouseleave="$event.target.style.backgroundColor='#fff'">
                  全选
                </button>
                <button @click="deselectAllGames(); showMenu = false" 
                        style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                        @mouseleave="$event.target.style.backgroundColor='#fff'">
                  取消全选
                </button>
                <button @click="deleteSelectedGames(); showMenu = false" 
                        :disabled="selectedGames.size === 0"
                        style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#f44336; cursor:pointer; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                        @mouseleave="$event.target.style.backgroundColor='#fff'">
                  删除选中 {{ selectedGames.size > 0 ? `(${selectedGames.size})` : '' }}
                </button>
              </template>
            </div>
          </div>
        </div>
        
        <!-- 选中的游戏显示 -->
        <div v-if="selectedExe" style="margin-bottom:16px; padding:12px; background:#f8f9fa; border-radius:4px; border:1px solid #e0e0e0; display:flex; gap:8px; align-items:center;">
          <div style="flex:1; font-size:12px; color:#666; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" :title="selectedExe">
            已选择: {{ selectedExe }}
          </div>
          <button @click="addSelectedToLibrary" style="padding:6px 12px; font-size:12px; white-space:nowrap;">加入游戏库</button>
        </div>

        <div v-if="games.length === 0" style="text-align:center; padding:60px 20px; color:#ccc;">
          <div style="font-size:14px;">暂无游戏</div>
        </div>
        
        <div v-else-if="filteredGames.length === 0" style="text-align:center; padding:60px 20px; color:#ccc;">
          <div style="font-size:14px;">当前筛选条件下没有游戏</div>
        </div>
        
        <div v-else style="display:grid; grid-template-columns:repeat(auto-fill, 140px); gap:12px;">
          <div v-for="g in filteredGames" :key="g.path" 
               :style="{
                 width: '140px',
                 border: selectedGames.has(g.path) ? '2px solid #666' : '1px solid #e0e0e0',
                 borderRadius: '4px',
                 overflow: 'hidden',
                 transition: 'all 0.2s',
                 background: selectedGames.has(g.path) ? '#f0f0f0' : '#fafafa',
                 userSelect: 'none',
                 cursor: 'pointer'
               }"
               @click.stop="isMultiSelectMode ? toggleGameSelection(g) : openGameDetail(g)"
               @mouseenter="if (!selectedGames.has(g.path)) { $event.currentTarget.style.backgroundColor='#eeeeee'; $event.currentTarget.style.borderColor='#bbb' }"
               @mouseleave="if (!selectedGames.has(g.path)) { $event.currentTarget.style.backgroundColor='#fafafa'; $event.currentTarget.style.borderColor='#e0e0e0' }">
            <div style="width:140px; height:196px; position:relative; background:#f0f0f0; overflow:hidden;">
              <img v-if="g.image" :src="getImageSrc(g.image)" style="position:absolute; top:0; left:0; width:100%; height:100%; object-fit:cover;" />
              <!-- 运行中状态指示器 -->
              <div v-if="runningGames.has(g.path)" 
                   :style="{
                     position: 'absolute',
                     top: '8px',
                     left: '8px',
                     padding: '4px 8px',
                     borderRadius: '3px',
                     background: 'rgba(76, 175, 80, 0.9)',
                     color: '#fff',
                     fontSize: '11px',
                     fontWeight: '500',
                     boxShadow: '0 2px 4px rgba(0,0,0,0.2)'
                   }">
                运行中
              </div>
              <!-- 多选模式下显示选中标记 -->
              <div v-if="isMultiSelectMode" 
                   :style="{
                     position: 'absolute',
                     top: '8px',
                     right: '8px',
                     width: '20px',
                     height: '20px',
                     borderRadius: '3px',
                     background: selectedGames.has(g.path) ? '#666' : '#fff',
                     border: '2px solid #666',
                     display: 'flex',
                     alignItems: 'center',
                     justifyContent: 'center',
                     fontSize: '12px',
                     transition: 'all 0.2s'
                   }">
                <span v-if="selectedGames.has(g.path)" style="color:#fff; font-weight:bold;">✓</span>
              </div>
            </div>
            <div style="padding:8px;">
              <div style="font-size:12px; font-weight:500; margin-bottom:2px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:#333;" :title="g.name">
                {{ g.name }}
              </div>
              <div style="font-size:10px; color:#999; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-bottom:6px;" :title="g.path">
                {{ g.path.split(/[\\/]/).pop() }}
              </div>
              <div v-if="!isMultiSelectMode" style="display:flex; gap:4px;" @click.stop>
                <!-- 运行中时显示关闭按钮（红色），否则显示启动按钮（绿色） -->
                <button v-if="runningGames.has(g.path)" 
                        @click="closeGame(g.path)" 
                        style="flex:1; padding:5px; font-size:11px; background:#f44336; border:1px solid #f44336; color:#fff; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#d32f2f'; $event.target.style.borderColor='#d32f2f'"
                        @mouseleave="$event.target.style.backgroundColor='#f44336'; $event.target.style.borderColor='#f44336'">关闭</button>
                <button v-else 
                        @click="launchFromLibrary(g)" 
                        style="flex:1; padding:5px; font-size:11px; background:#4CAF50; border:1px solid #4CAF50; color:#fff; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#45a049'; $event.target.style.borderColor='#45a049'"
                        @mouseleave="$event.target.style.backgroundColor='#4CAF50'; $event.target.style.borderColor='#4CAF50'">启动</button>
                <button @click.stop="openReplaceModal(g)" 
                        style="padding:5px 8px; font-size:11px; background:#fff; border:1px solid #ddd; color:#666; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#f0f0f0'; $event.target.style.borderColor='#999'"
                        @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">替换</button>
                <button @click="removeGame(g)" 
                        style="padding:5px 8px; font-size:11px; background:#fff; border:1px solid #ddd; color:#999; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#f0f0f0'; $event.target.style.borderColor='#999'"
                        @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">删除</button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Bangumi 测试页面 -->
      <div v-else-if="activeTab === 'test'">
        <h1 style="margin:0 0 20px 0; font-size:20px; font-weight:500; color:#333;">Bangumi 测试</h1>
        <BangumiTest />
      </div>

      <!-- 设置页面 -->
      <div v-else-if="activeTab === 'settings'">
        <h1 style="margin:0 0 20px 0; font-size:20px; font-weight:500; color:#333;">设置</h1>
        
        <div style="margin-bottom:20px; padding:16px; background:#f8f9fa; border-radius:4px; border:1px solid #e0e0e0;">
          <label style="display:block; font-weight:500; margin-bottom:8px; font-size:13px; color:#333;">Bangumi Access Token</label>
          <div style="font-size:12px; color:#999; margin-bottom:10px;">
            用于访问 NSFW 内容。在 <a href="https://bgm.tv/dev/app" target="_blank" style="color:#666; text-decoration:underline;">Bangumi 开发者页面</a> 获取。
          </div>
          <div style="display:flex; gap:8px;">
            <div style="position:relative; flex:1;">
              <input v-model="accessToken" 
                     :type="showToken ? 'text' : 'password'"
                     placeholder="输入 Access Token" 
                     style="width:100%; padding:8px 40px 8px 12px; font-size:13px; box-sizing:border-box;" />
              <button @click="showToken = !showToken" 
                      style="position:absolute; right:4px; top:50%; transform:translateY(-50%); padding:4px 8px; font-size:12px; background:transparent; border:1px solid #ddd; color:#666; cursor:pointer; transition:all 0.2s;"
                      @mouseenter="$event.target.style.backgroundColor='#f0f0f0'; $event.target.style.borderColor='#999'"
                      @mouseleave="$event.target.style.backgroundColor='transparent'; $event.target.style.borderColor='#ddd'">
                {{ showToken ? '隐藏' : '显示' }}
              </button>
            </div>
            <button @click="saveToken" style="padding:8px 16px; font-size:13px;">保存</button>
          </div>
        </div>
      </div>
    </section>

    <!-- 替换搜索模态框 -->
    <div v-if="replaceModalVisible" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.3); z-index:80;">
      <div style="width:70%; max-width:800px; max-height:80%; overflow:auto; background:#fff; padding:20px; border-radius:4px; border:1px solid #ddd;">
        <h3 style="margin:0 0 16px 0; font-size:16px; font-weight:500; color:#333;">搜索并替换: {{ replaceTargetGame?.name }}</h3>
        
        <!-- 更换 EXE 文件区域 -->
        <div style="margin-bottom:16px; padding:12px; background:#f8f9fa; border-radius:4px; border:1px solid #e0e0e0;">
          <div style="display:flex; gap:8px; align-items:center; margin-bottom:8px;">
            <button @click="replaceExeFile" style="padding:6px 12px; font-size:12px; white-space:nowrap;">更换 EXE 文件</button>
            <div style="flex:1; font-size:11px; color:#999;">更换游戏的可执行文件路径</div>
          </div>
          <div v-if="replaceTargetGame" style="font-size:11px; color:#666; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; padding:4px 8px; background:#fff; border-radius:3px; border:1px solid #e0e0e0;" :title="replaceTargetGame.path">
            当前: {{ replaceTargetGame.path }}
          </div>
        </div>
        
        <!-- 分割线 -->
        <div style="height:1px; background:#e0e0e0; margin:16px 0;"></div>
        
        <!-- 搜索替换区域 -->
        <div style="display:flex; gap:8px; align-items:center; margin-bottom:16px;">
          <input v-model="replaceSearchKeyword" placeholder="输入关键词搜索" style="flex:1; padding:8px 12px; font-size:13px;" @keyup.enter="runReplaceSearch" />
          <button @click="runReplaceSearch" style="padding:8px 16px; font-size:13px;">搜索</button>
          <button @click="closeReplaceModal" style="padding:8px 16px; font-size:13px; background:#999;">关闭</button>
        </div>
        
        <div v-if="replaceLoading" style="text-align:center; padding:30px; color:#999; font-size:13px;">搜索中...</div>
        <div v-else-if="replaceResults.length === 0 && replaceSearchKeyword" style="text-align:center; padding:30px; color:#ccc; font-size:13px;">
          没有找到相关结果
        </div>
        <div v-else style="display:grid; grid-template-columns: repeat(auto-fill, 140px); gap:12px;">
          <div v-for="(it, i) in replaceResults" :key="it.id || i" 
               style="width:140px; border:1px solid #e0e0e0; border-radius:4px; overflow:hidden; cursor:pointer; transition:all 0.2s; background:#fafafa;"
               @click="selectReplaceItem(it)"
               @mouseenter="$event.currentTarget.style.backgroundColor='#f5f5f5'; $event.currentTarget.style.borderColor='#ccc'"
               @mouseleave="$event.currentTarget.style.backgroundColor='#fafafa'; $event.currentTarget.style.borderColor='#e0e0e0'">
            <div style="width:140px; height:196px; position:relative; background:#f0f0f0; overflow:hidden;">
              <img v-if="it.image || it.subject?.image || it.images?.large" 
                   :src="it.image || it.subject?.image || it.images?.large || it.images?.medium || it.images?.small" 
                   style="position:absolute; top:0; left:0; width:100%; height:100%; object-fit:cover;" />
              <div v-else style="position:absolute; top:0; left:0; width:100%; height:100%; display:flex; align-items:center; justify-content:center; color:#ccc; font-size:32px;">?</div>
            </div>
            <div style="padding:10px;">
              <div style="font-weight:500; font-size:12px; margin-bottom:4px; overflow:hidden; text-overflow:ellipsis; display:-webkit-box; line-clamp:2; -webkit-line-clamp:2; -webkit-box-orient:vertical; color:#333;">
                {{ it.name_cn || it.name || it.title || it.subject?.name_cn || it.subject?.name || it.subject?.title }}
              </div>
              <div style="font-size:11px; color:#999;">ID: {{ it.id || it.subject?.id || 'N/A' }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
   
    <!-- 详情模态框 -->
    <div v-if="detailModalVisible && detailData" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.3); z-index:90; padding:20px;">
      <div style="width:90%; max-width:900px; max-height:90%; overflow:auto; background:#fff; border-radius:4px; border:1px solid #ddd; display:flex; flex-direction:column;">
        <!-- 头部 -->
        <div style="padding:20px; border-bottom:1px solid #e0e0e0; display:flex; justify-content:space-between; align-items:center;">
          <h3 style="margin:0; font-size:18px; font-weight:500; color:#333;">游戏详情</h3>
          <button @click="closeDetailModal" 
                  style="padding:6px 12px; font-size:13px; background:#fff; border:1px solid #ddd; color:#666; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#e8e8e8'; $event.target.style.borderColor='#999'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">关闭</button>
        </div>
        
        <!-- 主体内容 -->
        <div style="padding:20px; overflow-y:auto;">
          <div style="display:flex; gap:20px; flex-wrap:wrap;">
            <!-- 左侧封面 -->
            <div style="flex-shrink:0;">
              <img v-if="detailData.images?.large" :src="getImageSrc(detailData.images.large)" 
                   style="width:240px; height:auto; border-radius:4px; border:1px solid #e0e0e0;" />
              <div v-else style="width:240px; height:336px; background:#f0f0f0; border-radius:4px; display:flex; align-items:center; justify-content:center; color:#ccc; font-size:48px;">?</div>
            </div>
            
            <!-- 右侧信息 -->
            <div style="flex:1; min-width:300px;">
              <!-- 标题、运行状态和游戏时长 -->
              <div style="display:flex; align-items:baseline; gap:12px; margin-bottom:8px;">
                <h2 style="margin:0; font-size:20px; font-weight:600; color:#333;">
                  {{ detailData.name_cn || detailData.name }}
                </h2>
                <!-- 运行中状态 -->
                <div v-if="detailGame && runningGames.has(detailGame.path)" 
                     style="padding:4px 10px; background:rgba(76, 175, 80, 0.9); color:#fff; font-size:12px; font-weight:500; border-radius:3px; white-space:nowrap;">
                  运行中
                </div>
                <div v-if="detailGame && detailGame.playtime > 0" style="font-size:14px; color:#666; white-space:nowrap;">
                  ⏱ {{ formatPlaytime(detailGame.playtime) }}
                </div>
              </div>
              <div v-if="detailData.name_cn && detailData.name" style="margin-bottom:4px; font-size:14px; color:#999;">
                {{ detailData.name }}
              </div>
              <!-- 最后游玩时间 -->
              <div v-if="detailGame && detailGame.last_played" style="margin-bottom:16px; font-size:12px; color:#999;">
                最后游玩: {{ formatLastPlayed(detailGame.last_played) }}
              </div>
              <div v-else style="margin-bottom:16px;"></div>
              
              <!-- 基本信息 -->
              <div style="background:#f8f9fa; padding:16px; border-radius:4px; margin-bottom:16px; border:1px solid #e0e0e0;">
                <div style="display:grid; grid-template-columns:auto 1fr; gap:8px 16px; font-size:13px;">
                  <div style="color:#666; font-weight:500;">ID:</div>
                  <div style="color:#333;">{{ detailData.id }}</div>
                  
                  <div v-if="detailData.date" style="color:#666; font-weight:500;">发行日期:</div>
                  <div v-if="detailData.date" style="color:#333;">{{ detailData.date }}</div>
                  
                  <div v-if="getInfoboxValue('中文名')" style="color:#666; font-weight:500;">中文名:</div>
                  <div v-if="getInfoboxValue('中文名')" style="color:#333;">{{ getInfoboxValue('中文名') }}</div>
                  
                  <div v-if="getInfoboxValue('别名')" style="color:#666; font-weight:500;">别名:</div>
                  <div v-if="getInfoboxValue('别名')" style="color:#333;">{{ getInfoboxValue('别名') }}</div>
                  
                  <div v-if="getInfoboxValue('平台')" style="color:#666; font-weight:500;">平台:</div>
                  <div v-if="getInfoboxValue('平台')" style="color:#333;">{{ getInfoboxValue('平台') }}</div>
                  
                  <div v-if="getInfoboxValue('游戏类型')" style="color:#666; font-weight:500;">游戏类型:</div>
                  <div v-if="getInfoboxValue('游戏类型')" style="color:#333;">{{ getInfoboxValue('游戏类型') }}</div>
                  
                  <div v-if="getInfoboxValue('开发')" style="color:#666; font-weight:500;">开发:</div>
                  <div v-if="getInfoboxValue('开发')" style="color:#333;">{{ getInfoboxValue('开发') }}</div>
                  
                  <div v-if="getInfoboxValue('发行')" style="color:#666; font-weight:500;">发行:</div>
                  <div v-if="getInfoboxValue('发行')" style="color:#333;">{{ getInfoboxValue('发行') }}</div>
                </div>
              </div>
              
              <!-- 标签 -->
              <div v-if="detailData.meta_tags && detailData.meta_tags.length > 0" style="margin-bottom:16px;">
                <div style="font-size:13px; font-weight:500; color:#666; margin-bottom:8px;">标签</div>
                <div style="display:flex; gap:6px; flex-wrap:wrap;">
                  <span v-for="tag in detailData.meta_tags" :key="tag" 
                        style="padding:4px 10px; background:#f0f0f0; border-radius:3px; font-size:11px; color:#666; border:1px solid #e0e0e0;">
                    {{ tag }}
                  </span>
                </div>
              </div>
              
              <!-- 简介 -->
              <div v-if="detailData.summary" style="margin-bottom:16px;">
                <div style="font-size:13px; font-weight:500; color:#666; margin-bottom:8px;">简介</div>
                <div style="font-size:13px; color:#333; line-height:1.6; white-space:pre-wrap;">{{ detailData.summary }}</div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 底部操作栏 -->
        <div style="padding:16px 20px; border-top:1px solid #e0e0e0; display:flex; gap:8px; justify-content:flex-end; background:#f8f9fa;">
          <!-- 运行中时显示关闭按钮（红色），否则显示启动按钮（绿色） -->
          <button v-if="detailGame && runningGames.has(detailGame.path)" 
                  @click="closeGame(detailGame.path)" 
                  style="padding:8px 20px; font-size:13px; background:#f44336; border:1px solid #f44336; color:#fff; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#d32f2f'; $event.target.style.borderColor='#d32f2f'"
                  @mouseleave="$event.target.style.backgroundColor='#f44336'; $event.target.style.borderColor='#f44336'">
            关闭游戏
          </button>
          <button v-else-if="detailGame" 
                  @click="launchFromLibrary(detailGame)" 
                  style="padding:8px 20px; font-size:13px; background:#4CAF50; border:1px solid #4CAF50; color:#fff; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#45a049'; $event.target.style.borderColor='#45a049'"
                  @mouseleave="$event.target.style.backgroundColor='#4CAF50'; $event.target.style.borderColor='#4CAF50'">
            启动游戏
          </button>
          <button v-if="detailGame" @click="async () => { if (await removeGame(detailGame)) closeDetailModal() }" 
                  style="padding:8px 20px; font-size:13px; background:#fff; border:1px solid #ddd; color:#999; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#e8e8e8'; $event.target.style.borderColor='#999'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">
            删除游戏
          </button>
        </div>
      </div>
    </div>

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
/* 侧边栏按钮样式 */
aside button:hover {
  background-color: #f0f0f0;
}

aside button.active {
  background-color: #e8e8e8;
  border-left-color: #666 !important;
  font-weight: 500;
}

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
