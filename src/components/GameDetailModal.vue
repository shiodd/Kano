<template>
  <div v-if="visible && detailData" ref="overlayRef" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.3); z-index:90; padding:20px;">
    <!--
      使用边界感知的事件处理，防止在 modal 内滚动到边界时触发后面的页面滚动（滚动穿透）。
      通过 overlayRef 捕获 wheel / touchmove（非 passive）并在必要时 preventDefault/stopPropagation。
    -->
    <div ref="modalRef" style="width:90%; max-width:900px; max-height:90%; overflow:auto; background:#fff; border-radius:4px; border:1px solid #ddd; display:flex; flex-direction:column;">
      <!-- 头部 -->
      <div style="padding:20px; border-bottom:1px solid #e0e0e0; display:flex; justify-content:space-between; align-items:center;">
        <h3 style="margin:0; font-size:18px; font-weight:500; color:#333;">游戏详情</h3>
        <div style="display:flex; gap:8px; align-items:center;">
          <button v-if="game" @click="openNotesEditor" 
                  style="padding:6px 12px; font-size:13px; background:#fff; border:1px solid #ddd; color:#666; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#e8e8e8'; $event.target.style.borderColor='#999'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">记录</button>
          <button @click="$emit('close')" 
                  style="padding:6px 12px; font-size:13px; background:#fff; border:1px solid #ddd; color:#666; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#e8e8e8'; $event.target.style.borderColor='#999'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">关闭</button>
        </div>
      </div>
      
      <!-- 主体内容 -->
      <div style="padding:20px; overflow-y:auto;">
        <div style="display:flex; gap:20px; flex-wrap:wrap;">
          <!-- 左侧封面 -->
          <div style="flex-shrink:0;">
            <img v-if="detailData.images?.large" :src="imageSrc" 
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
              <div v-if="game && isRunning" 
                   style="padding:4px 10px; background:rgba(76, 175, 80, 0.9); color:#fff; font-size:12px; font-weight:500; border-radius:3px; white-space:nowrap;">
                运行中
              </div>
              <div v-if="game && game.playtime > 0" style="font-size:14px; color:#666; white-space:nowrap;">
                ⏱ {{ formatPlaytime(game.playtime) }}
              </div>
            </div>
            <div v-if="detailData.name_cn && detailData.name" style="margin-bottom:4px; font-size:14px; color:#999;">
              {{ detailData.name }}
            </div>
            <!-- 最后游玩时间 -->
            <div v-if="game && game.last_played" style="margin-bottom:16px; font-size:12px; color:#999;">
              最后游玩: {{ formatLastPlayed(game.last_played) }}
            </div>
            <div v-else style="margin-bottom:16px;"></div>
            
            <!-- 基本信息 -->
            <div style="background:#f8f9fa; padding:16px; border-radius:4px; margin-bottom:16px; border:1px solid #e0e0e0;">
              <div style="display:grid; grid-template-columns:auto 1fr; gap:8px 16px; font-size:13px;">
                <div style="color:#666; font-weight:500;">ID:</div>
                <div style="color:#333;">{{ detailData.id }}</div>
                
                <template v-if="detailData.date">
                  <div style="color:#666; font-weight:500;">发行日期:</div>
                  <div style="color:#333;">{{ detailData.date }}</div>
                </template>
                
                <template v-if="getInfoboxValue('中文名')">
                  <div style="color:#666; font-weight:500;">中文名:</div>
                  <div style="color:#333;">{{ getInfoboxValue('中文名') }}</div>
                </template>
                
                <template v-if="getInfoboxValue('别名')">
                  <div style="color:#666; font-weight:500;">别名:</div>
                  <div style="color:#333;">{{ getInfoboxValue('别名') }}</div>
                </template>
                
                <template v-if="getInfoboxValue('平台')">
                  <div style="color:#666; font-weight:500;">平台:</div>
                  <div style="color:#333;">{{ getInfoboxValue('平台') }}</div>
                </template>
                
                <template v-if="getInfoboxValue('游戏类型')">
                  <div style="color:#666; font-weight:500;">游戏类型:</div>
                  <div style="color:#333;">{{ getInfoboxValue('游戏类型') }}</div>
                </template>
                
                <template v-if="getInfoboxValue('开发')">
                  <div style="color:#666; font-weight:500;">开发:</div>
                  <div style="color:#333;">{{ getInfoboxValue('开发') }}</div>
                </template>
                
                <template v-if="getInfoboxValue('发行')">
                  <div style="color:#666; font-weight:500;">发行:</div>
                  <div style="color:#333;">{{ getInfoboxValue('发行') }}</div>
                </template>
              </div>
            </div>
            
            <!-- Bangumi 标签 -->
            <div v-if="detailData.meta_tags && detailData.meta_tags.length > 0" style="margin-bottom:16px;">
              <div style="font-size:13px; font-weight:500; color:#666; margin-bottom:8px;">Bangumi 标签</div>
              <div style="display:flex; gap:6px; flex-wrap:wrap;">
                <span v-for="tag in detailData.meta_tags" :key="tag" 
                      style="padding:4px 10px; background:#f0f0f0; border-radius:3px; font-size:11px; color:#666; border:1px solid #e0e0e0;">
                  {{ tag }}
                </span>
              </div>
            </div>
            
            <!-- 自定义标签 -->
            <div v-if="game" style="margin-bottom:16px;">
              <div style="font-size:13px; font-weight:500; color:#666; margin-bottom:8px;">我的标签</div>
              <div style="display:flex; gap:6px; flex-wrap:wrap; align-items:center;">
                <span v-for="tag in (game.tags || [])" :key="tag"
                      @click="removeTagFromGame(tag)"
                      style="padding:4px 10px; background:#e3f2fd; border-radius:3px; font-size:11px; color:#1976d2; border:1px solid #90caf9; cursor:pointer; transition:all 0.2s; display:flex; align-items:center; gap:4px;"
                      @mouseenter="$event.target.style.backgroundColor='#bbdefb'"
                      @mouseleave="$event.target.style.backgroundColor='#e3f2fd'">
                  {{ tag }}
                  <span style="font-weight:bold; font-size:12px;">×</span>
                </span>
                <button @click="showTagSelector = !showTagSelector"
                        style="padding:4px 10px; background:#fff; border:1px solid #ddd; border-radius:3px; font-size:11px; color:#666; cursor:pointer; transition:all 0.2s;"
                        @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                        @mouseleave="$event.target.style.backgroundColor='#fff'">
                  + 添加标签
                </button>
              </div>
              
              <!-- 标签选择器/输入器 -->
              <div v-if="showTagSelector" style="margin-top:8px; padding:12px; background:#f8f9fa; border:1px solid #e0e0e0; border-radius:4px;">
                <!-- 已有标签列表 -->
                <div style="display:flex; gap:6px; flex-wrap:wrap; margin-bottom:12px;">
                  <button v-for="tag in availableTags" :key="tag"
                          @click="addTagToGame(tag)"
                          :disabled="game.tags && game.tags.includes(tag)"
                          :style="{
                            padding: '4px 10px',
                            background: game.tags && game.tags.includes(tag) ? '#f5f5f5' : '#fff',
                            border: '1px solid #ddd',
                            borderRadius: '3px',
                            fontSize: '11px',
                            color: game.tags && game.tags.includes(tag) ? '#ccc' : '#666',
                            cursor: game.tags && game.tags.includes(tag) ? 'not-allowed' : 'pointer',
                            transition: 'all 0.2s'
                          }"
                          @mouseenter="!(game.tags && game.tags.includes(tag)) && ($event.target.style.backgroundColor='#e3f2fd')"
                          @mouseleave="!(game.tags && game.tags.includes(tag)) && ($event.target.style.backgroundColor='#fff')">
                    {{ tag }}
                  </button>
                </div>
                
                <!-- 新建标签输入框 -->
                <div style="display:flex; gap:8px; align-items:center; padding-top:8px; border-top:1px solid #e0e0e0;">
                  <input v-model="newTagName" 
                         @keyup.enter="addNewTag"
                         placeholder="输入新标签名称"
                         style="flex:1; padding:6px 10px; border:1px solid #ddd; border-radius:3px; font-size:12px; outline:none;">
                  <button @click="addNewTag"
                          :disabled="!newTagName.trim()"
                          :style="{
                            padding: '6px 12px',
                            background: newTagName.trim() ? '#888' : '#f5f5f5',
                            color: newTagName.trim() ? 'white' : '#ccc',
                            border: 'none',
                            borderRadius: '3px',
                            fontSize: '11px',
                            cursor: newTagName.trim() ? 'pointer' : 'not-allowed',
                            transition: 'all 0.2s'
                          }">
                    创建
                  </button>
                </div>
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
        <button v-if="game && isRunning" 
                @click="$emit('close-game')" 
                style="padding:8px 20px; font-size:13px; background:#f44336; border:1px solid #f44336; color:#fff; transition:all 0.2s;"
                @mouseenter="$event.target.style.backgroundColor='#d32f2f'; $event.target.style.borderColor='#d32f2f'"
                @mouseleave="$event.target.style.backgroundColor='#f44336'; $event.target.style.borderColor='#f44336'">
          关闭游戏
        </button>
        <button v-else-if="game" 
                @click="$emit('launch-game')" 
                style="padding:8px 20px; font-size:13px; background:#4CAF50; border:1px solid #4CAF50; color:#fff; transition:all 0.2s;"
                @mouseenter="$event.target.style.backgroundColor='#45a049'; $event.target.style.borderColor='#45a049'"
                @mouseleave="$event.target.style.backgroundColor='#4CAF50'; $event.target.style.borderColor='#4CAF50'">
          启动游戏
        </button>
        <button v-if="game" 
                @click="!isRunning && $emit('delete-game')"
                :disabled="isRunning"
                :style="{
                  padding: '8px 20px',
                  fontSize: '13px',
                  background: isRunning ? '#f5f5f5' : '#fff',
                  border: '1px solid #ddd',
                  color: isRunning ? '#ccc' : '#999',
                  cursor: isRunning ? 'not-allowed' : 'pointer',
                  transition: 'all 0.2s'
                }"
                @mouseenter="!isRunning && ($event.target.style.backgroundColor='#e8e8e8', $event.target.style.borderColor='#999')"
                @mouseleave="!isRunning && ($event.target.style.backgroundColor='#fff', $event.target.style.borderColor='#ddd')">
          删除游戏
        </button>
      </div>
      <!-- 记录编辑器：使用独立弹窗显示（由 NoteEditorModal 提供） -->
      <NoteEditorModal :visible="noteEditorVisible" :note="noteObj" @close="noteEditorVisible = false" @saved="onNoteSaved" @deleted="onNoteDeleted" />
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import gameService from '../services/gameService';
import NoteEditorModal from './NoteEditorModal.vue';

