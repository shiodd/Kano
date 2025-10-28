import { ref } from 'vue';
import * as gameService from '../services/gameService';

// 全局单例缓存，供整个应用共享。
const detailCache = ref(new Map());

function getDetailCacheKey(subjectId) {
  return `subject:${subjectId}`;
}

function setDetailCache(subjectId, data) {
  const key = getDetailCacheKey(subjectId);
  const filteredData = {
    id: data.id,
    name: data.name,
    name_cn: data.name_cn,
    date: data.date,
    summary: data.summary,
    meta_tags: data.meta_tags,
    images: data.images ? { large: `kano_data/images/${data.id}.jpg` } : null,
    infobox: data.infobox ? data.infobox.filter(item => ['中文名', '别名', '平台', '游戏类型', '开发', '发行'].includes(item.key)) : null
  };
  detailCache.value.set(key, filteredData);
  saveCacheToFile();
}

function getDetailCache(subjectId) {
  const key = getDetailCacheKey(subjectId);
  return detailCache.value.get(key) || null;
}

function removeDetailCache(subjectId) {
  if (!subjectId) return;
  const key = getDetailCacheKey(subjectId);
  detailCache.value.delete(key);
  saveCacheToFile();
}

function clearAllCache() {
  detailCache.value.clear();
  saveCacheToFile();
}

async function loadCacheFromFile() {
  try {
    const obj = await gameService.loadCache();
    if (obj && typeof obj === 'object') {
      detailCache.value = new Map(Object.entries(obj));
    }
  } catch (e) {
    console.error('Failed to load cache in useCache:', e);
  }
}

let saveTimer = null;
async function saveCacheToFile() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    try {
      const cacheObj = Object.fromEntries(detailCache.value);
      await gameService.saveCache(cacheObj);
    } catch (e) {
      console.error('Failed to save cache in useCache:', e);
    }
  }, 500);
}

export function useCache() {
  return {
    detailCache,
    getDetailCache,
    setDetailCache,
    removeDetailCache,
    clearAllCache,
    loadCacheFromFile,
    saveCacheToFile,
  };
}

export default useCache;
