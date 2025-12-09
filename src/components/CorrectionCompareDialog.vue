<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { CorrectionEntry, CorrectionEntryWithChoice, CorrectionChoice } from '@/types/correction'
import type { TimeStamp } from '@/types/subtitle'
import { computeDiff, segmentsToDiffGroups, buildFinalText, type DiffGroup } from '@/utils/textDiff'

const props = defineProps<{
  visible: boolean
  entries: CorrectionEntry[]
  audioPath?: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', entries: CorrectionEntryWithChoice[]): void
  (e: 'cancel'): void
}>()

// 带选择状态的条目列表
const entriesWithChoice = ref<CorrectionEntryWithChoice[]>([])

// 每个条目的差异组
const entryDiffGroups = ref<Map<number, DiffGroup[]>>(new Map())

// 只显示有差异的条目
const showOnlyDiff = ref(true)

// 当前展开进行细粒度编辑的条目 ID
const expandedEntryId = ref<number | null>(null)

// 初始化选择状态和差异组
watch(() => props.entries, (newEntries) => {
  entriesWithChoice.value = newEntries.map(entry => ({
    ...entry,
    choice: entry.has_diff ? 'corrected' : 'original' as CorrectionChoice,
    finalText: undefined
  }))

  // 计算每个条目的差异
  const diffMap = new Map<number, DiffGroup[]>()
  newEntries.forEach(entry => {
    if (entry.has_diff) {
      const segments = computeDiff(entry.original, entry.corrected)
      const groups = segmentsToDiffGroups(segments)
      diffMap.set(entry.id, groups)
    }
  })
  entryDiffGroups.value = diffMap
}, { immediate: true })

// 过滤后的条目
const filteredEntries = computed(() => {
  if (showOnlyDiff.value) {
    return entriesWithChoice.value.filter(e => e.has_diff)
  }
  return entriesWithChoice.value
})

// 统计信息
const stats = computed(() => {
  const total = entriesWithChoice.value.length
  const diffCount = entriesWithChoice.value.filter(e => e.has_diff).length
  const chosenCorrected = entriesWithChoice.value.filter(e => {
    if (!e.has_diff) return false
    if (e.finalText !== undefined) return true // 有细粒度编辑
    return e.choice === 'corrected'
  }).length
  return { total, diffCount, chosenCorrected }
})

// 格式化时间戳
function formatTime(ts: TimeStamp): string {
  const pad = (n: number, len = 2) => n.toString().padStart(len, '0')
  return `${pad(ts.hours)}:${pad(ts.minutes)}:${pad(ts.seconds)},${pad(ts.milliseconds, 3)}`
}

// 获取条目的差异组
function getDiffGroups(entryId: number): DiffGroup[] {
  return entryDiffGroups.value.get(entryId) || []
}

// 切换差异组的选择
function toggleDiffGroup(entryId: number, groupId: number) {
  const groups = entryDiffGroups.value.get(entryId)
  if (!groups) return

  const group = groups.find(g => g.id === groupId)
  if (group && group.type === 'change') {
    group.useNew = !group.useNew
    // 更新最终文本
    updateFinalText(entryId)
  }
}

// 更新条目的最终文本
function updateFinalText(entryId: number) {
  const groups = entryDiffGroups.value.get(entryId)
  const entry = entriesWithChoice.value.find(e => e.id === entryId)
  if (!groups || !entry) return

  const finalText = buildFinalText(groups)
  entry.finalText = finalText
  // 如果最终文本与原文相同，设为 original；与校正相同，设为 corrected
  if (finalText === entry.original) {
    entry.choice = 'original'
    entry.finalText = undefined
  } else if (finalText === entry.corrected) {
    entry.choice = 'corrected'
    entry.finalText = undefined
  } else {
    entry.choice = 'corrected' // 混合状态
  }
}

// 切换展开状态
function toggleExpand(entryId: number) {
  expandedEntryId.value = expandedEntryId.value === entryId ? null : entryId
}

// 快速选择：全部采用原文
function useAllOriginal() {
  entriesWithChoice.value.forEach(e => {
    if (e.has_diff) {
      e.choice = 'original'
      e.finalText = undefined
      // 重置差异组
      const groups = entryDiffGroups.value.get(e.id)
      if (groups) {
        groups.forEach(g => { if (g.type === 'change') g.useNew = false })
      }
    }
  })
}

