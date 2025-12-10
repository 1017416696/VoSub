<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { Search, ArrowDown, Switch } from '@element-plus/icons-vue'

const props = defineProps<{
  modelValue: string
  replaceValue: string
  matchCount: number
  showReplace: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'update:replaceValue', value: string): void
  (e: 'update:showReplace', value: boolean): void
  (e: 'replace-one'): void
  (e: 'replace-all'): void
  (e: 'close'): void
  (e: 'navigate', direction: 'up' | 'down'): void
}>()

const searchInputRef = ref<any>(null)
const replaceInputRef = ref<any>(null)

// 暴露给父组件
defineExpose({
  focus: () => {
    nextTick(() => {
      searchInputRef.value?.focus()
    })
  },
  blur: () => {
    searchInputRef.value?.blur()
    replaceInputRef.value?.blur()
  },
  getSearchInput: () => searchInputRef.value?.$el?.querySelector('input'),
  getReplaceInput: () => replaceInputRef.value?.$el?.querySelector('input')
})

const handleSearchKeydown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    e.preventDefault()
    searchInputRef.value?.blur()
    emit('navigate', e.key === 'ArrowDown' ? 'down' : 'up')
  } else if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
  } else if (e.key === 'Tab' && !e.shiftKey && props.showReplace) {
    // 按下 Tab 且替换框可见时，将焦点移到替换框
    e.preventDefault()
    nextTick(() => {
      replaceInputRef.value?.focus()
    })
  }
}

const handleReplaceKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
  } else if (e.key === 'Tab' && e.shiftKey) {
    // 按下 Shift+Tab 时，将焦点移回搜索框
    e.preventDefault()
    nextTick(() => {
      searchInputRef.value?.focus()
    })
  }
}
</script>

<template>
  <div class="search-replace-container">
    <!-- 搜索框 -->
    <div class="search-row">
      <el-input
        ref="searchInputRef"
        :model-value="modelValue"
        @update:model-value="emit('update:modelValue', $event)"
        @keydown="handleSearchKeydown"
        placeholder="搜索字幕"
        clearable
        class="search-input"
        size="small"
      >
        <template #prefix>
          <el-icon class="search-icon"><Search /></el-icon>
        </template>
      </el-input>
      <span v-if="modelValue && matchCount > 0" class="match-count">
        {{ matchCount }}
      </span>
      <button
        class="expand-btn"
        :class="{ expanded: showReplace }"
        @click="emit('update:showReplace', !showReplace)"
        :title="showReplace ? '收起替换' : '展开替换'"
      >
        <el-icon><ArrowDown /></el-icon>
      </button>
    </div>

    <!-- 替换框 -->
    <div v-if="showReplace" class="replace-row">
      <el-input
        ref="replaceInputRef"
        :model-value="replaceValue"
        @update:model-value="emit('update:replaceValue', $event)"
        @keydown="handleReplaceKeydown"
        placeholder="替换为..."
        clearable
        class="replace-input"
        size="small"
      >
        <template #prefix>
          <el-icon class="replace-icon"><Switch /></el-icon>
        </template>
      </el-input>
      <button
        class="replace-btn"
        @click="emit('replace-one')"
        :disabled="!modelValue || matchCount === 0"
        title="替换当前"
      >
        替换
      </button>
      <button
        class="replace-btn replace-all-btn"
        @click="emit('replace-all')"
        :disabled="!modelValue"
        title="全部替换"
      >
        全部
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 搜索和替换框 */
.search-replace-container {
  padding: 0.5rem;
  border-bottom: 1px solid #e5e7eb;
  background: white;
}

.search-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.search-row:last-of-type {
  margin-bottom: 0;
}

/* 搜索图标 */
.search-icon,
.replace-icon {
  color: #9ca3af;
  font-size: 14px;
}

/* 展开/收起按钮 */
.expand-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  border: 1px solid #e5e7eb;
  background: #f9fafb;
  color: #6b7280;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 0.375rem;
  transition: all 0.2s ease;
}

.expand-btn:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
  color: #374151;
}

.expand-btn .el-icon {
  font-size: 14px;
  transition: transform 0.2s ease;
}

.expand-btn.expanded {
  background: #eff6ff;
  border-color: #3b82f6;
  color: #3b82f6;
}

.expand-btn.expanded .el-icon {
  transform: rotate(180deg);
}

.search-input {
  flex: 1;
  min-width: 0;
}

.search-input :deep(.el-input__wrapper) {
  padding: 0.4rem 0.6rem;
  background: white;
  border: 1px solid #dcdfe6;
  border-radius: 0.3rem;
  height: 2rem;
}

.search-input :deep(.el-input__wrapper:hover) {
  border-color: #b3d8ff;
  background: white;
}

.search-input :deep(.el-input__wrapper.is-focus) {
  border-color: #409eff;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.2);
}

.search-input :deep(.el-input__input) {
  font-size: 0.875rem;
}

.match-count {
  padding: 0.25rem 0.5rem;
  background: #f0f0f0;
  border-radius: 0.25rem;
  font-size: 0.75rem;
  color: #666;
  white-space: nowrap;
  flex-shrink: 0;
}

.replace-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.replace-input {
  flex: 1;
  min-width: 0;
}

.replace-input :deep(.el-input__wrapper) {
  padding: 0.4rem 0.6rem;
  background: white;
  border: 1px solid #dcdfe6;
  border-radius: 0.3rem;
  height: 2rem;
}

.replace-input :deep(.el-input__wrapper:hover) {
  border-color: #b3d8ff;
  background: white;
}

.replace-input :deep(.el-input__wrapper.is-focus) {
  border-color: #409eff;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.2);
}

.replace-input :deep(.el-input__input) {
  font-size: 0.875rem;
}

.replace-btn {
  padding: 0.4rem 0.8rem;
  background: white;
  border: 1px solid #dcdfe6;
  border-radius: 0.3rem;
  color: #606266;
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s;
  white-space: nowrap;
  flex-shrink: 0;
}

.replace-btn:hover:not(:disabled) {
  border-color: #409eff;
  color: #409eff;
}

.replace-btn:disabled {
  color: #ccc;
  cursor: not-allowed;
}

.replace-all-btn:not(:disabled) {
  background: #409eff;
  border-color: #409eff;
  color: white;
}

.replace-all-btn:hover:not(:disabled) {
  background: #66b1ff;
  border-color: #66b1ff;
}
</style>
