<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import type { SubtitleEntry, TimeStamp } from '@/types/subtitle'
import { computeDiff, segmentsToDiffGroups, buildFinalText, type DiffGroup } from '@/utils/textDiff'

const props = defineProps<{
  entry: SubtitleEntry | null
  formatTimeStamp: (ts: TimeStamp) => string
  fireredReady?: boolean
  isCorrecting?: boolean
  correctionResult?: { original: string; corrected: string; has_diff: boolean } | null
  needsCorrectionCount?: number
}>()

const emit = defineEmits<{
  (e: 'update-text', text: string): void
  (e: 'update-time', type: 'start' | 'end', value: string): void
  (e: 'adjust-time', type: 'start' | 'end', deltaMs: number): void
  (e: 'delete-entry'): void
  (e: 'remove-html'): void
  (e: 'add-cjk-spaces'): void
  (e: 'remove-punctuation'): void
  (e: 'to-uppercase'): void
  (e: 'to-lowercase'): void
  (e: 'text-focus'): void
  (e: 'text-blur'): void
  (e: 'text-input'): void
  (e: 'correct-entry'): void
  (e: 'apply-correction', text?: string): void
  (e: 'dismiss-correction'): void
  (e: 'toggle-correction-mark'): void
  (e: 'apply-suggestion', text?: string): void
  (e: 'dismiss-suggestion'): void
  (e: 'quick-add-dictionary', selectedText: string): void
}>()

const editingText = ref('')
const editingStartTime = ref('')
const editingEndTime = ref('')
const textareaInputRef = ref<any>(null)

// 细粒度编辑状态
const correctionDiffGroups = ref<DiffGroup[]>([])
const suggestionDiffGroups = ref<DiffGroup[]>([])

// 监听选中字幕变化，更新编辑文本和时间
watch(() => props.entry, (entry) => {
  if (entry) {
    editingText.value = entry.text
    editingStartTime.value = props.formatTimeStamp(entry.startTime)
    editingEndTime.value = props.formatTimeStamp(entry.endTime)
  }
  // 重置细粒度编辑状态
  correctionDiffGroups.value = []
  suggestionDiffGroups.value = []
}, { immediate: true, deep: true })

// 监听校正结果变化，计算差异组并更新输入框
watch(() => props.correctionResult, (result) => {
  if (result && result.has_diff) {
    const segments = computeDiff(result.original, result.corrected)
    correctionDiffGroups.value = segmentsToDiffGroups(segments)
    // 默认全部采用校正，更新输入框显示校正后的结果
    editingText.value = result.corrected
  } else {
    correctionDiffGroups.value = []
  }
}, { immediate: true })

// 监听建议变化，计算差异组并更新输入框
watch(() => props.entry?.correctionSuggestion, (suggestion) => {
  if (suggestion && props.entry) {
    const segments = computeDiff(props.entry.text, suggestion)
    suggestionDiffGroups.value = segmentsToDiffGroups(segments)
    // 默认全部采用校正，更新输入框显示校正后的结果
    editingText.value = suggestion
  } else {
    suggestionDiffGroups.value = []
  }
}, { immediate: true })

// 计算时长显示
const durationDisplay = computed(() => {
  if (!props.entry) return '00:00,000'
  
  const startMs = props.entry.startTime.hours * 3600000 +
    props.entry.startTime.minutes * 60000 +
    props.entry.startTime.seconds * 1000 +
    props.entry.startTime.milliseconds
  
  const endMs = props.entry.endTime.hours * 3600000 +
    props.entry.endTime.minutes * 60000 +
    props.entry.endTime.seconds * 1000 +
    props.entry.endTime.milliseconds
  
  const durationMs = Math.max(0, endMs - startMs)
  const seconds = Math.floor(durationMs / 1000)
  const ms = durationMs % 1000
  
  return `00:${String(seconds).padStart(2, '0')},${String(ms).padStart(3, '0')}`
})

const handleTextInput = () => {
  emit('update-text', editingText.value)
  emit('text-input')
}

const handleTimeChange = (type: 'start' | 'end') => {
  const value = type === 'start' ? editingStartTime.value : editingEndTime.value
  emit('update-time', type, value)
}

