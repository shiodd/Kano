<template>
  <aside style="width:180px; background:#f8f9fa; border-right:1px solid #e0e0e0; padding:0; position:fixed; left:0; top:0; bottom:0; display:flex; flex-direction:column;">
    <div style="padding:20px 16px; border-bottom:1px solid #e0e0e0;">
      <h2 style="margin:0; font-size:16px; font-weight:600; color:#333;">kano</h2>
    </div>
    <nav style="flex:1; padding:12px 0; display:flex; flex-direction:column; gap:2px;">
      <button :class="{active: modelValue === 'library'}" @click="$emit('update:modelValue', 'library')" 
              style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent;">
        游戏库
      </button>
      <button :class="{active: modelValue === 'settings'}" @click="$emit('update:modelValue', 'settings')" 
              style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent;">
        设置
      </button>
      <button :class="{active: modelValue === 'about'}" @click="$emit('update:modelValue', 'about')" 
              style="text-align:left; padding:10px 16px; font-size:14px; background:transparent; border:none; color:#666; cursor:pointer; transition:all 0.2s; border-left:2px solid transparent;">
        关于
      </button>
    </nav>
  </aside>
</template>

<script setup>
defineProps({
  modelValue: {
    type: String,
    required: true
  }
});

defineEmits(['update:modelValue']);
</script>

<style scoped>
/* 侧边栏按钮样式 */
button.active {
  background: #e8e8e8 !important;
  color: #333 !important;
  font-weight: 500;
  border-left: 2px solid #666 !important;
}

button:hover {
  background: #f0f0f0;
  color: #333;
}
</style>
