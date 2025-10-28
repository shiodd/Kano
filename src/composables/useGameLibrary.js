import { ref } from 'vue';
import * as gameService from '../services/gameService';

export function useGameLibrary() {
  const games = ref([]);
  const isLoadingGames = ref(false);
  const projectRoot = ref('');

  async function loadProjectRoot() {
    try {
      projectRoot.value = await gameService.getProjectRoot();
    } catch (e) {
      console.error('getProjectRoot failed', e);
    }
  }

  async function listGames() {
    try {
      isLoadingGames.value = true;
      const res = await gameService.listGames();
    games.value = res || [];
  // 保持游戏列表大小的追踪；图片抓取进度由 image fetch composable 管理
      
    } catch (e) {
      console.error('listGames failed', e);
      games.value = [];
    } finally {
      isLoadingGames.value = false;
    }
  }

  async function addGame({ path, name, folderPath }) {
    return await gameService.addGame({ path, name, folderPath });
  }

  async function removeGame(path) {
    return await gameService.removeGame(path);
  }

  async function updateGameInfo(params) {
    return await gameService.updateGameInfo(params);
  }

  async function pickExe(initialDir = null) {
    return await gameService.pickExe({ initialDir });
  }

  async function pickFolderAndScan() {
    return await gameService.pickFolderAndScan();
  }

  async function launchExe(path) {
    return await gameService.launchExe({ path });
  }

  async function killGame(path) {
    return await gameService.killGame({ path });
  }

  async function listExes(game_dir) {
    return await gameService.listExes({ game_dir });
  }

  async function updateGamePlaytime(params) {
    return await gameService.updateGamePlaytime(params);
  }

  return {
    games,
    isLoadingGames,
  // （进度计数已迁移到 useImageFetch）
    projectRoot,
    loadProjectRoot,
    listGames,
    addGame,
    removeGame,
    updateGameInfo,
    pickExe,
    pickFolderAndScan,
    launchExe,
    killGame,
    listExes,
    updateGamePlaytime,
  };
}

export default useGameLibrary;