// 聚焦到文本输入框
const focusTextarea = async () => {
  await nextTick()
  if (textareaInputRef.value) {
    textareaInputRef.value.focus()
    await nextTick()
    // el-input 组件的内部 input 元素
    const inputEl = textareaInputRef.value.input as HTMLInputElement
    if (inputEl) {
      const textLength = inputEl.value.length
      inputEl.setSelectionRange(textLength, textLength)
    }
  }
}

// 切换差异组选择（校正结果）并同步更新输入框
function toggleCorrectionGroup(groupId: number) {
  const group = correctionDiffGroups.value.find(g => g.id === groupId)
  if (group && group.type === 'change') {
    group.useNew = !group.useNew
    // 同步更新输入框
    editingText.value = getCorrectionFinalText()
  }
}

// 切换差异组选择（建议）并同步更新输入框
function toggleSuggestionGroup(groupId: number) {
  const group = suggestionDiffGroups.value.find(g => g.id === groupId)
  if (group && group.type === 'change') {
    group.useNew = !group.useNew
    // 同步更新输入框
    editingText.value = getSuggestionFinalText()
  }
}

// 获取校正结果的最终文本
function getCorrectionFinalText(): string {
  return buildFinalText(correctionDiffGroups.value)
}

// 获取建议的最终文本
function getSuggestionFinalText(): string {
  return buildFinalText(suggestionDiffGroups.value)
}

// 快速全选原文（校正结果）
function useAllOriginalCorrection() {
  correctionDiffGroups.value.forEach(g => {
    if (g.type === 'change') g.useNew = false
  })
  editingText.value = getCorrectionFinalText()
}

// 快速全选校正（校正结果）
function useAllCorrectedCorrection() {
  correctionDiffGroups.value.forEach(g => {
    if (g.type === 'change') g.useNew = true
  })
  editingText.value = getCorrectionFinalText()
}

// 快速全选原文（建议）
function useAllOriginalSuggestion() {
  suggestionDiffGroups.value.forEach(g => {
    if (g.type === 'change') g.useNew = false
  })
  editingText.value = getSuggestionFinalText()
}

// 快速全选校正（建议）
function useAllCorrectedSuggestion() {
  suggestionDiffGroups.value.forEach(g => {
    if (g.type === 'change') g.useNew = true
  })
  editingText.value = getSuggestionFinalText()
}

// 应用校正（使用输入框的内容，因为用户可能手动修改了）
function handleApplyCorrection() {
  emit('apply-correction', editingText.value)
}

// 应用建议（使用输入框的内容，因为用户可能手动修改了）
function handleApplySuggestion() {
  emit('apply-suggestion', editingText.value)
}

// 获取输入框中选中的文本
function getSelectedText(): string {
  const inputEl = textareaInputRef.value?.$el?.querySelector('input') as HTMLInputElement | null
  if (inputEl && inputEl.selectionStart !== inputEl.selectionEnd) {
    return inputEl.value.substring(inputEl.selectionStart ?? 0, inputEl.selectionEnd ?? 0)
  }
  return ''
}

// 快速添加词典
function handleQuickAddDictionary() {
  const selectedText = getSelectedText()
  emit('quick-add-dictionary', selectedText)
}

// 暴露给父组件
defineExpose({
  focusTextarea,
  getTextareaRef: () => textareaInputRef.value,
  getSelectedText
})
</script>