// 快速选择：全部采用校正
function useAllCorrected() {
  entriesWithChoice.value.forEach(e => {
    if (e.has_diff) {
      e.choice = 'corrected'
      e.finalText = undefined
      // 重置差异组
      const groups = entryDiffGroups.value.get(e.id)
      if (groups) {
        groups.forEach(g => { if (g.type === 'change') g.useNew = true })
      }
    }
  })
}

// 单条快速选择原文
function useOriginal(entry: CorrectionEntryWithChoice) {
  entry.choice = 'original'
  entry.finalText = undefined
  const groups = entryDiffGroups.value.get(entry.id)
  if (groups) {
    groups.forEach(g => { if (g.type === 'change') g.useNew = false })
  }
}

// 单条快速选择校正
function useCorrected(entry: CorrectionEntryWithChoice) {
  entry.choice = 'corrected'
  entry.finalText = undefined
  const groups = entryDiffGroups.value.get(entry.id)
  if (groups) {
    groups.forEach(g => { if (g.type === 'change') g.useNew = true })
  }
}

// 获取预览文本
function getPreviewText(entry: CorrectionEntryWithChoice): string {
  if (entry.finalText !== undefined) {
    return entry.finalText
  }
  return entry.choice === 'corrected' ? entry.corrected : entry.original
}

// 确认应用
function handleConfirm() {
  emit('confirm', entriesWithChoice.value)
  emit('update:visible', false)
}

// 取消
function handleCancel() {
  emit('cancel')
  emit('update:visible', false)
}
</script>

<template>
  <div v-if="visible" class="correction-dialog-overlay" @click.self="handleCancel">
    <div class="correction-dialog">
      <!-- 头部 -->
      <div class="dialog-header">
        <h3>字幕校正对比</h3>
        <div class="header-stats">
          共 {{ stats.total }} 条，{{ stats.diffCount }} 处差异，已选择 {{ stats.chosenCorrected }} 处校正
        </div>
      </div>

      <!-- 工具栏 -->
      <div class="dialog-toolbar">
        <label class="filter-checkbox">
          <input type="checkbox" v-model="showOnlyDiff" />
          只显示有差异的条目
        </label>
        <div class="toolbar-actions">
          <button class="btn-secondary" @click="useAllOriginal">全部采用原文</button>
          <button class="btn-secondary" @click="useAllCorrected">全部采用校正</button>
        </div>
      </div>

      <!-- 对比列表 -->
      <div class="compare-list">
        <div
          v-for="entry in filteredEntries"
          :key="entry.id"
          class="compare-item"
          :class="{ 'has-diff': entry.has_diff, 'expanded': expandedEntryId === entry.id }"
        >
          <div class="item-header">
            <div class="item-info">
              <span class="item-id">#{{ entry.id }}</span>
              <span class="item-time">
                {{ formatTime(entry.start_time) }} → {{ formatTime(entry.end_time) }}
              </span>
            </div>
            <div class="item-actions" v-if="entry.has_diff">
              <button
                class="btn-small"
                :class="{ active: entry.choice === 'original' && !entry.finalText }"
                @click.stop="useOriginal(entry)"
              >
                原文
              </button>
              <button
                class="btn-small"
                :class="{ active: entry.choice === 'corrected' && !entry.finalText }"
                @click.stop="useCorrected(entry)"
              >
                校正
              </button>
              <button
                class="btn-small btn-edit"
                :class="{ active: expandedEntryId === entry.id }"
                @click.stop="toggleExpand(entry.id)"
              >
                细调
              </button>
            </div>
          </div>

          <!-- 简洁视图：差异高亮预览 -->
          <div v-if="expandedEntryId !== entry.id" class="item-preview">
            <div class="preview-row">
              <span class="preview-label">原</span>
              <span class="preview-text">
                <template v-for="group in getDiffGroups(entry.id)" :key="group.id">
                  <span v-if="group.type === 'equal'">{{ group.original }}</span>
                  <span v-else class="diff-delete" :class="{ 'not-used': group.useNew }">
                    {{ group.original }}
                  </span>
                </template>
                <span v-if="!entry.has_diff">{{ entry.original }}</span>
              </span>
            </div>
            <div class="preview-row">
              <span class="preview-label">新</span>
              <span class="preview-text">
                <template v-for="group in getDiffGroups(entry.id)" :key="group.id">
                  <span v-if="group.type === 'equal'">{{ group.corrected }}</span>
                  <span v-else class="diff-insert" :class="{ 'not-used': !group.useNew }">
                    {{ group.corrected }}
                  </span>
                </template>
                <span v-if="!entry.has_diff">{{ entry.corrected }}</span>
              </span>
            </div>
          </div>

          <!-- 展开视图：可点击选择的差异片段 -->
          <div v-else class="item-expanded">
            <div class="diff-editor">
              <div class="diff-hint">点击高亮部分切换使用原文或校正</div>
              <div class="diff-segments">
                <template v-for="group in getDiffGroups(entry.id)" :key="group.id">
                  <span v-if="group.type === 'equal'" class="seg-equal">{{ group.original }}</span>
                  <span
                    v-else
                    class="seg-change"
                    :class="{ 'use-new': group.useNew }"
                    @click="toggleDiffGroup(entry.id, group.id)"
                  >
                    <span class="seg-original" :class="{ active: !group.useNew }">{{ group.original }}</span>
                    <span class="seg-arrow">→</span>
                    <span class="seg-corrected" :class="{ active: group.useNew }">{{ group.corrected }}</span>
                  </span>
                </template>
              </div>
            </div>
            <div class="result-preview">
              <span class="result-label">结果预览：</span>
              <span class="result-text">{{ getPreviewText(entry) }}</span>
            </div>
          </div>

          <!-- 状态标签 -->
          <div class="item-status">
            <span v-if="!entry.has_diff" class="status-tag same">无差异</span>
            <span v-else-if="entry.finalText" class="status-tag use-mixed">自定义</span>
            <span v-else-if="entry.choice === 'corrected'" class="status-tag use-corrected">采用校正</span>
            <span v-else class="status-tag use-original">采用原文</span>
          </div>
        </div>

        <div v-if="filteredEntries.length === 0" class="empty-state">
          {{ showOnlyDiff ? '没有发现差异' : '没有校正结果' }}
        </div>
      </div>

      <!-- 底部按钮 -->
      <div class="dialog-footer">
        <button class="btn-cancel" @click="handleCancel">取消</button>
        <button class="btn-confirm" @click="handleConfirm">确认应用</button>
      </div>
    </div>
  </div>
