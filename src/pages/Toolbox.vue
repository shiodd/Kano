<template>
  <div style="padding:20px;">
    <h2>工具箱</h2>
    <p style="color:#666;">在这里添加你常用的工具</p>

    <div class="toolbox-grid" style="margin-top:16px; gap:20px;">
      <!-- 左侧：工具列表 + 添加工具（简洁， 无参数） -->
      <div class="tool-list">
        <h3 style="margin-bottom:8px;">已添加的工具</h3>

        <div class="add-tool-compact" style="margin-bottom:12px; display:flex; gap:8px; align-items:center;">
          <input v-model="name" placeholder="名称" class="input" style="flex:1;" />
          <input v-model="path" placeholder="可执行路径" class="input" style="flex:1;" />
          <button @click="pickExe" class="btn">选择</button>
          <button @click="addTool" class="btn primary">添加</button>
        </div>

        <div v-if="loading">加载中...</div>
        <div v-else>
          <div v-if="tools.length === 0" style="color:#999;">暂无工具，使用上方添加。</div>
          <div v-else style="display:flex; flex-direction:column; gap:8px;">
            <div v-for="t in tools" :key="t.id" class="tool-item">
              <div style="flex:1;">
                <div style="font-weight:600;">{{ t.name }}</div>
                <div style="font-size:12px; color:#666;">{{ t.path }}</div>
              </div>
              <div style="display:flex; gap:8px;">
                <button @click="launch(t.id)" class="btn">启动</button>
                <button @click="remove(t.id)" class="btn btn-ghost">删除</button>
              </div>
            </div>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const name = ref('');
const path = ref('');
const tools = ref([]);
const loading = ref(false);

async function loadTools() {
  loading.value = true;
  try {
    const res = await invoke('get_tools');
    tools.value = res || [];
  } catch (e) {
    console.error('get_tools failed', e);
  } finally {
    loading.value = false;
  }
}

onMounted(loadTools);

async function pickExe() {
  try {
    const p = await invoke('pick_exe', { initialDir: null });
    if (!p) return;
    path.value = String(p);
    // if name is empty, prefill with exe filename (without extension)
    if (!name.value || name.value.trim() === '') {
      try {
        const s = String(p).replace(/\\/g, '/');
        const parts = s.split('/');
        const file = parts[parts.length - 1] || s;
        name.value = file.replace(/\.[^/.]+$/, '');
      } catch (e) {
        // ignore
      }
    }
  } catch (e) {
    if (e === 'cancelled' || e === 'canceled') return;
    alert('选择失败: ' + e);
  }
}

async function addTool() {
  if (!path.value) return alert('请填写可执行路径');
  // if name is empty, use exe filename (without extension)
  if (!name.value || name.value.trim() === '') {
    try {
      const s = String(path.value).replace(/\\/g, '/');
      const parts = s.split('/');
      const file = parts[parts.length - 1] || s;
      name.value = file.replace(/\.[^/.]+$/, '');
    } catch (e) {
      // fallback
      name.value = path.value;
    }
  }

  try {
    const added = await invoke('add_tool', { name: name.value, path: path.value });
    tools.value.push(added);
    name.value = '';
    path.value = '';
  } catch (e) {
    alert('添加失败: ' + e);
  }
}

async function remove(id) {
  if (!confirm('确定删除该工具吗？')) return;
  try {
    await invoke('remove_tool', { id });
    tools.value = tools.value.filter(t => t.id !== id);
  } catch (e) {
    alert('删除失败: ' + e);
  }
}

async function launch(id) {
  try {
    await invoke('launch_tool', { id });
  } catch (e) {
    alert('启动失败: ' + e);
  }
}
</script>

<style scoped>
.toolbox-grid {
  display: flex;
  align-items: flex-start;
}
.tool-list {
  flex: 1 1 60%;
  min-width: 240px;
}
.tool-form {
  flex: 0 0 320px;
  background: #fafafa;
  border: 1px solid #eee;
  padding: 12px;
  border-radius: 6px;
}
.tool-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px;
  border: 1px solid #eee;
  border-radius: 6px;
  background: #fff;
}
.add-tool-compact .input {
  height: 36px;
}
.record-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px;
  border: 1px solid #eee;
  border-radius: 6px;
  background: #fff;
}
.input {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}
.btn {
  padding: 6px 10px;
  border: 1px solid #ccc;
  background: #fff;
  border-radius: 4px;
  cursor: pointer;
}
.btn:hover { background: #f5f5f5; }
.btn-ghost { border-color: #ddd; background: #fff; }
.btn.primary {
  background: #1976d2;
  color: #fff;
  border-color: #1976d2;
}

@media (max-width: 880px) {
  .toolbox-grid { flex-direction: column; }
  .tool-form { width: 100%; flex: 1 1 auto; }
}

.add-tool-compact .input {
  height: 36px;
}
</style>