<template>
  <!-- 字幕编辑区 -->
  <div v-if="entry" class="subtitle-edit-section">
    <!-- 编辑头部 -->
    <div class="edit-header">
      <div class="edit-header-left">
        <span class="edit-badge">#{{ entry.id }}</span>
        <h3 class="edit-title">编辑字幕</h3>
        <!-- 需要校正标记 -->
        <button 
          class="correction-mark-btn"
          :class="{ marked: entry.needsCorrection }"
          @click="emit('toggle-correction-mark')"
          :title="entry.needsCorrection ? '取消校正标记' : '标记为需要校正'"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
            <line x1="12" y1="9" x2="12" y2="13"/>
            <line x1="12" y1="17" x2="12.01" y2="17"/>
          </svg>
          <span>{{ entry.needsCorrection ? '已标记' : '标记校正' }}</span>
        </button>
      </div>
      <button class="delete-entry-btn" @click="emit('delete-entry')" title="删除此字幕">
        <el-icon><Delete /></el-icon>
      </button>
    </div>

    <!-- 时间设置 - 紧凑单行布局 -->
    <div class="time-row">
      <!-- 开始时间 -->
      <div class="time-block">
        <span class="time-label">开始</span>
        <div class="time-control">
          <button class="time-btn-sm" @click="emit('adjust-time', 'start', -100)" title="-100ms">−</button>
          <el-input
            v-model="editingStartTime"
            class="time-input-sm"
            size="small"
            @blur="() => handleTimeChange('start')"
            @keyup.enter="() => handleTimeChange('start')"
          />
          <button class="time-btn-sm" @click="emit('adjust-time', 'start', 100)" title="+100ms">+</button>
        </div>
      </div>

      <span class="time-separator">→</span>

      <!-- 结束时间 -->
      <div class="time-block">
        <span class="time-label">结束</span>
        <div class="time-control">
          <button class="time-btn-sm" @click="emit('adjust-time', 'end', -100)" title="-100ms">−</button>
          <el-input
            v-model="editingEndTime"
            class="time-input-sm"
            size="small"
            @blur="() => handleTimeChange('end')"
            @keyup.enter="() => handleTimeChange('end')"
          />
          <button class="time-btn-sm" @click="emit('adjust-time', 'end', 100)" title="+100ms">+</button>
        </div>
      </div>

      <!-- 时长 -->
      <div class="time-block duration-block">
        <span class="time-label">时长</span>
        <span class="duration-value-sm">{{ durationDisplay }}</span>
      </div>
    </div>

    <!-- 文本编辑卡片 -->
    <div class="text-edit-card" :class="{ 'has-correction': entry?.correctionSuggestion || (correctionResult && correctionResult.has_diff) }">
      <div class="text-card-header">
        <div class="text-header-left">
          <svg class="text-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14,2 14,8 20,8"/>
            <line x1="16" y1="13" x2="8" y2="13"/>
            <line x1="16" y1="17" x2="8" y2="17"/>
          </svg>
          <span class="text-card-title">
            {{ (entry?.correctionSuggestion || (correctionResult && correctionResult.has_diff)) ? '校正结果' : '字幕内容' }}
          </span>
          <span v-if="entry?.correctionSuggestion || (correctionResult && correctionResult.has_diff)" class="result-hint">
            可直接编辑
          </span>
        </div>
        <span class="char-count">{{ editingText.length }} 字符</span>
      </div>
      <div class="text-input-wrapper">
        <el-input
          ref="textareaInputRef"
          v-model="editingText"
          placeholder="在此输入字幕文本..."
          @focus="emit('text-focus')"
          @blur="emit('text-blur')"
          @input="handleTextInput"
          class="text-input-new"
        />
      </div>
    </div>

    <!-- 批量校正建议（存储在 entry 中的） -->
    <div v-if="entry?.correctionSuggestion" class="correction-result suggestion-result">
      <div class="correction-header">
        <div class="correction-header-left">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
          <span>校正建议</span>
          <span class="diff-hint">点击高亮切换</span>
        </div>
        <div class="header-quick-actions">
          <button class="quick-select-btn" @click="useAllOriginalSuggestion" title="全部使用原文">原文</button>
          <button class="quick-select-btn" @click="useAllCorrectedSuggestion" title="全部使用校正">校正</button>
        </div>
      </div>
      
      <!-- 双行对比视图 -->
      <div class="diff-compare">
        <div class="diff-row original">
          <span class="diff-label">原</span>
          <span class="diff-text">
            <template v-for="group in suggestionDiffGroups" :key="'orig-' + group.id">
              <span v-if="group.type === 'equal'">{{ group.original }}</span>
              <span
                v-else-if="group.original"
                class="diff-chunk delete"
                :class="{ selected: !group.useNew }"
                @click="toggleSuggestionGroup(group.id)"
                title="点击切换"
              >{{ group.original }}</span>
            </template>
          </span>
        </div>
        <div class="diff-row corrected">
          <span class="diff-label">新</span>
          <span class="diff-text">
            <template v-for="group in suggestionDiffGroups" :key="'corr-' + group.id">
              <span v-if="group.type === 'equal'">{{ group.corrected }}</span>
              <span
                v-else-if="group.corrected"
                class="diff-chunk insert"
                :class="{ selected: group.useNew }"
                @click="toggleSuggestionGroup(group.id)"
                title="点击切换"
              >{{ group.corrected }}</span>
            </template>
          </span>
        </div>
      </div>
      
      <div class="correction-actions">
        <button class="correction-btn dismiss" @click="emit('dismiss-suggestion')">忽略</button>
        <button class="correction-btn apply" @click="handleApplySuggestion">采用</button>
      </div>
    </div>

    <!-- 单条校正结果对比（内联显示） -->
    <div v-else-if="correctionResult && correctionResult.has_diff" class="correction-result">
      <div class="correction-header">
        <div class="correction-header-left">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
          <span>校正建议</span>
          <span class="diff-hint">点击高亮切换</span>
        </div>
        <div class="header-quick-actions">
          <button class="quick-select-btn" @click="useAllOriginalCorrection" title="全部使用原文">原文</button>
          <button class="quick-select-btn" @click="useAllCorrectedCorrection" title="全部使用校正">校正</button>
        </div>
      </div>
      
      <!-- 双行对比视图 -->
      <div class="diff-compare">
        <div class="diff-row original">
          <span class="diff-label">原</span>
          <span class="diff-text">
            <template v-for="group in correctionDiffGroups" :key="'orig-' + group.id">
              <span v-if="group.type === 'equal'">{{ group.original }}</span>
              <span
                v-else-if="group.original"
                class="diff-chunk delete"
                :class="{ selected: !group.useNew }"
                @click="toggleCorrectionGroup(group.id)"
                title="点击切换"
              >{{ group.original }}</span>
            </template>
          </span>
        </div>
        <div class="diff-row corrected">
          <span class="diff-label">新</span>
          <span class="diff-text">
            <template v-for="group in correctionDiffGroups" :key="'corr-' + group.id">
              <span v-if="group.type === 'equal'">{{ group.corrected }}</span>
              <span
                v-else-if="group.corrected"
                class="diff-chunk insert"
                :class="{ selected: group.useNew }"
                @click="toggleCorrectionGroup(group.id)"
                title="点击切换"
              >{{ group.corrected }}</span>
            </template>
          </span>
        </div>
      </div>
      
      <div class="correction-actions">
        <button class="correction-btn dismiss" @click="emit('dismiss-correction')">忽略</button>
        <button class="correction-btn apply" @click="handleApplyCorrection">采用</button>
      </div>
    </div>

    <!-- 校正结果无差异提示 -->
    <div v-else-if="correctionResult && !correctionResult.has_diff" class="correction-no-diff">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
        <polyline points="22 4 12 14.01 9 11.01"/>
      </svg>
      <span>识别结果与原文一致</span>
      <button class="dismiss-btn" @click="emit('dismiss-correction')">×</button>
    </div>

    <!-- 快捷操作 -->
    <div class="quick-actions">
      <div class="actions-row">
        <span class="actions-label">文本处理</span>
        <div class="actions-group">
          <button class="quick-action-btn" @click="emit('remove-html')" title="移除HTML标签">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="4,7 4,4 20,4 20,7"/>
              <line x1="9" y1="20" x2="15" y2="20"/>
              <line x1="12" y1="4" x2="12" y2="20"/>
            </svg>
            <span>移除标签</span>
          </button>
          <button class="quick-action-btn" @click="emit('add-cjk-spaces')" title="中英文之间添加空格">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 6H3M21 12H3M21 18H3"/>
            </svg>
            <span>添加空格</span>
          </button>
          <button class="quick-action-btn" @click="emit('remove-punctuation')" title="删除标点符号">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="15" y1="9" x2="9" y2="15"/>
              <line x1="9" y1="9" x2="15" y2="15"/>
            </svg>
            <span>删除标点</span>
          </button>
          <button class="quick-action-btn" @click="emit('to-uppercase')" title="转换为大写字母">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 18L9 6L15 18"/>
              <path d="M5 14H13"/>
              <path d="M18 18V10"/>
              <path d="M15 13H21"/>
            </svg>
            <span>转大写</span>
          </button>
          <button class="quick-action-btn" @click="emit('to-lowercase')" title="转换为小写字母">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="9" cy="14" r="4"/>
              <path d="M13 10V18"/>
              <path d="M17 14H21"/>
            </svg>
            <span>转小写</span>
          </button>
        </div>
      </div>
      <div class="actions-row">
        <span class="actions-label">智能工具</span>
        <div class="actions-group">
          <button class="quick-action-btn add-dict-btn" @click="handleQuickAddDictionary" title="快速添加到本地词典 (Cmd/Ctrl+D)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
              <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
              <line x1="12" y1="9" x2="12" y2="15"/>
              <line x1="9" y1="12" x2="15" y2="12"/>
            </svg>
            <span>添加词典</span>
          </button>
          <button
            class="quick-action-btn correct-btn"
            :class="{ loading: isCorrecting }"
            :disabled="!fireredReady || isCorrecting"
            @click="emit('correct-entry')"
            :title="fireredReady ? '使用 FireRedASR 校正此条字幕' : '请先在设置中安装 FireRedASR'"
          >
            <svg v-if="!isCorrecting" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
            <span v-if="isCorrecting" class="loading-spinner"></span>
            <span>{{ isCorrecting ? '校正中...' : 'AI校正' }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- 无选中状态 -->
  <div v-else class="no-selection">
    <div class="no-selection-content">
      <div class="no-selection-icon-wrapper">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14,2 14,8 20,8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <line x1="10" y1="9" x2="8" y2="9"/>
        </svg>
      </div>
      <div class="no-selection-text-group">
        <p class="no-selection-title">选择字幕开始编辑</p>
        <p class="no-selection-hint">从左侧列表中点击任意字幕条目</p>
      </div>
      <div class="no-selection-shortcuts">
        <div class="shortcut-item">
          <kbd>↑</kbd><kbd>↓</kbd>
          <span>切换字幕</span>
        </div>
        <div class="shortcut-item">
          <kbd>⌘</kbd><kbd>N</kbd>
          <span>新建字幕</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 右侧字幕编辑区 */
.subtitle-edit-section {
  padding: 1.5rem;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

/* 编辑头部 */
.edit-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.edit-header-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.edit-badge {
  font-size: 0.75rem;
  font-weight: 600;
  color: #3b82f6;
  background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);
  padding: 0.25rem 0.625rem;
  border-radius: 6px;
  border: 1px solid #bfdbfe;
  user-select: none;
  -webkit-user-select: none;
}

.edit-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: #1e293b;
  margin: 0;
  user-select: none;
  -webkit-user-select: none;
}

