<template>
  <aside style="width:180px; background:#f8f9fa; border-right:1px solid #e0e0e0; padding:0; position:fixed; left:0; top:0; bottom:0; display:flex; flex-direction:column;">
    <div style="padding:20px 16px; border-bottom:1px solid #e0e0e0;">
      <h2 style="margin:0; font-size:16px; font-weight:600; color:#333;">kano</h2>
    </div>
    <nav style="flex:1; padding:12px 0; display:flex; flex-direction:column; overflow:hidden;">
      <button :class="{active: modelValue === 'library'}" 
              @click.stop="toggleLibrary" 
              style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent; display:flex; justify-content:space-between; align-items:center;">
        <span>游戏库</span>
        <span style="font-size:12px; transition:transform 0.2s;" :style="{ transform: showTags ? 'rotate(90deg)' : 'rotate(0deg)' }">▶</span>
      </button>
      
      <!-- 标签列表（游戏库的子项，可折叠，可滚动） -->
      <div v-show="showTags" style="padding:0 0 8px 8px; overflow-y:auto; max-height:calc(100vh - 300px);">
        <div style="display:flex; align-items:center; justify-content:space-between; padding:6px 16px 8px 8px; margin-top:6px;">
          <span style="font-size:12px; color:#999; font-weight:600;">标签筛选</span>
          <button @click="openTagManagement"
                  :class="{ 'active': modelValue === 'tag-management', 'compact-manage': true }"
                  title="打开标签管理"
                  aria-label="打开标签管理">
            管理
          </button>
        </div>
        <div style="display:flex; flex-direction:column; gap:2px;">
          <button v-for="tag in allTags" :key="tag.name"
                  :class="{active: selectedTag === tag.name}" 
                  @click="handleTagClick(tag.name)"
                  style="text-align:left; padding:6px 8px 6px 16px; font-size:12px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-radius:4px; display:flex; justify-content:space-between; align-items:center;">
            <span>{{ tag.name }}</span>
            <span style="font-size:11px; color:#999;">{{ tag.count }}</span>
          </button>
        </div>
      </div>

      <!-- 将工具箱按钮放在分割线之上，靠近标签区，方便访问 -->
      <button :class="{active: modelValue === 'toolbox'}" @click="$emit('update:modelValue', 'toolbox')" 
              style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent; width:100%; display:block;">
        工具箱
      </button>

      <div style="flex:1;"></div>
      
      <!-- 固定在底部的设置和关于按钮 -->
      <div style="padding-top:8px; border-top:1px solid #e0e0e0;">
        <button :class="{active: modelValue === 'settings'}" @click="$emit('update:modelValue', 'settings')" 
                style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent; width:100%; display:block;">
          设置
        </button>
        <button :class="{active: modelValue === 'about'}" @click="$emit('update:modelValue', 'about')" 
                style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent; width:100%; display:block;">
          关于
        </button>
      </div>
    </nav>
  </aside>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

defineProps({
  modelValue: {
    type: String,
    required: true
  },
  selectedTag: {
    type: String,
    default: null
  }
});

const emit = defineEmits(['update:modelValue', 'tag-selected', 'tags-updated']);

const allTags = ref([]);
const showTags = ref(false); // 默认折叠标签列表

function toggleLibrary() {
  // 总是切换到游戏库页面
  emit('update:modelValue', 'library');
  
  // 切换标签显示状态
  showTags.value = !showTags.value;
  
  // 如果折叠了，清除标签筛选
  if (!showTags.value) {
    emit('tag-selected', null);
  }
}

function openTagManagement() {
  // 切换到标签管理页面并清除当前标签筛选，确保父组件会响应
  // Important: emit tag-selected first to avoid parent handler forcing activeTab back to 'library'
  emit('tag-selected', null);
  emit('update:modelValue', 'tag-management');
  // 展开标签区域以便用户看到管理入口（可选）
  showTags.value = true;
}

function handleTagClick(tag) {
  // 点击标签时自动切换到游戏库页面
  emit('update:modelValue', 'library');
  emit('tag-selected', tag);
}

async function confirmDeleteTag(tag) {
  const count = allTags.value.find(t => t.name === tag)?.count || 0;
  const message = count > 0 
    ? `确定要删除标签"${tag}"吗？\n这将从 ${count} 个游戏中移除此标签。`
    : `确定要删除标签"${tag}"吗？`;
  
  if (!confirm(message)) return;
  
  try {
    await invoke('remove_custom_tag', { tag });
    await loadTags();
    emit('tags-updated');
    // 如果删除的是当前选中的标签，清除筛选
    if (props.selectedTag === tag) {
      emit('tag-selected', null);
    }
  } catch (error) {
    alert('删除标签失败: ' + error);
  }
}

async function loadTags() {
  try {
    const tags = await invoke('get_all_tags');
    const tagCounts = [];
    for (const tag of tags) {
      const count = await invoke('get_games_count_by_tag', { tag });
      tagCounts.push({ name: tag, count });
    }
    allTags.value = tagCounts;
  } catch (error) {
    console.error('加载标签失败:', error);
  }
}

onMounted(() => {
  loadTags();
});

defineExpose({
  loadTags
});
</script>

<style scoped>
/* 侧边栏主按钮样式 */
nav > button.active,
nav > div > button.active {
  background: #e8e8e8 !important;
  color: #333 !important;
  font-weight: 500;
  border-left: 2px solid #666 !important;
}

nav > button:hover,
nav > div > button:hover {
  background: #f0f0f0;
  color: #333;
}

/* 标签按钮样式 - 子项样式，使用更具体的选择器 */
nav > div:first-of-type button {
  border-left: 2px solid transparent !important;
}

nav > div:first-of-type button.active {
  background: #e3f2fd !important;
  color: #1976d2 !important;
  font-weight: 500;
  border-left: 2px solid #1976d2 !important;
}

nav > div:first-of-type button:hover {
  background: #f5f5f5 !important;
}

/* 管理按钮（标签区域）统一样式 */
/* compact manage button override to keep dark background on hover and match size */
.compact-manage {
  padding: 2px 6px !important;
  font-size: 12px !important;
  background: transparent !important;
  color: #666 !important;
  border: none !important;
  border-radius: 0 !important;
  font-weight: 600;
}
.compact-manage:hover {
  background: transparent !important;
  color: #1976d2 !important;
}
/* when tag-management is active, make the manage button appear like other tag items */
nav > div:first-of-type button.compact-manage.active {
  background: #e3f2fd !important;
  color: #1976d2 !important;
  font-weight: 500 !important;
  border-left: 2px solid #1976d2 !important;
}
</style>
