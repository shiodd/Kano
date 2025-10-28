import { invoke } from '@tauri-apps/api/core';

// 轻量的 Tauri invoke 调用封装。将前端与直接调用 invoke 的细节隔离开。
export async function loadCache() {
  return await invoke('load_cache');
}

export async function greet(name) {
  return await invoke('greet', { name });
}

export async function saveCache(cache) {
  return await invoke('save_cache', { cache });
}

export async function getProjectRoot() {
  return await invoke('get_project_root');
}

export async function listGames() {
  return await invoke('list_games');
}

export async function addGame({ path, name, folderPath }) {
  return await invoke('add_game', { path, name, folderPath });
}

export async function removeGame(path) {
  return await invoke('remove_game', { path });
}

export async function updateGameInfo({ path, name, image, imageUrl, subjectId }) {
  return await invoke('update_game_info', { path, name, image, imageUrl, subjectId });
}

export async function downloadImage({ url, subjectId }) {
  return await invoke('download_image', { url, subjectId });
}

export async function getBangumiSubject(id) {
  return await invoke('get_bangumi_subject', { id });
}

export async function searchBangumi({ query, filter }) {
  return await invoke('search_bangumi', { query, filter });
}

export async function pickExe({ initialDir = null } = {}) {
  return await invoke('pick_exe', { initialDir });
}

export async function pickFolderAndScan() {
  return await invoke('pick_folder_and_scan');
}

export async function launchExe({ path }) {
  return await invoke('launch_exe', { path });
}

export async function killGame({ path }) {
  return await invoke('kill_game', { path });
}

export async function listExes({ game_dir }) {
  return await invoke('list_exes', { game_dir });
}

export async function updateGamePlaytime({ path, additionalSeconds, lastPlayed }) {
  return await invoke('update_game_playtime', { path, additionalSeconds, lastPlayed });
}

export async function deleteCachedImage({ subjectId }) {
  return await invoke('delete_cached_image', { subjectId });
}

export default {
  loadCache,
  greet,
  saveCache,
  getProjectRoot,
  listGames,
  addGame,
  removeGame,
  updateGameInfo,
  downloadImage,
  getBangumiSubject,
  searchBangumi,
  pickExe,
  pickFolderAndScan,
  launchExe,
  killGame,
  listExes,
  updateGamePlaytime,
  deleteCachedImage,
};
