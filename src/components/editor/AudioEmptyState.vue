<script setup lang="ts">
import { Plus } from '@element-plus/icons-vue'

const emit = defineEmits<{
  (e: 'open-audio'): void
}>()
</script>

<template>
  <div class="timeline-placeholder">
    <div class="audio-empty-state">
      <div class="audio-empty-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M9 18V5l12-2v13" stroke-linecap="round" stroke-linejoin="round"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
      </div>
      <div class="audio-empty-text">
        <span class="audio-empty-title">加载音频以启用波形预览</span>
        <span class="audio-empty-hint">支持 MP3、WAV、OGG、FLAC、M4A、AAC 格式</span>
      </div>
      <el-button type="primary" size="small" @click="emit('open-audio')" class="audio-load-btn">
        <el-icon style="margin-right: 4px;"><Plus /></el-icon>
        选择音频文件
      </el-button>
    </div>
  </div>
</template>

<style scoped>
.timeline-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-bottom: 1px solid #e2e8f0;
}

.audio-empty-state {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding: 0.625rem 1.25rem;
  background: white;
  border-radius: 10px;
  border: 1px dashed #cbd5e1;
  transition: all 0.2s ease;
}

.audio-empty-state:hover {
  border-color: #94a3b8;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.audio-empty-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);
  border-radius: 10px;
  color: #3b82f6;
  flex-shrink: 0;
}

.audio-empty-icon svg {
  width: 24px;
  height: 24px;
}

.audio-empty-text {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.audio-empty-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: #1e293b;
}

.audio-empty-hint {
  font-size: 0.6875rem;
  color: #94a3b8;
}

.audio-load-btn {
  padding: 0.375rem 0.875rem;
  border-radius: 8px;
  font-weight: 500;
  font-size: 0.8125rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
</style>
