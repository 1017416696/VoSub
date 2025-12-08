<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useSubtitleStore } from '@/stores/subtitle'
import { useAudioStore } from '@/stores/audio'
import { useConfigStore } from '@/stores/config'
import type { SRTFile, AudioFile, SubtitleEntry } from '@/types/subtitle'

interface WhisperModelInfo {
  name: string
  size: string
  downloaded: boolean
  path?: string
}

interface TranscriptionProgress {
  progress: number
  current_text: string
  status: string
}

const router = useRouter()
const subtitleStore = useSubtitleStore()
const audioStore = useAudioStore()
const configStore = useConfigStore()

const isDragging = ref(false)
const isLoading = ref(false)
const loadingMessage = ref('')

const showTranscriptionDialog = ref(false)
const availableModels = ref<WhisperModelInfo[]>([])
const isTranscribing = ref(false)
const transcriptionProgress = ref(0)
const transcriptionMessage = ref('')

// 已下载的模型
const downloadedModels = computed(() => availableModels.value.filter(m => m.downloaded))

// 当前选中的模型显示名称
const currentModelName = computed(() => {
  const model = availableModels.value.find(m => m.name === configStore.whisperModel)
  if (model?.downloaded) return model.name
  // 如果默认模型未下载，显示第一个已下载的模型或 base
  const firstDownloaded = downloadedModels.value[0]
  return firstDownloaded?.name || 'base'
})



let unlistenFileDrop: (() => void) | null = null

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow()
  const unlistenHover = await appWindow.onDragDropEvent((event) => {
    if (event.payload.type === 'over') isDragging.value = true
    else if (event.payload.type === 'leave') isDragging.value = false
    else if (event.payload.type === 'drop') { isDragging.value = false; handleFileDrop(event.payload.paths) }
  })
  unlistenFileDrop = unlistenHover

  const unlistenProgress = await listen<TranscriptionProgress>('transcription-progress', (event) => {
    transcriptionProgress.value = event.payload.progress
    transcriptionMessage.value = event.payload.current_text
  })
  onUnmounted(() => unlistenProgress())

  try { availableModels.value = await invoke<WhisperModelInfo[]>('get_whisper_models') } catch (e) { console.error(e) }
})

onUnmounted(() => { if (unlistenFileDrop) unlistenFileDrop() })

const handleFileDrop = async (paths: string[]) => {
  if (!paths || paths.length === 0) return
  const srtFile = paths.find((p) => p.toLowerCase().endsWith('.srt'))
  const audioFile = paths.find((p) => /\.(mp3|wav|ogg|flac|m4a|aac)$/i.test(p.toLowerCase()))
  if (!srtFile && !audioFile) {
    await ElMessageBox.alert('请拖放有效的 SRT 字幕文件或音频文件', '无效文件', { confirmButtonText: '确定', type: 'warning' })
    return
  }
  await processFiles({ srtPath: srtFile, audioPath: audioFile })
}

const openSRTFile = async () => {
  try {
    const selected = await open({ multiple: false, filters: [{ name: 'SRT 字幕文件', extensions: ['srt'] }] })
    if (selected) await processFiles({ srtPath: selected as string })
  } catch (e) { await ElMessageBox.alert('无法打开文件选择器', '错误', { confirmButtonText: '确定', type: 'error' }) }
}

const processFiles = async ({ srtPath, audioPath }: { srtPath?: string; audioPath?: string }) => {
  isLoading.value = true
  let srtLoaded = false
  try {
    if (srtPath) {
      loadingMessage.value = '正在加载字幕文件...'
      const srtFile = await invoke<SRTFile>('read_srt', { filePath: srtPath })
      await subtitleStore.loadSRTFile(srtFile)
      srtLoaded = true
      configStore.addRecentFile(srtPath)
      if ((window as any).__updateRecentFilesMenu) await (window as any).__updateRecentFilesMenu()
    }
    if (audioPath) {
      loadingMessage.value = '正在加载音频文件...'
      const fileName = audioPath.split('/').pop() || 'audio'
      const fileExtension = audioPath.split('.').pop()?.toLowerCase() || 'mp3'
      await audioStore.loadAudio({ name: fileName, path: audioPath, duration: 0, format: fileExtension })
    }
    if (srtLoaded) { loadingMessage.value = '即将进入编辑器...'; setTimeout(() => router.push('/editor'), 500) }
  } catch (error) {
    await ElMessageBox.alert(`加载失败：${error instanceof Error ? error.message : '未知错误'}`, '错误', { confirmButtonText: '确定', type: 'error' })
  } finally { if (!srtLoaded) { isLoading.value = false; loadingMessage.value = '' } }
}

