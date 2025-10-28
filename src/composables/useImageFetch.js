import { ref } from 'vue';
import * as gameService from '../services/gameService';
import { useCache } from './useCache';

// useImageFetch 设计为与来自 useGameLibrary 的 games 引用配合使用，负责为无图的游戏
// 自动检索封面/条目并更新游戏信息与缓存。
export function useImageFetch(gamesRef) {
  const imageFetchRunning = ref(false);
  const loadedGamesCount = ref(0);
  const totalGamesCount = ref(0);
  const { getDetailCache, setDetailCache } = useCache();
  // 并发下载限制，避免在低端设备或大量游戏时阻塞 UI
  const CONCURRENCY = 3;

  async function fetchImageForGame(g) {
    try {
      const p = g.path || '';
      const parts = p.split(/[\\/]/);
      const exeName = parts[parts.length - 1] || p;

      let gameFolderNames = [];
      if (g.folder_path && Array.isArray(g.folder_path) && g.folder_path.length > 0) {
        gameFolderNames = [...g.folder_path];
      }

      if (gameFolderNames.length === 0) {
        const pathParts = p.split(/[\\/]/).filter(Boolean);
        if (pathParts.length >= 2) {
          gameFolderNames = [pathParts[pathParts.length - 2]];
        } else {
          return false;
        }
      }

      let list = [];
      const filter = { type: [4], nsfw: true };

      for (const folderName of gameFolderNames) {
        if (!folderName) continue;
        const res = await gameService.searchBangumi({ query: folderName, filter });
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
        if (list && list.length > 0) break;
      }

      if (list && list.length > 0) {
        const fdFlag = gameFolderNames.some(name => /fd/i.test(name)) || /fd/i.test(exeName);
        const idx = (fdFlag && list.length > 1) ? 1 : 0;
        const first = list[idx];

        const image = first.image || (first.subject && first.subject.image) || (first.images && (first.images.large || first.images.medium || first.images.small || first.images.common || first.images.grid)) || (first.subject && first.subject.images && (first.subject.images.large || first.subject.images.medium || first.subject.images.small));
        const title = first.name_cn || first.name || first.title || (first.subject && (first.subject.name_cn || first.subject.name || first.subject.title)) || null;

        if (image || title || first.id) {
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
          if (sid && isNaN(numericSid)) return false;

          const updRes = await gameService.updateGameInfo({ path: g.path, name: title, image, imageUrl: image, subjectId: numericSid });
          if (updRes && updRes.name) g.name = updRes.name;
          if (updRes && updRes.image) g.image = updRes.image;
          if (updRes && updRes.image_url) g.image_url = updRes.image_url;
          if (updRes && updRes.subject_id) g.subject_id = updRes.subject_id;

          if (numericSid && image) {
            try {
              const localImagePath = await gameService.downloadImage({ url: image, subjectId: numericSid });
              if (localImagePath) {
                await gameService.updateGameInfo({ path: g.path, name: null, image: localImagePath, imageUrl: null, subjectId: null });
                g.image = localImagePath;
              }

              const detailData = await gameService.getBangumiSubject(numericSid);
              if (detailData) {
                setDetailCache(numericSid, detailData);
              }
            } catch (err) {
              console.error('download or cache failed in useImageFetch:', err);
            }
          }
          return true;
        }
      }
      return false;
    } catch (e) {
      console.error('fetchImageForGame failed in useImageFetch', e);
      return false;
    }
  }

  async function autoFetchImages() {
    if (imageFetchRunning.value) return;
    imageFetchRunning.value = true;
    try {
      let processedCount = 0;
      const gamesWithoutImage = (gamesRef.value || []).filter(g => !g.image);
      totalGamesCount.value = gamesWithoutImage.length;
      loadedGamesCount.value = 0;

      // 并发执行 fetchImageForGame，使用简单的 worker 池
      const queue = gamesWithoutImage.slice();

      const workers = Array.from({ length: Math.max(1, Math.min(CONCURRENCY, queue.length)) }, async () => {
        while (queue.length > 0) {
          const g = queue.shift();
          try {
            await fetchImageForGame(g);
          } catch (err) {
            console.error('fetchImageForGame (worker) error:', err);
          } finally {
            processedCount++;
            loadedGamesCount.value = processedCount;
            // 小延迟让 UI 有机会渲染进度
            await new Promise(r => setTimeout(r, 80));
          }
        }
      });

      await Promise.all(workers);
    } finally {
      imageFetchRunning.value = false;
    }
  }

  return {
    imageFetchRunning,
    loadedGamesCount,
    totalGamesCount,
    fetchImageForGame,
    autoFetchImages,
  };
}

export default useImageFetch;
