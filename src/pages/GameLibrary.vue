<template>
  <div @click="showMenu = false; showFilterMenu = false">
    <!-- 标题栏 -->
    <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:16px;">
      <div style="display:flex; gap:12px; align-items:center; flex:1;">
        <h1 style="margin:0; font-size:20px; font-weight:500; color:#333;">游戏库</h1>
        
        <!-- 当前标签筛选提示 -->
        <div v-if="selectedTag" style="display:flex; align-items:center; gap:8px; padding:4px 12px; background:#e3f2fd; border:1px solid #90caf9; border-radius:4px;">
          <span style="font-size:13px; color:#1976d2;">{{ selectedTag }}</span>
          <button @click="$emit('clear-tag-filter')" 
                  style="padding:2px 6px; font-size:11px; background:transparent; border:none; color:#1976d2; cursor:pointer; font-weight:bold;">
            ×
          </button>
        </div>
        
        <!-- 搜索框 -->
        <div style="flex:1; max-width:200px;">
          <input :value="searchKeyword" 
                 @input="$emit('update:searchKeyword', $event.target.value)"
                 placeholder="搜索游戏..." 
                 style="width:100%; padding:6px 12px; font-size:13px; border:1px solid #e0e0e0; border-radius:4px; outline:none;"
                 @focus="$event.target.style.borderColor='#999'"
                 @blur="$event.target.style.borderColor='#e0e0e0'" />
        </div>
        
        <!-- 筛选下拉框 -->
        <div style="position:relative;">
          <button @click.stop="showFilterMenu = !showFilterMenu" 
                  style="padding:6px 12px; font-size:13px; white-space:nowrap; display:flex; align-items:center; gap:4px;"
                  @mouseenter="$event.target.style.backgroundColor='#f0f0f0'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'">
            {{ selectedFilter }} ▼
          </button>
          
          <!-- 筛选菜单 -->
          <div v-if="showFilterMenu" 
               @click.stop
               style="position:absolute; top:100%; left:0; margin-top:4px; background:#fff; border:1px solid #e0e0e0; border-radius:4px; box-shadow:0 2px 8px rgba(0,0,0,0.1); z-index:100; min-width:100px;">
            <button v-for="filter in filters" :key="filter"
                    @click="$emit('update:selectedFilter', filter); showFilterMenu = false" 
                    :style="{
                      width: '100%',
                      textAlign: 'left',
                      padding: '8px 16px',
                      fontSize: '13px',
                      background: selectedFilter === filter ? '#f5f5f5' : '#fff',
                      border: 'none',
                      color: '#333',
                      fontWeight: selectedFilter === filter ? '500' : '400',
                      cursor: 'pointer',
                      transition: 'all 0.2s'
                    }"
                    @mouseenter="$event.target.style.backgroundColor='#f5f5f5'"
                    @mouseleave="$event.target.style.backgroundColor = selectedFilter === filter ? '#f5f5f5' : '#fff'">
              {{ filter }}
            </button>
          </div>
        </div>
        
        <div style="font-size:12px; color:#999;">
          {{ filteredGames.length }} / {{ games.length }}
        </div>
      </div>
      
      <!-- 右上角菜单 -->
      <div style="display:flex; gap:8px; align-items:center;">
        <!-- 多选模式下的操作按钮 -->
        <template v-if="isMultiSelectMode">
          <button @click="isAllSelected ? $emit('deselect-all') : $emit('select-all')" 
                  style="padding:8px 16px; font-size:13px; white-space:nowrap;"
                  @mouseenter="$event.target.style.backgroundColor='#f0f0f0'"
                  @mouseleave="$event.target.style.backgroundColor='#fff'">
            {{ isAllSelected ? '取消全选' : '全选' }}
          </button>
          <button @click="$emit('delete-selected')" 
                  :disabled="selectedGames.size === 0"
                  :style="{
                    padding: '8px 16px',
                    fontSize: '13px',
                    whiteSpace: 'nowrap',
                    color: selectedGames.size > 0 ? '#f44336' : '#999',
                    cursor: selectedGames.size > 0 ? 'pointer' : 'not-allowed',
                    opacity: selectedGames.size > 0 ? 1 : 0.5
                  }"
                  @mouseenter="selectedGames.size > 0 ? $event.target.style.backgroundColor='#ffebee' : null"
                  @mouseleave="$event.target.style.backgroundColor='#fff'">
            删除选中 {{ selectedGames.size > 0 ? `(${selectedGames.size})` : '' }}
          </button>
        </template>
        
        <!-- 多选按钮（固定位置） -->
        <button @click.stop="$emit('toggle-multi-select')" 
                :style="{
                  padding: '8px 16px',
                  fontSize: '13px',
                  whiteSpace: 'nowrap',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '4px',
                  background: isMultiSelectMode ? '#e8f5e9' : '#fff',
                  borderColor: isMultiSelectMode ? '#4CAF50' : '#ccc',
                  color: isMultiSelectMode ? '#2e7d32' : '#333'
                }"
                @mouseenter="$event.target.style.backgroundColor = isMultiSelectMode ? '#c8e6c9' : '#f0f0f0'"
                @mouseleave="$event.target.style.backgroundColor = isMultiSelectMode ? '#e8f5e9' : '#fff'">
          {{ isMultiSelectMode ? `多选 (${selectedGames.size})` : '多选' }}
        </button>
        
        <!-- 操作菜单 -->
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
          </div>
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

    <!-- 加载进度条 -->
    <div v-if="isLoadingGames || isFetchingImages" style="margin-bottom:16px; padding:12px; background:#f8f9fa; border-radius:4px; border:1px solid #e0e0e0;">
      <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:8px;">
        <span style="font-size:12px; color:#666;">
          {{ isLoadingGames ? '正在加载游戏...' : '正在获取游戏信息...' }}
        </span>
        <span style="font-size:12px; color:#666; font-weight:500;">
          {{ loadedGamesCount }} / {{ totalGamesCount }}
        </span>
      </div>
      <div style="width:100%; height:4px; background:#e0e0e0; border-radius:2px; overflow:hidden;">
        <div :style="{
               width: totalGamesCount > 0 ? `${(loadedGamesCount / totalGamesCount) * 100}%` : '0%',
               height: '100%',
               background: 'linear-gradient(90deg, #4CAF50, #45a049)',
               transition: 'width 0.3s ease'
             }"></div>
      </div>
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
                :image-src="getImageSrc(g.image)"
                :image-url="g.image_url || ''"
                @click="isMultiSelectMode ? $emit('toggle-selection', g) : $emit('open-detail', g)"
                @launch="$emit('launch', g)"
                @close="$emit('close-game', g.path)"
                @replace="$emit('open-replace', g)"
                @delete="$emit('delete-game', g)"
                @image-downloaded="$emit('image-downloaded', $event)" />
    </div>
  </div>
