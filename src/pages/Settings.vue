<template>
  <div>
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

    <!-- 网络测试 -->
    <div style="margin-bottom:20px; padding:16px; background:#f8f9fa; border-radius:4px; border:1px solid #e0e0e0;">
      <label style="display:block; font-weight:500; margin-bottom:8px; font-size:13px; color:#333;">网络连接测试</label>
      <div style="font-size:12px; color:#999; margin-bottom:10px;">
        测试与 Bangumi 的网络连接是否正常
      </div>
      <div style="display:flex; align-items:center; gap:12px;">
        <button @click="testNetwork" 
                :disabled="isTesting"
                style="padding:8px 16px; font-size:13px;">
          {{ isTesting ? '检测中...' : '测试连接' }}
        </button>
        <div v-if="networkStatus !== null" style="display:flex; align-items:center; gap:8px;">
          <div :style="{
            width: '10px',
            height: '10px',
            borderRadius: '50%',
            backgroundColor: networkStatus ? '#4CAF50' : '#f44336'
          }"></div>
          <span :style="{
            fontSize: '13px',
            color: networkStatus ? '#4CAF50' : '#f44336',
            fontWeight: '500'
          }">
            {{ networkStatus ? 'Bangumi 连接正常' : 'Bangumi 无法连接' }}
          </span>
          <span v-if="networkStatus && latency !== null" style="font-size:12px; color:#999;">
            ({{ latency }}ms)
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const accessToken = ref('');
const showToken = ref(false);
const isTesting = ref(false);
const networkStatus = ref(null); // null: 未测试, true: 正常, false: 失败
const latency = ref(null); // 延迟时间（毫秒）

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

async function testNetwork() {
  isTesting.value = true;
  networkStatus.value = null;
  latency.value = null;
  
  try {
    const result = await invoke('test_network_connection');
    networkStatus.value = result.success;
    latency.value = result.latency;
  } catch (e) {
    networkStatus.value = false;
    latency.value = null;
  } finally {
    isTesting.value = false;
  }
}

onMounted(() => {
  loadToken();
});
</script>
