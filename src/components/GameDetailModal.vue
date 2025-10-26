<template>
  <div v-if="visible && detailData" style="position:fixed; inset:0; display:flex; align-items:center; justify-content:center; background:rgba(0,0,0,0.3); z-index:90; padding:20px;">
    <div style="width:90%; max-width:900px; max-height:90%; overflow:auto; background:#fff; border-radius:4px; border:1px solid #ddd; display:flex; flex-direction:column;">
      <!-- 头部 -->
      <div style="padding:20px; border-bottom:1px solid #e0e0e0; display:flex; justify-content:space-between; align-items:center;">
        <h3 style="margin:0; font-size:18px; font-weight:500; color:#333;">游戏详情</h3>
        <button @click="$emit('close')" 
                style="padding:6px 12px; font-size:13px; background:#fff; border:1px solid #ddd; color:#666; transition:all 0.2s;"
                @mouseenter="$event.target.style.backgroundColor='#e8e8e8'; $event.target.style.borderColor='#999'"
                @mouseleave="$event.target.style.backgroundColor='#fff'; $event.target.style.borderColor='#ddd'">关闭</button>
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
            
            <!-- 标签 -->
            <div v-if="detailData.meta_tags && detailData.meta_tags.length > 0" style="margin-bottom:16px;">
              <div style="font-size:13px; font-weight:500; color:#666; margin-bottom:8px;">标签</div>
              <div style="display:flex; gap:6px; flex-wrap:wrap;">
                <span v-for="tag in detailData.meta_tags" :key="tag" 
                      style="padding:4px 10px; background:#f0f0f0; border-radius:3px; font-size:11px; color:#666; border:1px solid #e0e0e0;">
                  {{ tag }}
                </span>
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
    </div>
  </div>
</template>

<script setup>
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

defineEmits(['close', 'launch-game', 'close-game', 'delete-game']);

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
</script>
