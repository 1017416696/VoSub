<script setup lang="ts">
import { ref, watch, nextTick, onBeforeUnmount } from 'vue'
import { ElMessage } from 'element-plus'
import { useSmartDictionaryStore } from '@/stores/smartDictionary'

const props = defineProps<{
  visible: boolean
  initialVariant?: string // 可选的初始错误变体（从选中文本传入）
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
}>()

const smartDictionary = useSmartDictionaryStore()

const newWordCorrect = ref('')
const newWordVariant = ref('')
const correctInputRef = ref<HTMLInputElement | null>(null)

// 监听弹窗显示，自动填充选中的文本
watch(() => props.visible, (visible) => {
  if (visible) {
    // 如果有初始变体，填充到错误变体输入框
    if (props.initialVariant) {
      newWordVariant.value = props.initialVariant
    }
    // 自动聚焦到正确写法输入框
    nextTick(() => {
      correctInputRef.value?.focus()
    })
    document.addEventListener('keydown', handleKeydown, true)
  } else {
    // 关闭时清空输入
    newWordCorrect.value = ''
    newWordVariant.value = ''
    document.removeEventListener('keydown', handleKeydown, true)
  }
})

// 处理 ESC 键关闭
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.visible) {
    e.preventDefault()
    e.stopPropagation()
    handleClose()
  }
}

// 添加词条
const handleAdd = () => {
  if (!newWordCorrect.value.trim()) {
    ElMessage.warning('请输入正确写法')
    return
  }

  // 支持从变体输入框直接添加（逗号分隔）
  const variantText = newWordVariant.value.trim()
  const variants = variantText
    ? variantText.split(/[,，]/).map(v => v.trim()).filter(v => v)
    : []

  const result = smartDictionary.addManual(newWordCorrect.value.trim(), variants)

  if (result) {
    if (result.merged) {
      if (result.newVariants.length > 0) {
        ElMessage.success(`已合并，新增 ${result.newVariants.length} 个变体`)
      } else {
        ElMessage.info('词条已存在')
      }
    } else {
      ElMessage.success('添加成功')
    }
  }
  handleClose()
}

// 关闭弹窗
const handleClose = () => {
  emit('update:visible', false)
}

// 组件卸载时清理
onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown, true)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="quick-add-overlay" @click.self="handleClose">
        <div class="quick-add-dialog">
          <div class="quick-add-header">
            <h3>快速添加词条</h3>
            <button class="quick-add-close" @click="handleClose">×</button>
          </div>
          <div class="quick-add-body">
            <div class="quick-add-field">
              <label>正确写法 <span class="field-required">*</span></label>
              <input
                ref="correctInputRef"
                v-model="newWordCorrect"
                type="text"
                placeholder="输入正确的词语，如 Kubernetes"
                @keyup.enter="handleAdd"
              />
            </div>
            <div class="quick-add-field">
              <label>错误变体 <span class="field-hint">可选</span></label>
              <input
                v-model="newWordVariant"
                type="text"
                placeholder="语音识别可能出现的错误写法，多个用逗号分隔"
                @keyup.enter="handleAdd"
              />
              <p class="field-desc">例如：酷伯内特斯, K8S, 库伯内特斯</p>
            </div>
          </div>
          <div class="quick-add-footer">
            <button class="quick-add-cancel" @click="handleClose">取消</button>
            <button
              class="quick-add-confirm"
              :disabled="!newWordCorrect.trim()"
              @click="handleAdd"
            >
              添加
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.quick-add-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
}

.quick-add-dialog {
  width: 420px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.quick-add-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.quick-add-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
}

.quick-add-close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  color: #9ca3af;
  background: none;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.quick-add-close:hover {
  background: #f3f4f6;
  color: #6b7280;
}

.quick-add-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.quick-add-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.quick-add-field label {
  font-size: 13px;
  font-weight: 500;
  color: #374151;
}

.field-required {
  color: #ef4444;
}

.field-hint {
  font-weight: 400;
  color: #9ca3af;
}

.quick-add-field input {
  height: 40px;
  padding: 0 12px;
  font-size: 14px;
  color: #374151;
  background: #fff;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  outline: none;
  transition: all 0.15s;
}

.quick-add-field input:focus {
  border-color: #10b981;
  box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.1);
}

.quick-add-field input::placeholder {
  color: #9ca3af;
}

.field-desc {
  margin: 0;
  font-size: 12px;
  color: #9ca3af;
}

.quick-add-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 20px;
  background: #f9fafb;
  border-top: 1px solid #e5e7eb;
}

.quick-add-cancel {
  padding: 8px 16px;
  font-size: 14px;
  color: #6b7280;
  background: white;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.quick-add-cancel:hover {
  background: #f3f4f6;
}

.quick-add-confirm {
  padding: 8px 20px;
  font-size: 14px;
  font-weight: 500;
  color: white;
  background: #10b981;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.quick-add-confirm:hover:not(:disabled) {
  background: #059669;
}

.quick-add-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-active .quick-add-dialog,
.fade-leave-active .quick-add-dialog {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-from .quick-add-dialog,
.fade-leave-to .quick-add-dialog {
  transform: scale(0.95);
  opacity: 0;
}
</style>
