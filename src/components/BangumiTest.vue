<template>
  <div style="padding:12px;">
    <h3>Bangumi API 测试（POST /v0/search/subjects）</h3>
    <div style="display:flex; gap:8px; align-items:center; margin-bottom:8px;">
      <input v-model="testKeyword" placeholder="关键词（keyword）" style="flex:1" />
      <div style="padding:6px 10px; background:#fff; border-radius:6px; font-size:13px;">类型: 游戏 (固定)</div>
      <label style="display:flex; align-items:center; gap:6px;"><input type="checkbox" v-model="testNsfw"/> NSFW</label>
      <button @click="runBangumiTest">测试</button>
    </div>
    <div style="max-height:480px; overflow:auto; background:#fafafa; padding:8px; border-radius:4px;">
      <div v-if="!list || list.length === 0" style="color:#666">测试结果将显示在此。</div>
      <ul v-else style="list-style:none; padding:0; display:grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap:8px;">
  <li v-for="(item, idx) in list" :key="(item.data && item.data.id) || item.id || idx" style="display:flex; gap:8px; align-items:flex-start; padding:8px; border:1px solid #eee; background:#fff; border-radius:6px;">
          <div style="width:72px; height:100px; flex:0 0 72px; display:flex; align-items:center; justify-content:center; overflow:hidden;">
            <img v-if="getImage(item)" :src="getImage(item)" style="max-width:100%; max-height:100%; cursor:pointer;" @click="openPreview(getImage(item))" />
            <div v-else style="color:#999; font-size:12px">无图</div>
          </div>
          <div style="flex:1">
            <div style="font-weight:600">{{ item.name_cn || item.name || item.title || item.subject?.name_cn || item.subject?.name || item.subject?.title || '未知标题' }}</div>
            <div style="font-size:12px; color:#666; margin-top:6px;">ID: {{ (item.data && item.data.id) || item.id || item.subject?.id || 'N/A' }} - {{ item.type || item.subject?.type || 'N/A' }}</div>
            <div style="margin-top:8px; font-size:13px; color:#333">{{ item.summary || item.desc || item.subject?.summary || '' }}</div>
          </div>
        </li>
      </ul>
    </div>

    <!-- Subject by ID fetch -->
    <div style="margin-top:12px; padding:8px; background:#fff; border-radius:6px; border:1px solid #eee;">
      <h4 style="margin:0 0 8px 0;">按 ID 获取条目 (GET /v0/subject/{id})</h4>
      <div style="display:flex; gap:8px; align-items:center; margin-bottom:8px;">
        <input v-model="subjectId" placeholder="输入 subject id（例如 12345 ）" style="flex:1" />
        <button :disabled="subjectLoading || !subjectId" @click="fetchSubjectById">获取</button>
        <div v-if="subjectLoading" style="color:#666; font-size:13px; padding-left:6px;">加载中…</div>
      </div>

      <div v-if="subjectError" style="color:#b00020; background:#fff6f6; padding:8px; border-radius:4px; margin-bottom:8px;">错误: {{ subjectError }}</div>

      <div v-if="subjectData" style="background:#f7f9fb; padding:8px; border-radius:4px; max-height:320px; overflow:auto; font-size:13px;">
        <div style="display:flex; gap:12px; align-items:flex-start;">
          <div style="width:120px; flex:0 0 120px;">
            <img v-if="getImage(subjectData)">
            <img v-if="getImage(subjectData)" :src="getImage(subjectData)" style="max-width:120px; max-height:160px; display:block;" />
          </div>
          <div style="flex:1">
            <div style="font-weight:700; font-size:16px;">{{ subjectData.name_cn || subjectData.name || subjectData.title || '（无标题）' }}</div>
            <div style="color:#666; font-size:13px; margin-top:6px;">ID: {{ subjectData.id || subjectData.data?.id || 'N/A' }} · 类型: {{ subjectData.type || subjectData.data?.type || 'N/A' }}</div>
            <div style="margin-top:8px; color:#333; font-size:13px;">
              <pre style="white-space:pre-wrap; word-break:break-word; margin:0;">{{ formattedSubjectJson }}</pre>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showPreview" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.6); z-index:60;">
      <div style="max-width:90%; max-height:90%; background:#fff; padding:10px; border-radius:6px;">
        <img :src="previewUrl" style="max-width:100%; max-height:80vh; display:block;" />
        <div style="text-align:right; margin-top:8px;"><button @click="closePreview">关闭</button></div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const testKeyword = ref('');
const testType = ref('4');
const testNsfw = ref(false);
const testResult = ref(null);
const list = ref([]);
const showPreview = ref(false);
const previewUrl = ref('');

// Subject by ID
const subjectId = ref('');
const subjectLoading = ref(false);
const subjectError = ref('');
const subjectData = ref(null);

import { computed } from 'vue';
const formattedSubjectJson = computed(() => {
  try {
    return subjectData.value ? JSON.stringify(subjectData.value, null, 2) : '';
  } catch (e) {
    return String(subjectData.value || '');
  }
});

function openPreview(url) {
  previewUrl.value = url;
  showPreview.value = true;
}

function closePreview() {
  previewUrl.value = '';
  showPreview.value = false;
}

function extractList(res) {
  if (!res) return [];
  if (Array.isArray(res)) return res;
  if (res.data && Array.isArray(res.data)) return res.data;
  if (res.results && Array.isArray(res.results)) return res.results;
  if (res.subjects && Array.isArray(res.subjects)) return res.subjects;
  if (res.items && Array.isArray(res.items)) return res.items;
  for (const k in res) {
    if (Array.isArray(res[k])) return res[k];
  }
  return [];
}

function getImage(item) {
  // prefer top-level "image", then images.large/medium/small/common/grid
  if (!item) return null;
  const tryKeys = (obj) => {
    if (!obj) return null;
    if (typeof obj === 'string') return obj;
    if (obj.image) return obj.image;
    if (obj.images) {
      return obj.images.large || obj.images.medium || obj.images.small || obj.images.common || obj.images.grid;
    }
    return null;
  };

  // item may be wrapper: { subject: { ... } }
  let url = tryKeys(item) || tryKeys(item.subject) || tryKeys(item.data) || tryKeys(item.item) || null;
  return url;
}

async function runBangumiTest() {
  try {
    const filter = { type: [4], nsfw: testNsfw.value };
    const res = await invoke('search_bangumi', { query: testKeyword.value, filter });
    testResult.value = res;
    list.value = extractList(res);
  } catch (e) {
    testResult.value = { error: String(e) };
  }
}

async function fetchSubjectById() {
  subjectError.value = '';
  subjectData.value = null;
  if (!subjectId.value) return;
  subjectLoading.value = true;
  try {
    const idNum = Number(subjectId.value);
    if (!Number.isFinite(idNum) || idNum <= 0) {
      throw new Error('无效的 id');
    }
    const res = await invoke('get_bangumi_subject', { id: idNum });
    subjectData.value = res;
  } catch (e) {
    subjectError.value = String(e);
  } finally {
    subjectLoading.value = false;
  }
}
</script>