/* 校正标记按钮 */
.correction-mark-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.625rem;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0.75rem;
  font-weight: 500;
  color: #64748b;
}

.correction-mark-btn:hover {
  background: #fef3c7;
  border-color: #fcd34d;
  color: #d97706;
}

.correction-mark-btn.marked {
  background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
  border-color: #f59e0b;
  color: #d97706;
}

.correction-mark-btn.marked svg {
  fill: #f59e0b;
  stroke: #d97706;
}

.correction-mark-btn svg {
  flex-shrink: 0;
}

.delete-entry-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  color: #94a3b8;
  transition: all 0.2s ease;
}

.delete-entry-btn:hover {
  background: #fef2f2;
  border-color: #fecaca;
  color: #ef4444;
}

/* 时间设置 - 紧凑单行布局 */
.time-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.875rem 1rem;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
}

.time-block {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.time-block .time-label {
  font-size: 0.75rem;
  font-weight: 500;
  color: #64748b;
  white-space: nowrap;
  user-select: none;
  -webkit-user-select: none;
}

.time-control {
  display: flex;
  align-items: center;
}

.time-btn-sm {
  width: 28px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 0.9375rem;
  font-weight: 500;
  color: #64748b;
  padding: 0;
}

.time-btn-sm:first-child {
  border-radius: 6px 0 0 6px;
  border-right: none;
}

.time-btn-sm:last-child {
  border-radius: 0 6px 6px 0;
  border-left: none;
}

.time-btn-sm:hover {
  background: #eff6ff;
  color: #3b82f6;
}

.time-btn-sm:active {
  background: #dbeafe;
}

.time-input-sm {
  width: 115px;
}

.time-input-sm :deep(.el-input__wrapper) {
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
  font-size: 0.8125rem;
  padding: 0 0.5rem;
  background: #ffffff;
  border-radius: 0;
  border: 1px solid #e2e8f0;
  border-left: none;
  border-right: none;
  transition: all 0.2s ease;
  height: 32px;
  box-sizing: border-box;
  box-shadow: none;
}

.time-input-sm :deep(.el-input__wrapper:hover) {
  border-color: #e2e8f0;
}

.time-input-sm :deep(.el-input__wrapper.is-focus) {
  border-color: #3b82f6;
  box-shadow: none;
}

.time-input-sm :deep(.el-input__inner) {
  text-align: center;
  color: #1e293b;
  font-size: 0.8125rem;
}

.time-separator {
  font-size: 0.875rem;
  color: #94a3b8;
  font-weight: 400;
  user-select: none;
  -webkit-user-select: none;
}

.duration-block {
  margin-left: auto;
  background: #f0f9ff;
  padding: 0.375rem 0.75rem;
  border-radius: 8px;
  border: 1px solid #bae6fd;
}

.duration-block .time-label {
  color: #0369a1;
}

.duration-value-sm {
  font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
  font-size: 0.8125rem;
  font-weight: 600;
  color: #0369a1;
  user-select: none;
  -webkit-user-select: none;
}

/* 文本编辑卡片 */
.text-edit-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.text-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
  border-bottom: 1px solid #e2e8f0;
}