</template>

<script setup>
import { ref, inject, computed } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import GameCard from '../components/GameCard.vue';

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
  selectedTag: {
    type: String,
    default: null
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
  },
  isLoadingGames: {
    type: Boolean,
    default: false
  },
  isFetchingImages: {
    type: Boolean,
    default: false
  },
  loadedGamesCount: {
    type: Number,
    default: 0
  },
  totalGamesCount: {
    type: Number,
    default: 0
  },
  projectRoot: {
    type: String,
    default: ''
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
  'delete-game',
  'image-downloaded'
]);

const filters = ['全部', 'ADV', 'RPG'];
const showMenu = ref(false);
const showFilterMenu = ref(false);

// 判断是否已全选
const isAllSelected = computed(() => {
  if (props.filteredGames.length === 0) return false;
  return props.filteredGames.every(game => props.selectedGames.has(game.path));
});

// Helper function to get image source
function getImageSrc(imagePath) {
  if (!imagePath) return null;
  // 如果是 http/https URL，直接返回
  if (imagePath.startsWith('http://') || imagePath.startsWith('https://')) {
    return imagePath;
  }
  // 支持旧的 game_data/ 前缀，同时优先识别新的 kano_data/
  if (imagePath.startsWith('kano_data/') || imagePath.startsWith('game_data/')) {
    const fixed = imagePath.startsWith('game_data/') ? imagePath.replace(/^game_data\//, 'kano_data/') : imagePath;
    const absolutePath = `${props.projectRoot}\\${fixed.replace(/\//g, '\\\\')}`;
    return convertFileSrc(absolutePath);
  }
  // 其他情况（绝对路径）直接转换
  return convertFileSrc(imagePath);
}

</script>