const props = defineProps({
  visible: {
    type: Boolean,
    required: true
  },
  detailData: {
    type: Object,
    default: null
  },
  game: {
    type: Object,
    default: null
  },
  isRunning: {
    type: Boolean,
    default: false
  },
  imageSrc: {
    type: String,
    default: ''
  }
});

const emit = defineEmits(['close', 'launch-game', 'close-game', 'delete-game', 'tags-updated']);

const showTagSelector = ref(false);
const availableTags = ref([]);
const newTagName = ref('');
const noteEditorVisible = ref(false);
const noteObj = ref(null);

async function loadAvailableTags() {
  try {
    availableTags.value = await invoke('get_all_tags');
  } catch (error) {
    console.error('加载标签失败:', error);
  }
}

async function addNewTag() {
  if (!newTagName.value.trim()) return;
  
  try {
    // 先添加到自定义标签列表
    await invoke('add_custom_tag', { tag: newTagName.value.trim() });
    
    // 然后添加到当前游戏
    await addTagToGame(newTagName.value.trim());
    
    // 重新加载可用标签列表
    await loadAvailableTags();
    
    // 清空输入框
    newTagName.value = '';
  } catch (error) {
    alert('创建标签失败: ' + error);
  }
}

async function addTagToGame(tag) {
  if (!props.game) return;
  try {
    await invoke('add_tag_to_game', { path: props.game.path, tag });
    // 更新本地数据
    if (!props.game.tags) {
      props.game.tags = [];
    }
    if (!props.game.tags.includes(tag)) {
      props.game.tags.push(tag);
    }
    emit('tags-updated');
  } catch (error) {
    alert('添加标签失败: ' + error);
  }
}

