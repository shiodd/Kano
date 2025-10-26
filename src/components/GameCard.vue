<template>
  <div :style="{
         width: '140px',
         border: isSelected ? '2px solid #666' : '1px solid #e0e0e0',
         borderRadius: '4px',
         overflow: 'hidden',
         transition: 'all 0.2s',
         background: isSelected ? '#f0f0f0' : '#fafafa',
         userSelect: 'none',
         cursor: 'pointer'
       }"
       @click.stop="$emit('click')"
       @mouseenter="!isSelected && ($event.currentTarget.style.backgroundColor='#eeeeee', $event.currentTarget.style.borderColor='#bbb')"
       @mouseleave="!isSelected && ($event.currentTarget.style.backgroundColor='#fafafa', $event.currentTarget.style.borderColor='#e0e0e0')">
    <!-- 封面图片 -->
    <div style="width:140px; height:196px; position:relative; background:#f0f0f0; overflow:hidden;">
      <img v-if="currentImageSrc" 
           :src="currentImageSrc" 
           @error="handleImageError"
           style="position:absolute; top:0; left:0; width:100%; height:100%; object-fit:cover;" />
      
      <!-- 运行中状态指示器 -->
      <div v-if="isRunning" 
           :style="{
             position: 'absolute',
             top: '8px',
             left: '8px',
             padding: '4px 8px',
             borderRadius: '3px',
             background: 'rgba(76, 175, 80, 0.9)',
             color: '#fff',
             fontSize: '11px',
             fontWeight: '500',
             boxShadow: '0 2px 4px rgba(0,0,0,0.2)'
           }">
        运行中
      </div>
      
      <!-- 多选模式选中标记 -->
      <div v-if="isMultiSelectMode" 
           :style="{
             position: 'absolute',
             top: '8px',
             right: '8px',
             width: '20px',
             height: '20px',
             borderRadius: '3px',
             background: isSelected ? '#666' : '#fff',
             border: '2px solid #666',
             display: 'flex',
             alignItems: 'center',
             justifyContent: 'center',
             fontSize: '12px',
             transition: 'all 0.2s'
           }">
        <span v-if="isSelected" style="color:#fff; font-weight:bold;">✓</span>
      </div>
    </div>
    
    <!-- 游戏信息 -->
    <div style="padding:8px;">
      <div style="font-size:12px; font-weight:500; margin-bottom:2px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:#333;" :title="game.name">
        {{ game.name }}
      </div>
      <div style="font-size:10px; color:#999; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; margin-bottom:6px;" :title="game.path">
        {{ game.path.split(/[\\/]/).pop() }}
      </div>
      
      <!-- 操作按钮 -->
      <div v-if="!isMultiSelectMode" style="display:flex; gap:4px;" @click.stop>
        <!-- 运行中时显示关闭按钮（红色），否则显示启动按钮（绿色） -->
        <button v-if="isRunning" 
                @click="$emit('close')" 
                style="flex:1; padding:5px; font-size:11px; background:#f44336; border:1px solid #f44336; color:#fff; transition:all 0.2s;"
                @mouseenter="$event.target.style.backgroundColor='#d32f2f'; $event.target.style.borderColor='#d32f2f'"
                @mouseleave="$event.target.style.backgroundColor='#f44336'; $event.target.style.borderColor='#f44336'">关闭</button>
        <button v-else 
                @click="$emit('launch')" 
                style="flex:1; padding:5px; font-size:11px; background:#4CAF50; border:1px solid #4CAF50; color:#fff; transition:all 0.2s;"
                @mouseenter="$event.target.style.backgroundColor='#45a049'; $event.target.style.borderColor='#45a049'"
                @mouseleave="$event.target.style.backgroundColor='#4CAF50'; $event.target.style.borderColor='#4CAF50'">启动</button>
        <button @click.stop="!isRunning && $emit('replace')" 
                :disabled="isRunning"
                :style="{
                  padding: '5px 8px',
                  fontSize: '11px',
                  background: isRunning ? '#f5f5f5' : '#fff',
                  border: '1px solid #ddd',
                  color: isRunning ? '#ccc' : '#666',
                  cursor: isRunning ? 'not-allowed' : 'pointer',
                  transition: 'all 0.2s'
                }"
                @mouseenter="!isRunning && ($event.target.style.backgroundColor='#f0f0f0', $event.target.style.borderColor='#999')"
                @mouseleave="!isRunning && ($event.target.style.backgroundColor='#fff', $event.target.style.borderColor='#ddd')">替换</button>
        <button @click="!isRunning && $emit('delete')"
                :disabled="isRunning"
                :style="{
                  padding: '5px 8px',
                  fontSize: '11px',
                  background: isRunning ? '#f5f5f5' : '#fff',
                  border: '1px solid #ddd',
                  color: isRunning ? '#ccc' : '#999',
                  cursor: isRunning ? 'not-allowed' : 'pointer',
                  transition: 'all 0.2s'
                }"
                @mouseenter="!isRunning && ($event.target.style.backgroundColor='#f0f0f0', $event.target.style.borderColor='#999')"
                @mouseleave="!isRunning && ($event.target.style.backgroundColor='#fff', $event.target.style.borderColor='#ddd')">删除</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  game: {
    type: Object,
    required: true
  },
  isRunning: {
    type: Boolean,
    default: false
  },
  isSelected: {
    type: Boolean,
    default: false
  },
  isMultiSelectMode: {
    type: Boolean,
    default: false
  },
  imageSrc: {
    type: String,
    default: ''
  },
  imageUrl: {
    type: String,
    default: ''
  }
});

const emit = defineEmits(['click', 'launch', 'close', 'replace', 'delete', 'image-downloaded']);

const currentImageSrc = ref(props.imageSrc);
const isDownloading = ref(false);

// 监听 imageSrc 变化
watch(() => props.imageSrc, (newSrc) => {
  currentImageSrc.value = newSrc;
});

// 处理图片加载失败
async function handleImageError() {
  // 如果已经在下载中，不重复下载
  if (isDownloading.value) return;
  
  // 如果有网络图片地址且有 subject_id，尝试下载
  if (props.imageUrl && props.game.subject_id) {
    isDownloading.value = true;
    try {
      console.log('本地图片加载失败，尝试从网络下载...', props.imageUrl);
      
      // 下载图片
      const localImagePath = await invoke('download_image', {
        url: props.imageUrl,
        subjectId: props.game.subject_id
      });
      
      if (localImagePath) {
        // 更新数据库中的本地路径
        await invoke('update_game_info', {
          path: props.game.path,
          name: null,
          image: localImagePath,
          imageUrl: null,
          subjectId: null
        });
        
        // 通知父组件更新
        emit('image-downloaded', { path: props.game.path, localPath: localImagePath });
        
        // 更新当前显示的图片（需要重新生成 Tauri 资源 URL）
        // 这里我们依赖父组件重新传入正确的 imageSrc
      }
    } catch (err) {
      console.error('从网络下载图片失败:', err);
      // 如果下载失败，尝试直接显示网络图片
      if (props.imageUrl) {
        currentImageSrc.value = props.imageUrl;
      }
    } finally {
      isDownloading.value = false;
    }
  } else if (props.imageUrl) {
    // 如果没有 subject_id 但有网络地址，直接使用网络地址
    currentImageSrc.value = props.imageUrl;
  }
}
</script>
