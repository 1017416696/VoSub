<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { computeDiff, segmentsToDiffGroups, type DiffGroup } from '@/utils/textDiff'

interface DictionaryReplacement {
  id: number
  text: string
  newText: string
  replacements: Array<{ from: string; to: string }>
}

interface PreviewItem extends DictionaryReplacement {
  diffGroups: DiffGroup[]
}

const props = defineProps<{
  visible: boolean
  items: DictionaryReplacement[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'replace', id: number, newText: string): void
  (e: 'replaceAll', items: DictionaryReplacement[]): void
  (e: 'cancel'): void
}>()

const pendingItems = ref<PreviewItem[]>([])
const replacedCount = ref(0)

watch(() => props.items, (newItems) => {
  pendingItems.value = newItems.map(item => {
    const segments = computeDiff(item.text, item.newText)
    const diffGroups = segmentsToDiffGroups(segments)
    return { ...item, diffGroups }
  })
  replacedCount.value = 0
}, { immediate: true })

// ESC 快捷键关闭弹窗
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.visible) {
    e.preventDefault()
    handleClose()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
})

const handleReplace = (item: PreviewItem) => {
  emit('replace', item.id, item.newText)
  pendingItems.value = pendingItems.value.filter(i => i.id !== item.id)
  replacedCount.value++
  if (pendingItems.value.length === 0) {
    emit('update:visible', false)
  }
}

const handleIgnore = (item: PreviewItem) => {
  pendingItems.value = pendingItems.value.filter(i => i.id !== item.id)
  if (pendingItems.value.length === 0) {
    emit('update:visible', false)
  }
}

const handleReplaceAll = () => {
  emit('replaceAll', pendingItems.value)
  emit('update:visible', false)
}

const handleClose = () => {
  emit('cancel')
  emit('update:visible', false)
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="dict-overlay" @click.self="handleClose">
        <div class="dict-dialog">
          <div class="dict-header">
            <div class="dict-title">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
              </svg>
              <span>本地词典替换</span>
            </div>
            <div class="dict-stats">
              <span v-if="replacedCount > 0" class="stat-replaced">已替换 {{ replacedCount }} 条</span>
              <span class="stat-pending">剩余 {{ pendingItems.length }} 条</span>
            </div>
          </div>

          <div class="dict-list">
            <TransitionGroup name="item">
              <div v-for="item in pendingItems" :key="item.id" class="dict-item">
                <div class="item-header">
                  <span class="item-id">#{{ item.id }}</span>
                  <div class="item-tags">
                    <span v-for="r in item.replacements" :key="r.from" class="tag">
                      {{ r.from }} → {{ r.to }}
                    </span>
                  </div>
                </div>
                
                <div class="item-row">
                  <div class="item-diff">
                    <div class="diff-line old">
                      <span class="diff-badge">原</span>
                      <span class="diff-content">
                        <template v-for="group in item.diffGroups" :key="group.id">
                          <span v-if="group.type === 'equal'">{{ group.original }}</span>
                          <span v-else class="del">{{ group.original }}</span>
                        </template>
                      </span>
                    </div>
                    <div class="diff-line new">
                      <span class="diff-badge">新</span>
                      <span class="diff-content">
                        <template v-for="group in item.diffGroups" :key="group.id">
                          <span v-if="group.type === 'equal'">{{ group.corrected }}</span>
                          <span v-else class="ins">{{ group.corrected }}</span>
                        </template>
                      </span>
                    </div>
                  </div>
                  <div class="item-btns">
                    <button class="btn-skip" @click="handleIgnore(item)">忽略</button>
                    <button class="btn-apply" @click="handleReplace(item)">替换</button>
                  </div>
                </div>
              </div>
            </TransitionGroup>

            <div v-if="pendingItems.length === 0" class="done-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                <polyline points="22 4 12 14.01 9 11.01"/>
              </svg>
              <p>处理完成</p>
            </div>
          </div>

          <div class="dict-footer">
            <button class="btn-close" @click="handleClose">关闭</button>
            <button v-if="pendingItems.length > 1" class="btn-all" @click="handleReplaceAll">
              全部替换 ({{ pendingItems.length }})
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>


<style scoped>
.dict-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.dict-dialog {
  background: #fff;
  border-radius: 12px;
  width: 560px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
}

.dict-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.dict-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
}

.dict-title svg { color: #10b981; }

.dict-stats {
  display: flex;
  gap: 12px;
  font-size: 13px;
}

.stat-replaced { color: #10b981; font-weight: 500; }
.stat-pending { color: #6b7280; }

.dict-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.dict-item {
  padding: 12px 16px;
  background: #f9fafb;
  border-radius: 8px;
  margin-bottom: 8px;
}

.dict-item:last-child { margin-bottom: 0; }

.item-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.item-id { font-size: 13px; font-weight: 600; color: #6b7280; }

.item-tags { display: flex; flex-wrap: wrap; gap: 6px; }

.tag {
  font-size: 11px;
  padding: 2px 8px;
  background: #ecfdf5;
  color: #059669;
  border-radius: 4px;
  font-family: ui-monospace, monospace;
}

.item-row {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.item-diff {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.diff-line {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  font-size: 14px;
  line-height: 1.6;
}

.diff-badge {
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

.diff-line.old .diff-badge { background: #fee2e2; color: #dc2626; }
.diff-line.new .diff-badge { background: #dcfce7; color: #16a34a; }

.diff-content { flex: 1; word-break: break-word; color: #374151; }
.diff-line.old .diff-content { color: #6b7280; }

.del {
  background: rgba(220, 53, 69, 0.15);
  color: #dc2626;
  text-decoration: line-through;
  padding: 0 2px;
  border-radius: 2px;
}

.ins {
  background: rgba(40, 167, 69, 0.15);
  color: #16a34a;
  padding: 0 2px;
  border-radius: 2px;
}

.item-btns { 
  display: flex; 
  flex-direction: row;
  gap: 8px;
  flex-shrink: 0;
  align-items: center;
}

.btn-skip, .btn-apply {
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: none;
}

.btn-skip {
  background: #fff;
  color: #6b7280;
  border: 1px solid #d1d5db;
}

.btn-skip:hover { background: #f3f4f6; color: #374151; }

.btn-apply { background: #10b981; color: #fff; }
.btn-apply:hover { background: #059669; }

.done-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 48px 24px;
  color: #10b981;
}

.done-state svg { margin-bottom: 12px; }
.done-state p { margin: 0; font-size: 15px; font-weight: 500; }

.dict-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid #e5e7eb;
  background: #f9fafb;
}

.btn-close, .btn-all {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: none;
}

.btn-close {
  background: #fff;
  color: #374151;
  border: 1px solid #d1d5db;
}

.btn-close:hover { background: #f3f4f6; }

.btn-all { background: #10b981; color: #fff; }
.btn-all:hover { background: #059669; }

.item-enter-active, .item-leave-active { transition: all 0.3s ease; }
.item-enter-from { opacity: 0; transform: translateX(-20px); }
.item-leave-to { opacity: 0; transform: translateX(20px); }

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