</template>


<style scoped>
.correction-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.correction-dialog {
  background: var(--bg-color, #fff);
  border-radius: 12px;
  width: 90%;
  max-width: 900px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.dialog-header {
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.dialog-header h3 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
}

.header-stats {
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.dialog-toolbar {
  padding: 12px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
  background: var(--bg-secondary, #f5f5f5);
}

.filter-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
}

.btn-secondary {
  padding: 6px 12px;
  border: 1px solid var(--border-color, #ddd);
  background: var(--bg-color, #fff);
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: var(--bg-hover, #f0f0f0);
}

.compare-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
}

.compare-item {
  padding: 16px;
  margin-bottom: 12px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  background: var(--bg-color, #fff);
}

.compare-item.has-diff {
  border-color: var(--warning-color, #f0ad4e);
}

.compare-item.expanded {
  border-color: var(--primary-color, #007aff);
  box-shadow: 0 2px 8px rgba(0, 122, 255, 0.15);
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.item-info {
  display: flex;
  gap: 12px;
  font-size: 13px;
}

.item-id {
  font-weight: 600;
  color: var(--primary-color, #007aff);
}

.item-time {
  color: var(--text-secondary, #666);
  font-family: monospace;
}

.item-actions {
  display: flex;
  gap: 6px;
}

.btn-small {
  padding: 4px 10px;
  border: 1px solid var(--border-color, #ddd);
  background: var(--bg-color, #fff);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-small:hover {
  background: var(--bg-hover, #f0f0f0);
}

.btn-small.active {
  background: var(--primary-color, #007aff);
  border-color: var(--primary-color, #007aff);
  color: #fff;
}

.btn-edit {
  background: var(--bg-secondary, #f5f5f5);
}

.btn-edit.active {
  background: var(--warning-color, #f0ad4e);
  border-color: var(--warning-color, #f0ad4e);
  color: #fff;
}

/* 预览视图 */
.item-preview {
  font-size: 14px;
  line-height: 1.6;
}

.preview-row {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
}

.preview-label {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
}

.preview-row:first-child .preview-label {
  background: rgba(220, 53, 69, 0.1);
  color: #dc3545;
}

.preview-row:last-child .preview-label {
  background: rgba(40, 167, 69, 0.1);
  color: #28a745;
}

.preview-text {
  flex: 1;
  word-break: break-word;
}

.diff-delete {
  background: rgba(220, 53, 69, 0.2);
  color: #dc3545;
  text-decoration: line-through;
  padding: 0 2px;
  border-radius: 2px;
}

.diff-delete.not-used {
  background: rgba(220, 53, 69, 0.1);
  text-decoration: none;
  color: inherit;
}

.diff-insert {
  background: rgba(40, 167, 69, 0.2);
  color: #28a745;
  padding: 0 2px;
  border-radius: 2px;
}

.diff-insert.not-used {
  background: rgba(40, 167, 69, 0.1);
  text-decoration: line-through;
  color: var(--text-secondary, #999);
}

/* 展开编辑视图 */
.item-expanded {
  background: var(--bg-secondary, #f8f9fa);
  border-radius: 6px;
  padding: 12px;
}

.diff-editor {
  margin-bottom: 12px;
}

.diff-hint {
  font-size: 12px;
  color: var(--text-secondary, #666);
  margin-bottom: 8px;
}

.diff-segments {
  font-size: 14px;
  line-height: 2;
  word-break: break-word;
}

.seg-equal {
  color: var(--text-color, #333);
}

.seg-change {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px;
  margin: 2px;
  border-radius: 4px;
  background: var(--bg-color, #fff);
  border: 1px solid var(--border-color, #ddd);
  cursor: pointer;
  transition: all 0.2s;
}

.seg-change:hover {
  border-color: var(--primary-color, #007aff);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.seg-original {
  color: #dc3545;
  text-decoration: line-through;
  opacity: 0.5;
}

.seg-original.active {
  text-decoration: none;
  opacity: 1;
  font-weight: 500;
}

.seg-arrow {
  color: var(--text-secondary, #999);
  font-size: 12px;
}

.seg-corrected {
  color: #28a745;
  opacity: 0.5;
}

.seg-corrected.active {
  opacity: 1;
  font-weight: 500;
}

.result-preview {
  padding-top: 12px;
  border-top: 1px solid var(--border-color, #e0e0e0);
  font-size: 13px;
}

.result-label {
  color: var(--text-secondary, #666);
  margin-right: 8px;
}

.result-text {
  color: var(--text-color, #333);
  font-weight: 500;
}

.item-status {
  margin-top: 12px;
  text-align: right;
}

.status-tag {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.status-tag.same {
  background: var(--bg-secondary, #f0f0f0);
  color: var(--text-secondary, #666);
}

.status-tag.use-corrected {
  background: rgba(40, 167, 69, 0.1);
  color: #28a745;
}

.status-tag.use-original {
  background: rgba(0, 122, 255, 0.1);
  color: var(--primary-color, #007aff);
}

.status-tag.use-mixed {
  background: rgba(240, 173, 78, 0.1);
  color: #f0ad4e;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary, #666);
}

.dialog-footer {
  padding: 16px 24px;
  border-top: 1px solid var(--border-color, #e0e0e0);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-cancel,
.btn-confirm {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel {
  background: var(--bg-secondary, #f0f0f0);
  border: none;
  color: var(--text-color, #333);
}

.btn-cancel:hover {
  background: var(--bg-hover, #e0e0e0);
}

.btn-confirm {
  background: var(--primary-color, #007aff);
  border: none;
  color: #fff;
}

.btn-confirm:hover {
  background: var(--primary-hover, #0056b3);
}

/* 暗色主题适配 */
:root.dark .correction-dialog {
  --bg-color: #1e1e1e;
  --bg-secondary: #2d2d2d;
  --bg-hover: #3d3d3d;
  --border-color: #404040;
  --text-color: #e0e0e0;
  --text-secondary: #999;
}
</style>