.text-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.text-icon {
  color: #64748b;
}

.text-card-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: #475569;
  user-select: none;
  -webkit-user-select: none;
}

.result-hint {
  font-size: 0.6875rem;
  color: #94a3b8;
  font-weight: 400;
}

/* 有校正时的样式 */
.text-edit-card.has-correction {
  border-color: #93c5fd;
}

.text-edit-card.has-correction .text-card-header {
  background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);
}

.text-edit-card.has-correction .text-card-title {
  color: #2563eb;
}

.text-edit-card.has-correction .text-icon {
  color: #3b82f6;
}

.char-count {
  font-size: 0.75rem;
  color: #94a3b8;
  font-weight: 500;
  padding: 0.25rem 0.625rem;
  background: #ffffff;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
  user-select: none;
  -webkit-user-select: none;
}

.text-input-wrapper {
  padding: 1rem;
}

.text-input-new :deep(.el-input__wrapper) {
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  padding: 0 0.875rem;
  font-size: 0.9375rem;
  line-height: 1.6;
  transition: all 0.2s ease;
  background: #ffffff;
  height: 44px;
  box-sizing: border-box;
}

.text-input-new :deep(.el-input__wrapper:hover) {
  border-color: #cbd5e1;
}

.text-input-new :deep(.el-input__wrapper.is-focus) {
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.text-input-new :deep(.el-input__inner) {
  color: #1e293b;
  font-size: 0.9375rem;
}

.text-input-new :deep(.el-input__inner::placeholder) {
  color: #94a3b8;
}

/* 快捷操作 */
.quick-actions {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
  background: #f8fafc;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
}

.actions-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.actions-label {
  font-size: 0.6875rem;
  font-weight: 500;
  color: #94a3b8;
  white-space: nowrap;
  user-select: none;
  -webkit-user-select: none;
  min-width: 48px;
}

.actions-group {
  display: flex;
  gap: 0.375rem;
  flex: 1;
  flex-wrap: wrap;
}

.quick-action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.5rem 0.75rem;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 0.8125rem;
  font-weight: 500;
  color: #475569;
  user-select: none;
  -webkit-user-select: none;
}

