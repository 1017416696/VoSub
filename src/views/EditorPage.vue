<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useSubtitleStore } from '@/stores/subtitle'
import { useAudioStore } from '@/stores/audio'
import { useConfigStore } from '@/stores/config'

const router = useRouter()
const subtitleStore = useSubtitleStore()
const audioStore = useAudioStore()
const configStore = useConfigStore()

// UI 状态
const searchText = ref('')
const showSearchPanel = ref(false)
const selectedEntryId = ref<number | null>(null)

// 计算属性
const hasContent = computed(() => subtitleStore.entries.length > 0)
const hasAudio = computed(() => audioStore.currentAudio !== null)
const canUndo = computed(() => subtitleStore.canUndo)
const canRedo = computed(() => subtitleStore.canRedo)

// 键盘快捷键处理
const handleKeydown = (e: KeyboardEvent) => {
  const shortcuts = configStore.keyboardShortcuts
  const key = `${e.ctrlKey ? 'Ctrl+' : ''}${e.shiftKey ? 'Shift+' : ''}${e.altKey ? 'Alt+' : ''}${e.key}`

  // 保存
  if (shortcuts.save === key) {
    e.preventDefault()
    handleSave()
  }
  // 撤销
  else if (shortcuts.undo === key) {
    e.preventDefault()
    subtitleStore.undo()
  }
  // 重做
  else if (shortcuts.redo === key) {
    e.preventDefault()
    subtitleStore.redo()
  }
  // 播放/暂停
  else if (shortcuts.playPause === key) {
    e.preventDefault()
    audioStore.togglePlay()
  }
  // 查找
  else if (shortcuts.find === key) {
    e.preventDefault()
    showSearchPanel.value = !showSearchPanel.value
  }
  // 新增字幕
  else if (shortcuts.addEntry === key) {
    e.preventDefault()
    handleAddEntry()
  }
  // 删除字幕
  else if (shortcuts.deleteEntry === key && selectedEntryId.value !== null) {
    e.preventDefault()
    handleDeleteEntry(selectedEntryId.value)
  }
}

// 保存文件
const handleSave = async () => {
  if (!subtitleStore.currentFilePath) {
    ElMessage.warning('没有可保存的文件')
    return
  }

  try {
    await subtitleStore.saveToFile()
    ElMessage.success('保存成功')
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  }
}

// 添加字幕条目
const handleAddEntry = () => {
  subtitleStore.addEntry()
  ElMessage.success('已添加新字幕')
}

// 删除字幕条目
const handleDeleteEntry = async (id: number) => {
  try {
    await ElMessageBox.confirm('确定要删除这条字幕吗？', '确认删除', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    })

    subtitleStore.deleteEntry(id)
    ElMessage.success('已删除字幕')
    selectedEntryId.value = null
  } catch {
    // 用户取消
  }
}

// 返回欢迎页
const goBack = async () => {
  if (subtitleStore.hasUnsavedChanges) {
    try {
      await ElMessageBox.confirm('有未保存的更改，确定要离开吗？', '确认', {
        confirmButtonText: '离开',
        cancelButtonText: '取消',
        type: 'warning',
      })
    } catch {
      return
    }
  }

  router.push('/')
}

// 搜索
const handleSearch = () => {
  if (!searchText.value.trim()) {
    ElMessage.warning('请输入搜索内容')
    return
  }

  const results = subtitleStore.search(searchText.value)
  ElMessage.info(`找到 ${results.length} 条结果`)
}

