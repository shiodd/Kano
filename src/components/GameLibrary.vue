<template>
  <div>
  <!-- 标题栏（固定在顶部） -->
    <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px; position:sticky; top:0; background:#fff; z-index:40; padding:8px 0; box-shadow:0 2px 6px rgba(0,0,0,0.04);">
      <div style="display:flex; gap:12px; align-items:center; flex:1; margin-left:-4px;">
        <h1 style="margin:0; font-size:20px; font-weight:500; color:#333;">游戏库</h1>
        
        <!-- 搜索框 -->
        <div style="flex:1; max-width:300px;">
          <input :value="searchKeyword" 
                 @input="$emit('update:searchKeyword', $event.target.value)"
                 placeholder="搜索游戏..." 
                 style="width:100%; padding:6px 12px; font-size:13px; border:1px solid #e0e0e0; border-radius:4px; outline:none;"
                 @focus="$event.target.style.borderColor='#999'"
                 @blur="$event.target.style.borderColor='#e0e0e0'" />
        </div>
        
        <!-- 筛选按钮 -->
        <div style="display:flex; gap:4px; align-items:center; padding:4px; background:#fff; border-radius:4px; border:1px solid #e0e0e0;">
          <button v-for="filter in filters" :key="filter"
                  @click="$emit('update:selectedFilter', filter)" 
                  :style="{
                    padding: '4px 12px',
                    fontSize: '12px',
                    background: selectedFilter === filter ? '#e8e8e8' : '#fff',
                    border: selectedFilter === filter ? '1px solid #999' : '1px solid #ddd',
                    color: selectedFilter === filter ? '#333' : '#666',
                    fontWeight: selectedFilter === filter ? '500' : '400',
                    borderRadius: '3px',
                    cursor: 'pointer'
                  }">
            {{ filter }}
          </button>
        </div>
        <div style="font-size:12px; color:#999;">
          {{ filteredGames.length }} / {{ games.length }}
        </div>
      </div>
      
      <!-- 右上角菜单 -->
      <div style="position:relative;">
        <button @click.stop="showMenu = !showMenu" 
                style="padding:8px 16px; font-size:13px; white-space:nowrap; display:flex; align-items:center; gap:4px;"
                @mouseenter="$event.target.style.backgroundColor='#f0f0f0'"
                @mouseleave="$event.target.style.backgroundColor='#fff'">
          操作 ▼
        </button>
        
        <!-- 下拉菜单 -->
        <div v-if="showMenu" 
             @click.stop
             style="position:absolute; top:100%; right:0; margin-top:4px; background:#fff; border:1px solid #e0e0e0; border-radius:4px; box-shadow:0 2px 8px rgba(0,0,0,0.1); z-index:100; min-width:160px;">
          <button @click="$emit('pick-exe'); showMenu = false" 
                  style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'">
            选择游戏
          </button>
          <button @click="$emit('scan-folder'); showMenu = false" 
                  :disabled="isScanningFolder"
                  style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'">
            {{ isScanningFolder ? '扫描中...' : '扫描文件夹' }}
          </button>
          <div style="height:1px; background:#e0e0e0; margin:4px 0;"></div>
          <button @click="$emit('toggle-multi-select'); showMenu = false" 
                  style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                  @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'">
            {{ isMultiSelectMode ? '退出多选模式' : '多选模式' }}
          </button>
          <template v-if="isMultiSelectMode">
            <button @click="$emit('select-all'); showMenu = false" 
                    style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                    @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                    @mouseleave="$event.target.style.backgroundColor='#fff'">
              全选
            </button>
            <button @click="$emit('deselect-all'); showMenu = false" 
                    style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#333; cursor:pointer; transition:all 0.2s;"
                    @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                    @mouseleave="$event.target.style.backgroundColor='#fff'">
              取消全选
            </button>
            <button @click="$emit('delete-selected'); showMenu = false" 
                    :disabled="selectedGames.size === 0"
                    style="width:100%; text-align:left; padding:10px 16px; font-size:13px; background:#fff; border:none; color:#f44336; cursor:pointer; transition:all 0.2s;"
                    @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                    @mouseleave="$event.target.style.backgroundColor='#fff'">
              删除选中 {{ selectedGames.size > 0 ? `(${selectedGames.size})` : '' }}
            </button>
          </template>
        </div>
      </div>
    </div>
    
    <!-- 选中的游戏显示 -->
    <div v-if="selectedExe" style="margin-bottom:16px; padding:12px; background:#f8f9fa; border-radius:4px; border:1px solid #e0e0e0; display:flex; gap:8px; align-items:center;">
      <div style="flex:1; font-size:12px; color:#666; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;" :title="selectedExe">
        已选择: {{ selectedExe }}
      </div>
      <button @click="$emit('add-to-library')" style="padding:6px 12px; font-size:12px; white-space:nowrap;">加入游戏库</button>
    </div>

    <!-- 游戏列表 -->
    <div v-if="games.length === 0" style="text-align:center; padding:60px 20px; color:#ccc;">
      <div style="font-size:14px;">暂无游戏</div>
    </div>
    
    <div v-else-if="filteredGames.length === 0" style="text-align:center; padding:60px 20px; color:#ccc;">
      <div style="font-size:14px;">当前筛选条件下没有游戏</div>
    </div>
    
    <div v-else style="display:grid; grid-template-columns:repeat(auto-fill, 140px); gap:12px;">
      <GameCard v-for="g in filteredGames" :key="g.path"
                :game="g"
                :is-running="runningGames.has(g.path)"
                :is-selected="selectedGames.has(g.path)"
                :is-multi-select-mode="isMultiSelectMode"
                :image-src="g.image"
                @click="isMultiSelectMode ? $emit('toggle-selection', g) : $emit('open-detail', g)"
                @launch="$emit('launch', g)"
                @close="$emit('close-game', g.path)"
                @replace="$emit('open-replace', g)"
                @delete="$emit('delete-game', g)" />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import GameCard from './GameCard.vue';

const props = defineProps({
  games: {
    type: Array,
    required: true
  },
  filteredGames: {
    type: Array,
    required: true
  },
  runningGames: {
    type: Set,
    required: true
  },
  selectedGames: {
    type: Set,
    required: true
  },
  searchKeyword: {
    type: String,
    default: ''
  },
  selectedFilter: {
    type: String,
    default: '全部'
  },
  isMultiSelectMode: {
    type: Boolean,
    default: false
  },
  selectedExe: {
    type: String,
    default: ''
  },
  isScanningFolder: {
    type: Boolean,
    default: false
  }
});

defineEmits([
  'update:searchKeyword',
  'update:selectedFilter',
  'pick-exe',
  'scan-folder',
  'toggle-multi-select',
  'select-all',
  'deselect-all',
  'delete-selected',
  'add-to-library',
  'toggle-selection',
  'open-detail',
  'launch',
  'close-game',
  'open-replace',
  'delete-game'
]);

const filters = ['全部', 'ADV', 'RPG'];
const showMenu = ref(false);
</script>