.quick-action-btn:hover {
  background: #f1f5f9;
  border-color: #cbd5e1;
  color: #3b82f6;
  transform: translateY(-1px);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.04);
}

.quick-action-btn:active {
  transform: translateY(0);
}

.quick-action-btn svg {
  flex-shrink: 0;
}

.quick-action-btn.add-dict-btn {
  background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
  border-color: #86efac;
  color: #16a34a;
}

.quick-action-btn.add-dict-btn:hover {
  background: linear-gradient(135deg, #dcfce7 0%, #bbf7d0 100%);
  border-color: #4ade80;
}

.quick-action-btn.correct-btn {
  background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);
  border-color: #93c5fd;
  color: #2563eb;
}

.quick-action-btn.correct-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #dbeafe 0%, #bfdbfe 100%);
  border-color: #60a5fa;
}

.quick-action-btn.correct-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.quick-action-btn.correct-btn.loading {
  pointer-events: none;
}

.loading-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid #93c5fd;
  border-top-color: #2563eb;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 校正结果样式 - 简洁现代风格 */
.correction-result {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  overflow: hidden;
}

.correction-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 1rem;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
}

.correction-header-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  font-weight: 600;
  color: #64748b;
}

.correction-header svg {
  color: #3b82f6;
}

.diff-hint {
  font-size: 0.625rem;
  color: #94a3b8;
  font-weight: 400;
  margin-left: 0.25rem;
}