// 生命周期
onMounted(() => {
  window.addEventListener('keydown', handleKeydown)

  // 如果没有内容，提示用户加载文件
  if (!hasContent.value) {
    ElMessage.info('请先加载 SRT 文件')
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="editor-page">
    <!-- 顶部工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <el-button :icon="'Back'" @click="goBack">返回</el-button>
        <el-divider direction="vertical" />
        <el-button :icon="'FolderOpened'" type="primary">打开文件</el-button>
        <el-button
          :icon="'Document'"
          :disabled="!hasContent"
          @click="handleSave"
        >
          保存
        </el-button>
      </div>

      <div class="toolbar-center">
        <span v-if="subtitleStore.currentFilePath" class="file-name">
          {{ subtitleStore.currentFilePath.split('/').pop() }}
        </span>
        <span v-else class="text-gray-400">未加载文件</span>
      </div>

      <div class="toolbar-right">
        <el-button
          :icon="'RefreshLeft'"
          :disabled="!canUndo"
          @click="subtitleStore.undo()"
          title="撤销 (Ctrl+Z)"
        />
        <el-button
          :icon="'RefreshRight'"
          :disabled="!canRedo"
          @click="subtitleStore.redo()"
          title="重做 (Ctrl+Y)"
        />
        <el-divider direction="vertical" />
        <el-button
          :icon="'Search'"
          @click="showSearchPanel = !showSearchPanel"
          title="查找 (Ctrl+F)"
        />
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="content-area">
      <!-- 左侧：字幕列表 -->
      <div class="subtitle-panel">
        <div class="panel-header">
          <h3 class="panel-title">字幕列表</h3>
          <el-button
            size="small"
            :icon="'Plus'"
            type="primary"
            @click="handleAddEntry"
          >
            添加
          </el-button>
        </div>

        <!-- 搜索框 -->
        <div v-if="showSearchPanel" class="search-panel">
          <el-input
            v-model="searchText"
            placeholder="搜索字幕内容..."
            :prefix-icon="'Search'"
            clearable
            @keyup.enter="handleSearch"
          >
            <template #append>
              <el-button :icon="'Search'" @click="handleSearch" />
            </template>
          </el-input>
        </div>

        <!-- 字幕列表 -->
        <div class="subtitle-list">
          <div
            v-if="!hasContent"
            class="empty-state"
          >
            <i class="i-mdi-file-document-outline text-6xl text-gray-300 mb-4"></i>
            <p class="text-gray-500">暂无字幕数据</p>
            <el-button type="primary" class="mt-4" @click="goBack">
              加载文件
            </el-button>
          </div>

          <div
            v-for="entry in subtitleStore.entries"
            v-else
            :key="entry.id"
            class="subtitle-item"
            :class="{
              'is-selected': selectedEntryId === entry.id,
              'has-conflict': entry.hasConflict,
            }"
            @click="selectedEntryId = entry.id"
          >
            <div class="item-header">
              <span class="item-number">#{{ entry.id }}</span>
              <span class="item-time">
                {{ subtitleStore.formatTimeStamp(entry.startTime) }}
                →
                {{ subtitleStore.formatTimeStamp(entry.endTime) }}
              </span>
            </div>
            <div class="item-text">{{ entry.text }}</div>
            <div class="item-actions">
              <el-button
                size="small"
                :icon="'Edit'"
                text
                @click.stop="entry.isEditing = true"
              />
              <el-button
                size="small"
                :icon="'Delete'"
                text
                type="danger"
                @click.stop="handleDeleteEntry(entry.id)"
              />
            </div>
          </div>
        </div>

        <!-- 底部统计 -->
        <div class="panel-footer">
          <span class="text-sm text-gray-500">
            共 {{ subtitleStore.entries.length }} 条字幕
          </span>
        </div>
      </div>

      <!-- 右侧：音频播放器和时间轴 -->
      <div class="player-panel">
        <div class="panel-header">
          <h3 class="panel-title">音频播放器</h3>
        </div>

        <div v-if="!hasAudio" class="empty-state">
          <i class="i-mdi-music-note-outline text-6xl text-gray-300 mb-4"></i>
          <p class="text-gray-500">暂未加载音频</p>
          <p class="text-sm text-gray-400 mt-2">您可以在没有音频的情况下编辑字幕</p>
        </div>

        <div v-else class="audio-player">
          <!-- 音频信息 -->
          <div class="audio-info">
            <p class="audio-name">{{ audioStore.currentAudio?.name }}</p>
            <p class="audio-duration text-sm text-gray-500">
              时长: {{ audioStore.formatTime(audioStore.playerState.duration) }}
            </p>
          </div>

          <!-- 播放控制 -->
          <div class="player-controls">
            <el-button
              :icon="audioStore.playerState.isPlaying ? 'VideoPause' : 'VideoPlay'"
              circle
              size="large"
              type="primary"
              @click="audioStore.togglePlay()"
            />
            <el-button :icon="'RefreshLeft'" circle @click="audioStore.seek(0)" />
            <div class="volume-control">
              <i class="i-mdi-volume-high"></i>
              <el-slider
                v-model="audioStore.playerState.volume"
                :max="1"
                :step="0.01"
                :show-tooltip="false"
                class="ml-2"
                style="width: 100px"
                @input="(val: number) => audioStore.setVolume(val)"
              />
            </div>
          </div>

          <!-- 进度条 -->
          <div class="progress-bar">
            <span class="time-label">
              {{ audioStore.formatTime(audioStore.playerState.currentTime) }}
            </span>
            <el-slider
              :model-value="audioStore.playerState.currentTime"
              :max="audioStore.playerState.duration"
              :step="0.1"
              :show-tooltip="false"
              @change="(val: number) => audioStore.seek(val)"
            />
            <span class="time-label">
              {{ audioStore.formatTime(audioStore.playerState.duration) }}
            </span>
          </div>

          <!-- 播放速度 -->
          <div class="playback-rate">
            <span class="text-sm text-gray-600 mr-2">播放速度:</span>
            <el-radio-group
              v-model="audioStore.playerState.playbackRate"
              size="small"
              @change="(val: number) => audioStore.setPlaybackRate(val)"
            >
              <el-radio-button :value="0.5">0.5x</el-radio-button>
              <el-radio-button :value="0.75">0.75x</el-radio-button>
              <el-radio-button :value="1">1x</el-radio-button>
              <el-radio-button :value="1.25">1.25x</el-radio-button>
              <el-radio-button :value="1.5">1.5x</el-radio-button>
            </el-radio-group>
          </div>

          <!-- 时间轴可视化区域（占位） -->
          <div class="timeline-placeholder">
            <p class="text-gray-400 text-sm">时间轴可视化（待实现）</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.editor-page {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f5f5f5;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: white;
  border-bottom: 1px solid #e5e7eb;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toolbar-center {
  flex: 1;
  text-align: center;
  font-weight: 500;
}

.file-name {
  color: #333;
}

/* 内容区 */
.content-area {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  padding: 1rem;
  overflow: hidden;
}

/* 面板 */
.subtitle-panel,
.player-panel {
  background: white;
  border-radius: 0.5rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  border-bottom: 1px solid #e5e7eb;
}

.panel-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: #333;
}

.panel-footer {
  padding: 0.75rem 1rem;
  border-top: 1px solid #e5e7eb;
  background-color: #f9fafb;
}

/* 搜索面板 */
.search-panel {
  padding: 0.75rem 1rem;
  background-color: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
}

/* 字幕列表 */
.subtitle-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.subtitle-item {
  padding: 0.75rem;
  margin-bottom: 0.5rem;
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 0.375rem;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.subtitle-item:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
}

.subtitle-item.is-selected {
  background: #eef2ff;
  border-color: #818cf8;
}

.subtitle-item.has-conflict {
  border-color: #fca5a5;
  background: #fef2f2;
}

.item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.item-number {
  font-size: 0.75rem;
  font-weight: 600;
  color: #6b7280;
}

.item-time {
  font-size: 0.75rem;
  color: #9ca3af;
  font-family: monospace;
}

.item-text {
  color: #333;
  line-height: 1.5;
  margin-bottom: 0.5rem;
}

.item-actions {
  display: flex;
  gap: 0.25rem;
  opacity: 0;
  transition: opacity 0.2s;
}

.subtitle-item:hover .item-actions {
  opacity: 1;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
}

/* 音频播放器 */
.audio-player {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.audio-info {
  text-align: center;
}

.audio-name {
  font-weight: 600;
  color: #333;
  margin-bottom: 0.25rem;
}

.player-controls {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
}

.volume-control {
  display: flex;
  align-items: center;
  margin-left: 1rem;
}

.progress-bar {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.time-label {
  font-size: 0.875rem;
  color: #6b7280;
  font-family: monospace;
  min-width: 50px;
}

.playback-rate {
  display: flex;
  align-items: center;
  justify-content: center;
}

.timeline-placeholder {
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f9fafb;
  border: 2px dashed #e5e7eb;
  border-radius: 0.375rem;
}
</style>
