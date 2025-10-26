<template>
  <div v-if="visible" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.3); z-index:80;">
    <div style="width:70%; max-width:800px; max-height:80%; overflow:auto; background:#fff; padding:20px; border-radius:4px; border:1px solid #ddd;">
      <h3 style="margin:0 0 16px 0; font-size:16px; font-weight:500; color:#333;">搜索并替换: {{ targetGame?.name }}</h3>
      
      <!-- 更换 EXE 文件区域 -->
      <div style="margin-bottom:16px; padding:12px; background:#f8f9fa; border-radius:4px; border:1px solid #e0e0e0;">
        <div style="display:flex; gap:8px; align-items:center; margin-bottom:8px;">
          <button @click="$emit('replace-exe')" style="padding:6px 12px; font-size:12px; white-space:nowrap;">更换 EXE 文件</button>
          <div style="flex:1; font-size:11px; color:#999;">更换游戏的可执行文件路径</div>
        </div>
        <div v-if="targetGame" style="font-size:11px; color:#666; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; padding:4px 8px; background:#fff; border-radius:3px; border:1px solid #e0e0e0;" :title="targetGame.path">
          当前: {{ targetGame.path }}
        </div>
      </div>
      
      <!-- 分割线 -->
      <div style="height:1px; background:#e0e0e0; margin:16px 0;"></div>
      
      <!-- 搜索替换区域 -->
      <div style="display:flex; gap:8px; align-items:center; margin-bottom:16px;">
        <input v-model="searchKeyword" placeholder="输入关键词搜索" style="flex:1; padding:8px 12px; font-size:13px;" @keyup.enter="$emit('search', searchKeyword)" />
        <button @click="$emit('search', searchKeyword)" style="padding:8px 16px; font-size:13px;">搜索</button>
        <button @click="$emit('close')" style="padding:8px 16px; font-size:13px; background:#999;">关闭</button>
      </div>
      
      <div v-if="loading" style="text-align:center; padding:30px; color:#999; font-size:13px;">搜索中...</div>
      <div v-else-if="results.length === 0 && searchKeyword" style="text-align:center; padding:30px; color:#ccc; font-size:13px;">
        没有找到相关结果
      </div>
      <div v-else style="display:grid; grid-template-columns: repeat(auto-fill, 140px); gap:12px;">
        <div v-for="(it, i) in results" :key="it.id || i" 
             style="width:140px; border:1px solid #e0e0e0; border-radius:4px; overflow:hidden; cursor:pointer; transition:all 0.2s; background:#fafafa;"
             @click="$emit('select', it)"
             @mouseenter="$event.currentTarget.style.backgroundColor='#f5f5f5'; $event.currentTarget.style.borderColor='#ccc'"
             @mouseleave="$event.currentTarget.style.backgroundColor='#fafafa'; $event.currentTarget.style.borderColor='#e0e0e0'">
          <div style="width:140px; height:196px; position:relative; background:#f0f0f0; overflow:hidden;">
            <img v-if="it.image || it.subject?.image || it.images?.large" 
                 :src="it.image || it.subject?.image || it.images?.large || it.images?.medium || it.images?.small" 
                 style="position:absolute; top:0; left:0; width:100%; height:100%; object-fit:cover;" />
            <div v-else style="position:absolute; top:0; left:0; width:100%; height:100%; display:flex; align-items:center; justify-content:center; color:#ccc; font-size:32px;">?</div>
          </div>
          <div style="padding:10px;">
            <div style="font-weight:500; font-size:12px; margin-bottom:4px; overflow:hidden; text-overflow:ellipsis; display:-webkit-box; line-clamp:2; -webkit-line-clamp:2; -webkit-box-orient:vertical; color:#333;">
              {{ it.name_cn || it.name || it.title || it.subject?.name_cn || it.subject?.name || it.subject?.title }}
            </div>
            <div style="font-size:11px; color:#999;">ID: {{ it.id || it.subject?.id || 'N/A' }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';

const props = defineProps({
  visible: {
    type: Boolean,
    required: true
  },
  targetGame: {
    type: Object,
    default: null
  },
  results: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  }
});

defineEmits(['close', 'search', 'select', 'replace-exe']);

const searchKeyword = ref('');

// 当模态框打开时，自动填充搜索关键词
watch(() => props.visible, (newVal) => {
  if (newVal && props.targetGame) {
    searchKeyword.value = props.targetGame.name;
  }
});
</script>