const formatRelativeTime = (timestamp: number): string => {
  const diff = Date.now() - timestamp
  const minutes = Math.floor(diff / 60000), hours = Math.floor(diff / 3600000), days = Math.floor(diff / 86400000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes} 分钟前`
  if (hours < 24) return `${hours} 小时前`
  if (days < 7) return `${days} 天前`
  const date = new Date(timestamp)
  return `${date.getMonth() + 1}/${date.getDate()}`
}

const openRecentFile = async (filePath: string) => {
  isLoading.value = true
  loadingMessage.value = '正在加载字幕文件...'
  try {
    const srtFile = await invoke<SRTFile>('read_srt', { filePath })
    await subtitleStore.loadSRTFile(srtFile)
    configStore.addRecentFile(filePath)
    if ((window as any).__updateRecentFilesMenu) await (window as any).__updateRecentFilesMenu()
    setTimeout(() => router.push('/editor'), 300)
  } catch (error) {
    isLoading.value = false; loadingMessage.value = ''
    await ElMessageBox.alert(`加载文件失败：${error instanceof Error ? error.message : '文件可能已被移动或删除'}`, '加载失败', { confirmButtonText: '确定', type: 'error' })
  }
}

let lastClickTime = 0
const onTitlebarMousedown = async (e: MouseEvent) => {
  if (e.button === 0) {
    const now = Date.now()
    if (now - lastClickTime < 300) { await onTitlebarDoubleClick(); return }
    lastClickTime = now
    e.preventDefault()
    try { await getCurrentWindow().startDragging() } catch {}
  }
}

const onTitlebarDoubleClick = async () => {
  const window = getCurrentWindow()
  if (await window.isMaximized()) await window.unmaximize()
  else await window.maximize()
}

const startTranscription = async () => {
  try {
    // 先选择音频文件
    const selected = await open({ multiple: false, filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'ogg', 'flac', 'm4a', 'aac'] }] })
    if (!selected || typeof selected !== 'string') return

    const modelName = configStore.whisperModel
    const model = availableModels.value.find(m => m.name === modelName)
    
    // 如果模型未下载，自动下载
    if (!model || !model.downloaded) {
      const targetModel = model || availableModels.value.find(m => m.name === 'base')!
      const confirm = await ElMessageBox.confirm(
        `模型 ${targetModel.name} (${targetModel.size}) 尚未下载，是否现在下载？`,
        '需要下载模型',
        { confirmButtonText: '下载', cancelButtonText: '取消', type: 'info' }
      ).catch(() => false)
      if (!confirm) return
      
      isTranscribing.value = true
      transcriptionProgress.value = 0
      transcriptionMessage.value = '正在下载模型...'
      showTranscriptionDialog.value = true
      try {
        await invoke('download_whisper_model', { modelSize: targetModel.name })
        availableModels.value = await invoke<WhisperModelInfo[]>('get_whisper_models')
        // 更新默认模型为刚下载的模型
        configStore.whisperModel = targetModel.name
        configStore.saveWhisperSettings()
      } catch (error) {
        isTranscribing.value = false
        showTranscriptionDialog.value = false
        await ElMessageBox.alert(`下载模型失败：${error instanceof Error ? error.message : '未知错误'}`, '下载失败', { confirmButtonText: '确定', type: 'error' })
        return
      }
    }

    isTranscribing.value = true
    transcriptionProgress.value = 0
    transcriptionMessage.value = '正在转录音频...'
    showTranscriptionDialog.value = true
    
    const entries = await invoke<SubtitleEntry[]>('transcribe_audio_to_subtitles', {
      audioPath: selected,
      modelSize: configStore.whisperModel,
      language: configStore.whisperLanguage,
    })
    const fileName = selected.split('/').pop() || 'transcription.srt'
    await subtitleStore.loadSRTFile({ name: fileName.replace(/\.[^.]+$/, '.srt'), path: '', entries, encoding: 'UTF-8' })
    isTranscribing.value = false
    showTranscriptionDialog.value = false
    ElMessage.success(`转录成功！生成了 ${entries.length} 条字幕`)
    setTimeout(() => router.push('/editor'), 500)
  } catch (error) {
    isTranscribing.value = false
    showTranscriptionDialog.value = false
    await ElMessageBox.alert(`转录失败：${error instanceof Error ? error.message : '未知错误'}`, '转录失败', { confirmButtonText: '确定', type: 'error' })
  }
}

const cancelTranscription = () => { showTranscriptionDialog.value = false; isTranscribing.value = false }

const onSelectModel = (modelName: string) => {
  configStore.whisperModel = modelName
  configStore.saveWhisperSettings()
}
</script>

<template>
  <div class="welcome-page" :class="{ 'is-dragging': isDragging }">
    <div class="titlebar" @mousedown.left="onTitlebarMousedown" @dblclick="onTitlebarDoubleClick">
      <span class="titlebar-title">SRT 字幕编辑器</span>
    </div>

    <div class="welcome-content">
      <div class="main-section">
        <!-- 品牌区域 -->
        <div class="brand-area">
          <div class="brand-icon">
            <i class="i-mdi-subtitles-outline"></i>
          </div>
          <div class="brand-text">
            <h1 class="brand-title">SRT 字幕编辑器</h1>
            <p class="brand-desc">专业的字幕编辑工具，支持音频同步和批量操作</p>
          </div>
        </div>

        <!-- 开始使用 -->
        <div class="get-started">
          <p class="section-title">开始使用</p>
          <div class="action-buttons">
            <button class="primary-btn" :disabled="isLoading" @click="openSRTFile">
              <span v-if="!isLoading">打开字幕文件</span>
              <span v-else>{{ loadingMessage }}</span>
            </button>
            <div class="transcription-btn-group">
              <button class="transcription-btn" :disabled="isLoading" @click="startTranscription">
                <i class="i-mdi-microphone"></i>
                <span>AI 语音转录</span>
                <span class="model-badge">{{ currentModelName }}</span>
              </button>
              <el-dropdown v-if="downloadedModels.length > 1" trigger="click" @command="onSelectModel">
                <button class="transcription-dropdown" :disabled="isLoading">
                  <i class="i-mdi-chevron-down"></i>
                </button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item
                      v-for="model in downloadedModels"
                      :key="model.name"
                      :command="model.name"
                      :class="{ 'is-active': model.name === configStore.whisperModel }"
                    >
                      {{ model.name }}
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </div>
        </div>

        <!-- 最近文件 -->
        <div v-if="configStore.recentFiles.length > 0" class="recent-section">
          <p class="section-title">最近打开</p>
          <div class="recent-list">
            <div v-for="file in configStore.recentFiles.slice(0, 5)" :key="file.path" class="recent-item" @click="openRecentFile(file.path)">
              <i class="i-mdi-file-document-outline file-icon"></i>
              <span class="recent-name">{{ file.name }}</span>
              <span class="recent-time">{{ formatRelativeTime(file.lastOpened) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 拖放提示遮罩 -->
    <div v-if="isDragging" class="drag-overlay">
      <div class="drag-hint">
        <i class="i-mdi-file-upload-outline"></i>
        <p>释放以打开文件</p>
      </div>
    </div>

    <!-- 转录进度对话框 -->
    <el-dialog v-model="showTranscriptionDialog" title="音频转录" :close-on-click-modal="false" :close-on-press-escape="false" :show-close="!isTranscribing" width="400px">
      <div v-if="isTranscribing" class="progress-content">
        <el-progress :percentage="Math.round(transcriptionProgress)" :stroke-width="10" />
        <p class="progress-message">{{ transcriptionMessage }}</p>
      </div>
      <template #footer>
        <el-button v-if="!isTranscribing" @click="cancelTranscription">取消</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.welcome-page {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #e8e8e8;
  overflow: hidden;
  position: relative;
}

.welcome-page.is-dragging {
  background: #ddd;
}

/* 标题栏 */
.titlebar {
  height: 38px;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  -webkit-app-region: drag;
  user-select: none;
}

.titlebar-title {
  font-size: 13px;
  font-weight: 500;
  color: #666;
}

/* 主内容区 */
.welcome-content {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 2rem;
}

.main-section {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  width: 340px;
}

/* 品牌区域 */
.brand-area {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
}

.brand-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.brand-icon i {
  font-size: 42px;
  color: #333;
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.brand-title {
  font-size: 22px;
  font-weight: 700;
  color: #222;
  margin: 0;
  letter-spacing: -0.3px;
}

.brand-desc {
  font-size: 13px;
  color: #888;
  margin: 0;
}

/* 区块标题 */
.section-title {
  font-size: 13px;
  font-weight: 500;
  color: #555;
  margin: 0 0 0.625rem;
}

/* 操作按钮组 */
.action-buttons {
  display: flex;
  gap: 0.75rem;
}

/* 主按钮 */
.primary-btn {
  flex: 0 0 auto;
  padding: 0.625rem 1.25rem;
  background: #409eff;
  color: white;
  font-size: 13px;
  font-weight: 500;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.primary-btn:hover:not(:disabled) {
  background: #337ecc;
}

.primary-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

/* 次要按钮 */
.secondary-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.875rem 1rem;
  background: rgba(64, 158, 255, 0.08);
  color: #666;
  font-size: 14px;
  font-weight: 500;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.secondary-btn:hover:not(:disabled) {
  background: rgba(64, 158, 255, 0.15);
  color: #409eff;
}

.secondary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.secondary-btn i {
  font-size: 18px;
}

/* 转录按钮组 */
.transcription-btn-group {
  flex: 1;
  display: flex;
}

.transcription-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 0.625rem 0.75rem;
  background: rgba(64, 158, 255, 0.08);
  color: #666;
  font-size: 13px;
  font-weight: 500;
  border: none;
  border-radius: 6px 0 0 6px;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.transcription-btn:hover:not(:disabled) {
  background: rgba(64, 158, 255, 0.15);
  color: #409eff;
}

.transcription-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.transcription-btn i {
  font-size: 16px;
}

.model-badge {
  font-size: 10px;
  padding: 1px 5px;
  background: rgba(64, 158, 255, 0.15);
  color: #409eff;
  border-radius: 3px;
}

.transcription-dropdown {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  background: rgba(64, 158, 255, 0.08);
  color: #666;
  border: none;
  border-left: 1px solid rgba(64, 158, 255, 0.15);
  border-radius: 0 6px 6px 0;
  cursor: pointer;
  transition: all 0.15s ease;
}

.transcription-dropdown:hover:not(:disabled) {
  background: rgba(64, 158, 255, 0.15);
  color: #409eff;
}

.transcription-dropdown:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.transcription-dropdown i {
  font-size: 14px;
}

/* 只有一个模型时，按钮圆角完整 */
.transcription-btn-group .transcription-btn:only-child {
  border-radius: 6px;
}

/* 最近文件 */
.recent-section {
  margin-top: 0.25rem;
}

.recent-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 0.75rem;
  background: rgba(255, 255, 255, 0.6);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.recent-item:hover {
  background: rgba(255, 255, 255, 0.9);
}

.recent-item .file-icon {
  font-size: 20px;
  color: #409eff;
  flex-shrink: 0;
}

.recent-name {
  flex: 1;
  font-size: 13px;
  color: #333;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-time {
  font-size: 12px;
  color: #999;
  flex-shrink: 0;
}



/* 拖放遮罩 */
.drag-overlay {
  position: absolute;
  inset: 0;
  background: rgba(64, 158, 255, 0.06);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.drag-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  padding: 2rem 3rem;
  background: white;
  border-radius: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12);
}

.drag-hint i {
  font-size: 48px;
  color: #409eff;
}

.drag-hint p {
  font-size: 15px;
  font-weight: 500;
  color: #333;
  margin: 0;
}

/* 转录对话框 */
.progress-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem 0;
}

.progress-message {
  font-size: 13px;
  color: #666;
  text-align: center;
  margin: 0;
}
</style>