async function removeTagFromGame(tag) {
  if (!props.game) return;
  try {
    await invoke('remove_tag_from_game', { path: props.game.path, tag });
    // 更新本地数据
    if (props.game.tags) {
      const index = props.game.tags.indexOf(tag);
      if (index > -1) {
        props.game.tags.splice(index, 1);
      }
    }
    emit('tags-updated');
  } catch (error) {
    alert('移除标签失败: ' + error);
  }
}

watch(() => props.visible, (newVal) => {
  if (newVal) {
    loadAvailableTags();
    showTagSelector.value = false;
    newTagName.value = '';
  }
});

// 在 modal 打开时锁定 body 滚动，关闭时恢复（作为额外保险，彻底避免背景滚动）
let _prevBodyOverflow = null;
watch(() => props.visible, (visible) => {
  try {
    const docBody = document && document.body;
    if (!docBody) return;
    if (visible) {
      _prevBodyOverflow = docBody.style.overflow;
      docBody.style.overflow = 'hidden';
    } else {
      if (_prevBodyOverflow !== null) docBody.style.overflow = _prevBodyOverflow;
      else docBody.style.overflow = '';
      _prevBodyOverflow = null;
    }
  } catch (e) {
    // 在非浏览器环境或测试环境忽略
  }
});

onMounted(() => {
  if (props.visible) {
    loadAvailableTags();
  }
});