.header-quick-actions {
  display: flex;
  gap: 0.375rem;
}

/* 双行对比样式 */
.diff-compare {
  display: flex;
  flex-direction: column;
}

.diff-row {
  display: flex;
  align-items: flex-start;
  padding: 0.625rem 1rem;
  gap: 0.75rem;
  border-bottom: 1px solid #f1f5f9;
}

.diff-row:last-child {
  border-bottom: none;
}

.diff-row.original {
  background: #fafafa;
}

.diff-row.corrected {
  background: #f0fdf4;
}

.diff-label {
  flex-shrink: 0;
  font-size: 0.6875rem;
  font-weight: 600;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  min-width: 20px;
  text-align: center;
}

.diff-row.original .diff-label {
  background: #fee2e2;
  color: #dc2626;
}

.diff-row.corrected .diff-label {
  background: #dcfce7;
  color: #16a34a;
}

.diff-row.result {
  background: #eff6ff;
  border-top: 1px dashed #93c5fd;
}

.diff-row.result .diff-label {
  background: #dbeafe;
  color: #2563eb;
}

.diff-row.result .result-text {
  font-weight: 500;
  color: #1e40af;
}

.diff-text {
  flex: 1;
  font-size: 0.875rem;
  line-height: 1.6;
  color: #1e293b;
  word-break: break-word;
}

.diff-chunk {
  padding: 0.0625rem 0.25rem;
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.15s;
}

.diff-chunk.delete {
  background: #fecaca;
  color: #dc2626;
  text-decoration: line-through;
  opacity: 0.5;
}

.diff-chunk.delete.selected {
  background: #fca5a5;
  text-decoration: none;
  opacity: 1;
  font-weight: 500;
}

.diff-chunk.delete:hover {
  background: #fca5a5;
}

.diff-chunk.insert {
  background: #bbf7d0;
  color: #16a34a;
  opacity: 0.5;
}

.diff-chunk.insert.selected {
  background: #86efac;
  opacity: 1;
  font-weight: 500;
}

.diff-chunk.insert:hover {
  background: #86efac;
}

.correction-compare {
  display: flex;
  flex-direction: column;
}

.correction-item {
  display: flex;
  align-items: center;
  padding: 0.75rem 1rem;
  gap: 1rem;
  border-bottom: 1px solid #f1f5f9;
}

.correction-item:last-child {
  border-bottom: none;
}

.correction-item.original {
  background: #fafafa;
}

.correction-item.corrected {
  background: #f0fdf4;
}

.correction-label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  min-width: 36px;
  text-align: center;
}

.correction-item.original .correction-label {
  background: #f1f5f9;
  color: #64748b;
}

.correction-item.corrected .correction-label {
  background: #dcfce7;
  color: #16a34a;
}

.correction-text {
  flex: 1;
  font-size: 0.875rem;
  color: #1e293b;
  line-height: 1.5;
}

