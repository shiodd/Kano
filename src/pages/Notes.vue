<template>
  <div style="padding:20px;">
    <div class="notes-header">
      <h2 style="margin:0;">记录</h2>
      <div class="note-search">
        <input v-model="searchQuery" placeholder="按游戏名搜索记录" />
      </div>
    </div>
    <div style="margin-top:12px; display:flex; gap:12px;">
      <div style="flex:1; max-width:480px;">
        <div style="display:flex; flex-direction:column; gap:12px;">
          <div v-for="(note, idx) in filteredNotes" :key="note.game_id || idx" @click.stop.prevent="openModal(note)" class="note-item">
            <div class="note-main">
              <div class="note-title">{{ note.title || '(无标题)' }}</div>
              <div class="note-game">{{ note.game_name || '未关联游戏' }}</div>
              <div class="note-excerpt">{{ (note.content || '').replace(/\n/g, ' ') }}</div>
            </div>
            <div style="display:flex; gap:8px; margin-left:12px;">
              <button @click.stop.prevent="remove(note.game_id, idx)" class="note-delete">删除</button>
            </div>
          </div>
        </div>
      </div>
    </div>
    <!-- Note editor modal for viewing/editing -->
    <NoteEditorModal :visible="noteModalVisible" :note="selectedNote" @close="noteModalVisible = false" @saved="onNoteSaved" @deleted="onNoteDeleted" />
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue';
import gameService from '../services/gameService';
import NoteEditorModal from '../components/NoteEditorModal.vue';

const notes = ref([]);
const selectedNote = ref(null);
const noteModalVisible = ref(false);
const searchQuery = ref('');

const filteredNotes = computed(() => {
  const q = String(searchQuery.value || '').trim().toLowerCase();
  if (!q) return notes.value;
  return notes.value.filter(n => (n.game_name || '').toLowerCase().includes(q));
});

async function load() {
  try {
    const res = await gameService.listNotes();
    notes.value = Array.isArray(res) ? res : [];
  } catch (e) {
    console.error('load notes failed', e);
    notes.value = [];
  }
}


function openModal(note) {
  selectedNote.value = { ...note };
  noteModalVisible.value = true;
}

function onNoteSaved(saved) {
  // refresh list and close modal
  load();
  noteModalVisible.value = false;
}

function onNoteDeleted(id) {
  load();
  noteModalVisible.value = false;
}



async function remove(gameId, idx) {
  if (!confirm('确认删除该记录吗？')) return;
  try {
    if (gameId && String(gameId).trim() !== '') {
      await gameService.deleteNote(gameId);
    } else {
      await gameService.deleteNoteByIndex(idx);
    }
    await load();
  } catch (e) {
    alert('删除失败: ' + e);
  }
}

onMounted(() => {
  load();
});
</script>

<style scoped>
.note-item{
  padding:14px;
  border:1px solid #e6e6e6;
  border-radius:6px;
  display:flex;
  align-items:flex-start;
  cursor:pointer;
  background:#fff;
  min-height:86px;
}
.note-item:hover{ box-shadow:0 6px 18px rgba(0,0,0,0.06); }
.note-main{ flex:1; overflow:hidden; }
.note-title{ font-weight:700; color:#222; font-size:15px; margin-bottom:6px; white-space:nowrap; text-overflow:ellipsis; overflow:hidden; }
.note-game{ font-size:12px; color:#777; margin-bottom:8px; }
.note-excerpt{
  font-size:13px;
  color:#444;
  line-height:1.4;
  max-height:4.2em; /* ~3 lines */
  overflow:hidden;
  display:-webkit-box;
  line-clamp:3;
  -webkit-line-clamp:3;
  -webkit-box-orient:vertical;
  white-space:normal;
  text-overflow:ellipsis;
}
.note-delete{ padding:6px 10px; background:#f44336; border:1px solid #f44336; color:#fff; border-radius:4px; cursor:pointer; }
.note-delete:hover{ background:#d32f2f; border-color:#d32f2f; }
/* search input in header */
.note-search input{ padding:6px 10px; font-size:13px; width:220px; border:1px solid #ddd; border-radius:4px; }
.note-search input:focus{ outline:none; border-color:#bbb; box-shadow:0 0 0 3px rgba(0,0,0,0.03); }

/* header sticky + tighter spacing so search is closer to title */
.notes-header{
  position:sticky;
  top:0;
  background:#fff;
  z-index:15;
  display:flex;
  align-items:center;
  justify-content:flex-start;
  gap:10px;
  padding:8px 0 12px 0;
  border-bottom:1px solid #f2f2f2;
}


</style>