// Notes editor helpers
async function openNotesEditor() {
  if (!props.game) return;
  noteEditorVisible.value = true;
  // try to find an existing note for this game path
  try {
  const notes = await gameService.listNotes();
  const gameId = String((props.detailData && props.detailData.id) || (props.game && props.game.id) || '');
    const gameName = (props.detailData && (props.detailData.name_cn || props.detailData.name)) || (props.game && props.game.name) || '';
    if (Array.isArray(notes)) {
  const found = notes.find(n => String(n.game_id || '') === gameId);
      if (found) {
        noteObj.value = { ...found };
        return;
      }
    }
  } catch (e) {
    console.error('加载记录失败:', e);
  }
  // new note
  noteObj.value = {
    id: '',
    game_id: String((props.detailData && props.detailData.id) || (props.game && props.game.id) || ''),
    game_name: String((props.detailData && (props.detailData.name_cn || props.detailData.name)) || (props.game && props.game.name) || ''),
    title: '',
    content: ''
  };
}

async function saveNote() {
  if (!noteObj.value) return;
  // ensure game id/name present and normalize to strings before sending to backend
  if (!noteObj.value.game_id) {
    noteObj.value.game_id = String((props.detailData && props.detailData.id) || (props.game && props.game.id) || '');
  } else {
    noteObj.value.game_id = String(noteObj.value.game_id);
  }
  if (!noteObj.value.game_name) {
    noteObj.value.game_name = String((props.detailData && (props.detailData.name_cn || props.detailData.name)) || (props.game && props.game.name) || '');
  } else {
    noteObj.value.game_name = String(noteObj.value.game_name);
  }

  try {
    // normalize payload to ensure all fields expected as strings are strings
    const payload = {
      ...noteObj.value,
      id: String(noteObj.value.id || ''),
      game_id: String(noteObj.value.game_id || ''),
      game_name: String(noteObj.value.game_name || ''),
      title: String(noteObj.value.title || ''),
      content: String(noteObj.value.content || ''),
    };
    const saved = await gameService.saveNote(payload);
    // update local state with returned object (id, timestamps)
    noteObj.value = { ...saved };
    noteEditorVisible.value = false;
    // emit a DOM event so other parts can optionally listen
    try { window.dispatchEvent(new CustomEvent('notes-updated')); } catch (e) {}
  } catch (e) {
    alert('保存记录失败: ' + e);
  }
}

async function deleteNote() {
  if (!noteObj.value || !noteObj.value.id) return;
  if (!confirm('确定要删除这条记录吗？')) return;
  try {
    if (noteObj.value.game_id && String(noteObj.value.game_id).trim() !== '') {
      await gameService.deleteNote(noteObj.value.game_id);
      noteObj.value = null;
      noteEditorVisible.value = false;
      try { window.dispatchEvent(new CustomEvent('notes-updated')); } catch (e) {}
    }
  } catch (e) {
    alert('删除记录失败: ' + e);
  }
}

function onNoteSaved(saved) {
  // update local state and notify others
  noteObj.value = { ...saved };
  try { window.dispatchEvent(new CustomEvent('notes-updated')); } catch (e) {}
}

function onNoteDeleted(gameId) {
  noteObj.value = null;
  try { window.dispatchEvent(new CustomEvent('notes-updated')); } catch (e) {}
}