/* 差异高亮 */
.correction-text :deep(.diff-del) {
  background: #fecaca;
  color: #dc2626;
  text-decoration: line-through;
  padding: 0 2px;
  border-radius: 2px;
}

.correction-text :deep(.diff-add) {
  background: #bbf7d0;
  color: #16a34a;
  font-weight: 500;
  padding: 0 2px;
  border-radius: 2px;
}

.correction-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: #f8fafc;
  border-top: 1px solid #e2e8f0;
}

.correction-btn {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-size: 0.8125rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.correction-btn.dismiss {
  background: #fff;
  border: 1px solid #e2e8f0;
  color: #64748b;
}

.correction-btn.dismiss:hover {
  background: #f1f5f9;
  border-color: #cbd5e1;
}

.correction-btn.apply {
  background: #3b82f6;
  border: 1px solid #2563eb;
  color: #fff;
}

.correction-btn.apply:hover {
  background: #2563eb;
}

/* 细粒度编辑样式 */
.fine-tune-editor {
  padding: 0.75rem 1rem;
}

.quick-select-btn {
  padding: 0.25rem 0.5rem;
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  font-size: 0.6875rem;
  color: #64748b;
  cursor: pointer;
  transition: all 0.15s;
}

.quick-select-btn:hover {
  background: #eff6ff;
  border-color: #93c5fd;
  color: #3b82f6;
}

.diff-segments {
  font-size: 0.875rem;
  line-height: 1.8;
  word-break: break-word;
  padding: 0.5rem;
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  margin-bottom: 0.75rem;
}

.seg-equal {
  color: #1e293b;
}

.seg-change {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.125rem 0.375rem;
  margin: 0.125rem;
  border-radius: 4px;
  background: #fef3c7;
  border: 1px solid #fcd34d;
  cursor: pointer;
  transition: all 0.15s;
}

.seg-change:hover {
  border-color: #f59e0b;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.seg-change.use-new {
  background: #dcfce7;
  border-color: #86efac;
}

.seg-original {
  color: #dc2626;
  text-decoration: line-through;
  opacity: 0.5;
}

.seg-original.active {
  text-decoration: none;
  opacity: 1;
  font-weight: 500;
}

.seg-arrow {
  color: #94a3b8;
  font-size: 0.625rem;
}

.seg-corrected {
  color: #16a34a;
  opacity: 0.5;
}

.seg-corrected.active {
  opacity: 1;
  font-weight: 500;
}

.result-preview {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.5rem;
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
}

.result-label {
  font-size: 0.6875rem;
  color: #94a3b8;
  white-space: nowrap;
}

.result-text {
  font-size: 0.8125rem;
  color: #1e293b;
  font-weight: 500;
  word-break: break-word;
}

/* 无差异提示 */
.correction-no-diff {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  font-size: 0.8125rem;
  color: #16a34a;
}

.correction-no-diff svg {
  flex-shrink: 0;
  color: #22c55e;
}

.correction-no-diff .dismiss-btn {
  margin-left: auto;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  color: #94a3b8;
  font-size: 1rem;
}

.correction-no-diff .dismiss-btn:hover {
  background: #f1f5f9;
  color: #64748b;
}

/* 无选中状态 */
.no-selection {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3rem;
  background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
}

.no-selection-content {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.25rem;
  max-width: 280px;
}

.no-selection-icon-wrapper {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #ffffff 0%, #f8fafc 100%);
  border-radius: 20px;
  border: 1px solid #e2e8f0;
  color: #cbd5e1;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.04);
}

.no-selection-text-group {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.no-selection-title {
  font-size: 1rem;
  font-weight: 600;
  color: #475569;
  margin: 0;
}

.no-selection-hint {
  font-size: 0.8125rem;
  color: #94a3b8;
  margin: 0;
}

.no-selection-shortcuts {
  display: flex;
  gap: 1.5rem;
  padding-top: 0.5rem;
}

.shortcut-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: #94a3b8;
}

.shortcut-item kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  padding: 0 0.375rem;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  font-family: inherit;
  font-size: 0.6875rem;
  font-weight: 500;
  color: #64748b;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}
</style>
