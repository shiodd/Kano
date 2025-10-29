<template>
  <div v-if="visible" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.4); z-index:200;">
  <div style="width:640px; max-width:95%; background:#fff; border-radius:6px; box-shadow:0 8px 24px rgba(0,0,0,0.2); max-height:80vh; display:flex; flex-direction:column; overflow:hidden;">
      <!-- header (fixed) -->
        <div style="position:sticky; top:0; background:#fff; z-index:20; padding:12px 16px; border-bottom:1px solid #eee;">
  <div style="display:flex; align-items:center; justify-content:space-between; gap:12px;">
              <div style="display:flex; align-items:center; gap:12px;">
                <h3 style="margin:0; font-size:16px;">记录详情</h3>
                <div v-if="local.game_name" style="padding:4px 8px; background:#fafafa; border-radius:4px; border:1px solid #eee; color:#333; font-weight:600;">{{ local.game_name }}</div>
              </div>

              <div>
                <button class="modal-close" @click="$emit('close')">关闭</button>
              </div>
            </div>
          </div>

      <!-- body (scrollable) -->
      <div style="padding:16px; flex:1 1 auto; overflow:auto;">
        <div style="display:flex; flex-direction:column; gap:8px;">
          <div style="display:flex; gap:8px; align-items:center;">
            <template v-if="isEditing">
              <input v-model="local.title" placeholder="标题（可选）" style="flex:1; padding:8px; font-size:14px;" />
            </template>
            <template v-else>
              <div style="flex:1; padding:8px 0; font-size:16px; font-weight:600;">{{ local.title || '(无标题)' }}</div>
            </template>
          </div>
          <div>
            <template v-if="isEditing">
              <textarea v-model="local.content" rows="8" placeholder="在此记录你的笔记..." style="padding:8px; font-size:14px; width:100%; box-sizing:border-box; resize:vertical; max-height:60vh; max-width:100%; overflow:auto;"></textarea>
            </template>
            <template v-else>
              <div style="padding:8px; font-size:14px; width:100%; box-sizing:border-box; white-space:pre-wrap;">{{ local.content }}</div>
            </template>
          </div>
        </div>
      </div>
      <!-- footer (fixed) -->
      <div style="position:sticky; bottom:0; background:#fff; padding:12px 16px; border-top:1px solid #eee; display:flex; align-items:center; justify-content:space-between; gap:8px; z-index:20;">
        <div style="font-size:12px; color:#666;">
          <div>创建时间: {{ formatTime(local.created_at) }}</div>
          <div>更新时间: {{ formatTime(local.updated_at || local.created_at) }}</div>
        </div>
        <div style="display:flex; gap:8px;">
          <template v-if="isEditing">
            <button @click="save" class="btn-primary">保存</button>
            <button @click="remove" v-if="hasGameId" class="btn-danger">删除</button>
            <button @click="cancelEdit" class="modal-close">取消</button>
          </template>
          <template v-else>
            <button @click="enterEdit" class="modal-close">编辑</button>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { reactive, watch, computed } from 'vue';
import gameService from '../services/gameService';

const props = defineProps({
  visible: { type: Boolean, required: true },
  note: { type: Object, default: null }
});
const emit = defineEmits(['close', 'saved', 'deleted']);

const local = reactive({ title: '', content: '', game_id: '', game_name: '', created_at: '', updated_at: '' });
import { ref } from 'vue';
const isEditing = ref(false);

function enterEdit() {
  isEditing.value = true;
}

function cancelEdit() {
  // revert local to props.note
  const n = props.note;
  if (n) {
    local.title = n.title || '';
    local.content = n.content || '';
    local.game_id = n.game_id ? String(n.game_id) : '';
    local.game_name = n.game_name || '';
    local.created_at = n.created_at || '';
    local.updated_at = n.updated_at || '';
  }
  isEditing.value = false;
}

watch(() => props.note, (n) => {
  if (n) {
    local.title = n.title || '';
    local.content = n.content || '';
    local.game_id = n.game_id ? String(n.game_id) : '';
    local.game_name = n.game_name || '';
    local.created_at = n.created_at || '';
    local.updated_at = n.updated_at || '';
    // default to view mode when opening a note
    isEditing.value = false;
  } else {
    local.title = '';
    local.content = '';
    local.game_id = '';
    local.game_name = '';
  }
  // 同步本地内容
  local.content = local.content || local.content;
}, { immediate: true });

const hasGameId = computed(() => local.game_id && String(local.game_id).trim() !== '');

async function save() {
  try {
    // local.content 已由 textarea 双向绑定，直接使用即可

    const payload = {
      game_id: String(local.game_id || ''),
      game_name: String(local.game_name || ''),
      title: String(local.title || ''),
      content: String(local.content || ''),
    };
    const saved = await gameService.saveNote(payload);
    emit('saved', saved);
    emit('close');
  } catch (e) {
    alert('保存记录失败: ' + e);
  }
}

async function remove() {
  if (!confirm('确认删除该记录吗？')) return;
  try {
    if (hasGameId.value) {
      await gameService.deleteNote(local.game_id);
      emit('deleted', local.game_id);
      emit('close');
    } else {
      alert('当前记录未关联游戏，无法按游戏删除。');
    }
  } catch (e) {
    alert('删除记录失败: ' + e);
  }
}

function formatTime(iso) {
  if (!iso) return '-';
  try {
    const d = new Date(iso);
    return d.toLocaleString();
  } catch (e) {
    return iso;
  }
}
</script>

<style scoped>
.modal-close{
  padding:6px 12px;
  font-size:13px;
  background:#fff;
  border:1px solid #ddd;
  color:#666;
  border-radius:4px;
  cursor:pointer;
  transition:all 0.2s;
}
.modal-close:hover{
  background:#e8e8e8;
  border-color:#999;
}

.btn-primary{
  padding:6px 12px;
  font-size:13px;
  background:#4CAF50;
  border:1px solid #4CAF50;
  color:#fff;
  border-radius:4px;
  cursor:pointer;
  transition:all 0.2s;
}
.btn-primary:hover{ background:#45a049; border-color:#45a049; }

.btn-danger{
  padding:6px 12px;
  font-size:13px;
  background:#f44336;
  border:1px solid #f44336;
  color:#fff;
  border-radius:4px;
  cursor:pointer;
  transition:all 0.2s;
}
.btn-danger:hover{ background:#d32f2f; border-color:#d32f2f; }
</style>