function getInfoboxValue(key) {
  if (!props.detailData?.infobox) return null;
  const item = props.detailData.infobox.find(i => i.key === key);
  if (!item) return null;
  if (Array.isArray(item.value)) {
    return item.value.map(v => v.v || v).join(', ');
  }
  return item.value;
}

function formatPlaytime(seconds) {
  if (!seconds) return '0分钟';
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours > 0) {
    return `${hours}小时${minutes}分钟`;
  }
  return `${minutes}分钟`;
}

function formatLastPlayed(timestamp) {
  if (!timestamp) return '';
  // 支持 ISO 字符串和时间戳（秒）两种格式
  const date = typeof timestamp === 'string' ? new Date(timestamp) : new Date(timestamp * 1000);
  const now = new Date();
  const diffMs = now - date;
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);
  
  if (diffMins < 1) return '刚刚';
  if (diffMins < 60) return `${diffMins}分钟前`;
  if (diffHours < 24) return `${diffHours}小时前`;
  if (diffDays < 7) return `${diffDays}天前`;
  
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}
// 边界感知的滚动穿透防护逻辑

const overlayRef = ref(null);
const modalRef = ref(null);

// 查找从 target 向上最近的可滚动元素（限制在 modalRef 内）
function findScrollableWithinModal(target) {
  let el = target;
  const modal = modalRef.value;
  while (el && el !== document.body) {
    if (el === modal) {
      // modal 本身可滚动时返回 modal
      if (modal && modal.scrollHeight > modal.clientHeight) return modal;
      return null;
    }
    if (el.scrollHeight > el.clientHeight) {
      // 发现内部可滚动元素
      return el;
    }
    el = el.parentElement;
  }
  return null;
}

function onWheel(e) {
  // 仅在 modal 可见时处理
  if (!overlayRef.value || !modalRef.value) return;
  const target = e.target;
  const scrollEl = findScrollableWithinModal(target) || modalRef.value;
  if (!scrollEl) {
    // 没有可滚动元素，阻止冒泡以避免背景滚动
    e.preventDefault();
    e.stopPropagation();
    return;
  }

  const delta = e.deltaY;
  const atTop = scrollEl.scrollTop <= 0;
  const atBottom = Math.ceil(scrollEl.scrollTop + scrollEl.clientHeight) >= scrollEl.scrollHeight;

  // 当尝试继续向上滚动并且已经在顶部，或继续向下滚动并且已经在底部，阻止事件穿透
  if ((delta < 0 && atTop) || (delta > 0 && atBottom)) {
    e.preventDefault();
    e.stopPropagation();
  }
  // 否则允许正常滚动（不阻止）
}

let touchStartY = 0;
let touchScrollEl = null;

function onTouchStart(e) {
  if (!modalRef.value) return;
  touchStartY = e.touches && e.touches.length ? e.touches[0].clientY : 0;
  touchScrollEl = findScrollableWithinModal(e.target) || modalRef.value;
}

function onTouchMove(e) {
  if (!touchScrollEl) {
    e.preventDefault();
    e.stopPropagation();
    return;
  }
  const currentY = e.touches && e.touches.length ? e.touches[0].clientY : 0;
  const delta = touchStartY - currentY;
  const atTop = touchScrollEl.scrollTop <= 0;
  const atBottom = Math.ceil(touchScrollEl.scrollTop + touchScrollEl.clientHeight) >= touchScrollEl.scrollHeight;

  if ((delta < 0 && atTop) || (delta > 0 && atBottom)) {
    // 在边界继续滑动，阻止穿透
    e.preventDefault();
    e.stopPropagation();
  }
}

onMounted(() => {
  // 使用 capture 并且 passive: false，这样可以在 handler 中调用 preventDefault
  const overlay = overlayRef.value;
  if (!overlay) return;
  overlay.addEventListener('wheel', onWheel, { capture: true, passive: false });
  overlay.addEventListener('touchstart', onTouchStart, { capture: true, passive: true });
  overlay.addEventListener('touchmove', onTouchMove, { capture: true, passive: false });
});

onBeforeUnmount(() => {
  const overlay = overlayRef.value;
  if (!overlay) return;
  overlay.removeEventListener('wheel', onWheel, { capture: true, passive: false });
  overlay.removeEventListener('touchstart', onTouchStart, { capture: true, passive: true });
  overlay.removeEventListener('touchmove', onTouchMove, { capture: true, passive: false });
});

</script>
