<template>
  <div>
    <h1 style="margin:0 0 20px 0; font-size:20px; font-weight:500; color:#333;">标签管理</h1>
    
    <div style="background:#f8f9fa; padding:20px; border-radius:8px; border:1px solid #e0e0e0;">
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; gap:12px;">
        <div>
          <div style="font-size:14px; font-weight:500; color:#666;">现有标签</div>
          <div style="font-size:12px; color:#999;">可多选后批量删除</div>
        </div>
        <div style="display:flex; gap:8px; align-items:center;">
          <input v-model="newTagName" @keyup.enter="addTag" placeholder="新增标签名" 
                 style="padding:6px 10px; border:1px solid #ddd; border-radius:4px; font-size:13px;" />
          <button @click="addTag" :disabled="!newTagName.trim()" 
                  style="padding:6px 12px; font-size:13px; background:#1976d2; color:#fff; border:none; border-radius:4px; cursor:pointer;">
            新增
          </button>
          <button @click="deleteSelectedTags" :disabled="selectedTags.size === 0"
                  :style="{
                    padding: '6px 12px',
                    fontSize: '13px',
                    background: selectedTags.size > 0 ? '#f44336' : '#f5f5f5',
                    color: selectedTags.size > 0 ? '#fff' : '#999',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: selectedTags.size > 0 ? 'pointer' : 'not-allowed'
                  }">
            删除选中 ({{ selectedTags.size }})
          </button>
        </div>
      </div>

      <div style="margin-bottom:16px;">
        <!-- compact grid of fixed-size tag tiles -->
        <div style="display:grid; grid-template-columns:repeat(auto-fill, 140px); gap:10px; align-items:start;">
          <div v-for="tag in allTags" :key="tag.name" style="width:140px;">
            <div @click="toggleTagSelection(tag.name)" style="background:#fff; border:1px solid #e0e0e0; border-radius:6px; padding:10px; display:flex; flex-direction:column; gap:8px; min-height:64px; box-sizing:border-box; cursor:pointer;">
              <div style="display:flex; align-items:center; gap:8px;">
                <input type="checkbox" :checked="selectedTags.has(tag.name)" @click.stop="toggleTagSelection(tag.name)" />
                <div style="overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:13px; font-weight:500; color:#333;">
                  {{ tag.name }}
                </div>
              </div>
              <div style="display:flex; justify-content:space-between; align-items:center;">
                <div style="font-size:12px; color:#999;">{{ tag.count }} 个</div>
                <button @click.stop.prevent="confirmDeleteTag(tag.name, tag.count)"
                        style="padding:4px 8px; background:#f44336; color:white; border:none; border-radius:4px; cursor:pointer; font-size:12px;"
                        @mouseenter="$event.target.style.backgroundColor='#d32f2f'"
                        @mouseleave="$event.target.style.backgroundColor='#f44336'">
                  删除
                </button>
              </div>
            </div>
          </div>
        </div>

        <div v-if="allTags.length === 0" style="text-align:center; padding:40px 20px; color:#ccc; font-size:14px;">
          暂无标签
        </div>
      </div>

      <div style="margin-top:20px; padding-top:20px; border-top:1px solid #e0e0e0;">
        <div style="font-size:13px; color:#999; line-height:1.6;">
          <p style="margin:0 0 8px 0;">提示：</p>
          <p style="margin:0 0 4px 0;">• 删除标签会将其从所有游戏中移除</p>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const emit = defineEmits(['tags-updated']);

const allTags = ref([]);
const selectedTags = ref(new Set());
const newTagName = ref('');

async function loadTags() {
  try {
    const tags = await invoke('get_all_tags');
    const tagCounts = [];
    for (const tag of tags) {
      const count = await invoke('get_games_count_by_tag', { tag });
      tagCounts.push({ name: tag, count });
    }
    allTags.value = tagCounts;
    // 清理已删除但仍在 selectedTags 中的项
    for (const t of Array.from(selectedTags.value)) {
      if (!allTags.value.find(x => x.name === t)) selectedTags.value.delete(t);
    }
  } catch (error) {
    console.error('加载标签失败:', error);
  }
}

function toggleTagSelection(tag) {
  if (selectedTags.value.has(tag)) selectedTags.value.delete(tag);
  else selectedTags.value.add(tag);
}

async function addTag() {
  const name = String(newTagName.value || '').trim();
  if (!name) return;
  try {
    await invoke('add_custom_tag', { tag: name });
    newTagName.value = '';
    await loadTags();
    emit('tags-updated');
  } catch (err) {
    alert('新增标签失败: ' + err);
  }
}

async function confirmDeleteTag(tag, count) {
  const message = count > 0 
    ? `确定要删除标签"${tag}"吗？\n这将从 ${count} 个游戏中移除此标签。`
    : `确定要删除标签"${tag}"吗？`;
  
  if (!confirm(message)) return;
  
  try {
    await invoke('remove_custom_tag', { tag });
    await loadTags();
    emit('tags-updated');
  } catch (error) {
    alert('删除标签失败: ' + error);
  }
}

async function deleteSelectedTags() {
  if (selectedTags.value.size === 0) return;
  const names = Array.from(selectedTags.value);
  if (!confirm(`确定要删除选中的 ${names.length} 个标签吗？\n这会从所有游戏中移除这些标签。`)) return;
  try {
    for (const t of names) {
      await invoke('remove_custom_tag', { tag: t });
    }
    selectedTags.value = new Set();
    await loadTags();
    emit('tags-updated');
  } catch (err) {
    alert('批量删除失败: ' + err);
  }
}

onMounted(() => {
  loadTags();
});

defineExpose({
  loadTags
});
</script>
