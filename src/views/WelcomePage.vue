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
import { useSmartDictionaryStore } from '@/stores/smartDictionary'
import WelcomeGuide from '@/components/WelcomeGuide.vue'
import type { SRTFile, AudioFile, SubtitleEntry } from '@/types/subtitle'

interface WhisperModelInfo {
  name: string
  size: string
  downloaded: boolean
  path?: string
  partial_size?: number
}

interface TranscriptionProgress {
  progress: number
  current_text: string
  status: string
}

interface SenseVoiceEnvStatus {
  uv_installed: boolean
  env_exists: boolean
  ready: boolean
}

const router = useRouter()
const subtitleStore = useSubtitleStore()
const audioStore = useAudioStore()
const configStore = useConfigStore()
const smartDictionary = useSmartDictionaryStore()

// 🎄 圣诞季节判断
const isChristmasSeason = computed(() => {
  const now = new Date()
  const month = now.getMonth() + 1
  const day = now.getDate()
  return month === 12 && day >= 20 && day <= 31
})

const isDragging = ref(false)
const isLoading = ref(false)
const loadingMessage = ref('')

// 新手引导
const showWelcomeGuide = ref(false)

const showTranscriptionDialog = ref(false)
const availableModels = ref<WhisperModelInfo[]>([])
const isTranscribing = ref(false)
const transcriptionProgress = ref(0)
const transcriptionMessage = ref('')
const isCancelled = ref(false)
const isTransitioningToEditor = ref(false)

// SenseVoice 相关状态
const sensevoiceEnvStatus = ref<SenseVoiceEnvStatus>({ uv_installed: false, env_exists: false, ready: false })
const isInstallingSensevoice = ref(false)

// 已下载的模型
const downloadedModels = computed(() => availableModels.value.filter(m => m.downloaded))

// 当前选中的模型显示名称
const currentModelName = computed(() => {
  if (configStore.transcriptionEngine === 'sensevoice') {
    return 'SenseVoice Small'
  }
  const model = availableModels.value.find(m => m.name === configStore.whisperModel)
  if (model?.downloaded) return `Whisper ${model.name}`
  // 如果默认模型未下载，显示第一个已下载的模型或 tiny（首次使用默认）
  const firstDownloaded = downloadedModels.value[0]
  return firstDownloaded ? `Whisper ${firstDownloaded.name}` : 'Whisper tiny'
})

// 是否显示引擎切换下拉
const showEngineDropdown = computed(() => {
  return downloadedModels.value.length > 0 || sensevoiceEnvStatus.value.ready
})

// 是否显示下载进度条（下载模型时）
const isDownloading = computed(() => {
  return transcriptionMessage.value.includes('Downloading') || 
         (transcriptionMessage.value.includes('下载') && !transcriptionMessage.value.includes('首次'))
})

// 是否正在使用 SenseVoice 转录（现在也显示进度条）
const isSensevoiceTranscribing = computed(() => {
  return configStore.transcriptionEngine === 'sensevoice' && isTranscribing.value && !isInstallingSensevoice.value
})

// 是否显示真实进度条（转录时都显示）
const showRealProgress = computed(() => {
  return isTranscribing.value && !isInstallingSensevoice.value
})

// 本地化消息
const localizedMessage = computed(() => {
  const msg = transcriptionMessage.value
  if (!msg) return '准备中...'
  
  // 英文消息映射到中文
  const messageMap: Record<string, string> = {
    'Loading audio file...': '正在加载音频文件...',
    'Loading Whisper model...': '正在加载语音模型...',
    'Transcribing audio...': '正在识别语音内容...',
    'Converting to subtitles...': '正在生成字幕...',
  }
  
  // 检查是否包含下载进度
  if (msg.includes('Downloading')) {
    const match = msg.match(/Downloading (\w+) model\.\.\. ([\d.]+)%/)
    if (match) {
      return `正在下载 ${match[1]} 模型... ${match[2]}%`
    }
    return '正在下载模型...'
  }
  
  // 检查完成消息
  if (msg.includes('completed') || msg.includes('Generated')) {
    const match = msg.match(/Generated (\d+) subtitles/)
    if (match) {
      return `转录完成！生成了 ${match[1]} 条字幕`
    }
    return '转录完成！'
  }
  
  return messageMap[msg] || msg
})



let unlistenFileDrop: (() => void) | null = null

let unlistenTranscriptionProgress: (() => void) | null = null
let unlistenModelDownloadProgress: (() => void) | null = null

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow()
  const unlistenHover = await appWindow.onDragDropEvent((event) => {
    if (event.payload.type === 'over') isDragging.value = true
    else if (event.payload.type === 'leave') isDragging.value = false
    else if (event.payload.type === 'drop') { isDragging.value = false; handleFileDrop(event.payload.paths) }
  })
  unlistenFileDrop = unlistenHover

  // 监听转录进度
  unlistenTranscriptionProgress = await listen<TranscriptionProgress>('transcription-progress', (event) => {
    // 如果已取消，忽略后续进度更新
    if (isCancelled.value) return
    transcriptionProgress.value = event.payload.progress
    transcriptionMessage.value = event.payload.current_text
    // 更新 tray 进度
    invoke('update_tray_progress', { progress: event.payload.progress }).catch(console.error)
  })

  // 监听模型下载进度（单独事件，避免与转录进度冲突）
  unlistenModelDownloadProgress = await listen<TranscriptionProgress>('model-download-progress', (event) => {
    // 如果已取消，忽略后续进度更新
    if (isCancelled.value) return
    transcriptionProgress.value = event.payload.progress
    transcriptionMessage.value = event.payload.current_text
    // 更新 tray 进度
    invoke('update_tray_progress', { progress: event.payload.progress }).catch(console.error)
  })

  try { availableModels.value = await invoke<WhisperModelInfo[]>('get_whisper_models_cmd') } catch (e) { console.error(e) }
  // 检查 SenseVoice 环境
  try { sensevoiceEnvStatus.value = await invoke<SenseVoiceEnvStatus>('check_sensevoice_env_status') } catch (e) { console.error(e) }

  // 首次使用显示新手引导
  if (!configStore.onboardingState.welcomePageSeen) {
    showWelcomeGuide.value = true
  }
})

// 关闭欢迎页引导
const closeWelcomeGuide = () => {
  showWelcomeGuide.value = false
  configStore.markWelcomePageSeen()
}

onUnmounted(() => {
  if (unlistenFileDrop) unlistenFileDrop()
  if (unlistenTranscriptionProgress) unlistenTranscriptionProgress()
  if (unlistenModelDownloadProgress) unlistenModelDownloadProgress()
})

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
      loadingMessage.value = '正在检查文件权限...'
      // 检查文件写入权限
      const permissionCheck = await invoke<{ readable: boolean; writable: boolean; error_message: string | null; is_locked: boolean }>('check_file_write_permission', { filePath: srtPath })
      
      if (!permissionCheck.writable) {
        if (permissionCheck.is_locked) {
          // 文件被锁定，提供解锁选项
          try {
            await ElMessageBox.confirm(
              '文件已被锁定，无法写入。\n\n点击「解锁」按钮可以解除锁定并继续编辑。',
              '文件已锁定',
              { confirmButtonText: '解锁', cancelButtonText: '取消', type: 'warning' }
            )
            // 用户点击解锁
            await invoke('unlock_file_cmd', { filePath: srtPath })
          } catch {
            // 用户点击取消
            isLoading.value = false
            loadingMessage.value = ''
            return
          }
        } else {
          // 其他权限问题
          const warningMessage = permissionCheck.error_message || '文件无法写入。'
          await ElMessageBox.alert(warningMessage, '无法打开文件', { confirmButtonText: '我知道了', type: 'warning', dangerouslyUseHTMLString: true })
          isLoading.value = false
          loadingMessage.value = ''
          return
        }
      }
      
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
    if (srtLoaded) { router.push('/editor') }
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
  loadingMessage.value = '正在检查文件权限...'
  try {
    // 检查文件写入权限
    const permissionCheck = await invoke<{ readable: boolean; writable: boolean; error_message: string | null; is_locked: boolean }>('check_file_write_permission', { filePath })
    
    if (!permissionCheck.writable) {
      if (permissionCheck.is_locked) {
        // 文件被锁定，提供解锁选项
        try {
          await ElMessageBox.confirm(
            '文件已被锁定，无法写入。\n\n点击「解锁」按钮可以解除锁定并继续编辑。',
            '文件已锁定',
            { confirmButtonText: '解锁', cancelButtonText: '取消', type: 'warning' }
          )
          // 用户点击解锁
          await invoke('unlock_file_cmd', { filePath })
        } catch {
          // 用户点击取消
          isLoading.value = false
          loadingMessage.value = ''
          return
        }
      } else {
        // 其他权限问题
        const warningMessage = permissionCheck.error_message || '文件无法写入。'
        await ElMessageBox.alert(warningMessage, '无法打开文件', { confirmButtonText: '我知道了', type: 'warning', dangerouslyUseHTMLString: true })
        isLoading.value = false
        loadingMessage.value = ''
        return
      }
    }
    
    loadingMessage.value = '正在加载字幕文件...'
    const srtFile = await invoke<SRTFile>('read_srt', { filePath })
    await subtitleStore.loadSRTFile(srtFile)
    configStore.addRecentFile(filePath)
    if ((window as any).__updateRecentFilesMenu) await (window as any).__updateRecentFilesMenu()
    router.push('/editor')
  } catch (error) {
    isLoading.value = false; loadingMessage.value = ''
    await ElMessageBox.alert(`加载文件失败：${error instanceof Error ? error.message : '文件可能已被移动或删除'}`, '加载失败', { confirmButtonText: '确定', type: 'error' })
    // 文件加载失败后自动从最近列表中删除
    configStore.removeRecentFile(filePath)
    if ((window as any).__updateRecentFilesMenu) await (window as any).__updateRecentFilesMenu()
  }
}

const removeRecentFile = (filePath: string, event: MouseEvent) => {
  event.stopPropagation()
  configStore.removeRecentFile(filePath)
  if ((window as any).__updateRecentFilesMenu) (window as any).__updateRecentFilesMenu()
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

// 格式化转录错误消息
const formatTranscriptionError = (error: string): string => {
  const lowerError = error.toLowerCase()
  
  // 网络连接错误
  if (lowerError.includes('connection reset') || lowerError.includes('errno 54')) {
    return '网络连接被中断，请检查网络连接后重试。如果问题持续，可能需要使用代理或VPN。'
  }
  if (lowerError.includes('connection refused') || lowerError.includes('errno 61')) {
    return '无法连接到服务器，请检查网络连接。'
  }
  if (lowerError.includes('timeout') || lowerError.includes('timed out')) {
    return '连接超时，请检查网络连接后重试。'
  }
  if (lowerError.includes('dns') || lowerError.includes('resolve')) {
    return '无法解析服务器地址，请检查网络设置或DNS配置。'
  }
  if (lowerError.includes('ssl') || lowerError.includes('certificate')) {
    return '安全连接失败，请检查系统时间或网络代理设置。'
  }
  
  // 模型下载相关错误
  if (lowerError.includes('model') && (lowerError.includes('not found') || lowerError.includes('404'))) {
    return '模型文件不存在，请稍后重试或选择其他模型。'
  }
  if (lowerError.includes('download') && lowerError.includes('failed')) {
    return '模型下载失败，请检查网络连接后重试。'
  }
  
  // 如果包含"转录失败:"前缀，提取后面的内容（避免重复）
  if (error.includes('转录失败:')) {
    const parts = error.split('转录失败:')
    if (parts.length > 1) {
      return parts.slice(1).join('转录失败:').trim()
    }
  }
  
  // 返回原始错误信息（Rust后端已经清理过ANSI转义码和进度条格式）
  return error.trim() || '未知错误，请查看日志获取详细信息。'
}

const startTranscription = async () => {
  try {
    // 先选择音频文件
    const selected = await open({ multiple: false, filters: [{ name: '音频文件', extensions: ['mp3', 'wav', 'ogg', 'flac', 'm4a', 'aac'] }] })
    if (!selected || typeof selected !== 'string') return

    // 根据引擎类型处理
    if (configStore.transcriptionEngine === 'sensevoice') {
      await startSensevoiceTranscription(selected)
    } else {
      await startWhisperTranscription(selected)
    }
  } catch (error) {
    // 隐藏 tray 进度
    await invoke('hide_tray_progress').catch(console.error)
    
    isTranscribing.value = false
    showTranscriptionDialog.value = false
    if (isCancelled.value) return
    const errorMsg = error instanceof Error ? error.message : String(error)
    if (errorMsg.includes('取消') || errorMsg.includes('cancel')) return
    
    // 格式化错误消息，清理ANSI转义码和进度条格式
    const formattedError = formatTranscriptionError(errorMsg)
    await ElMessageBox.alert(`转录失败：${formattedError}`, '转录失败', { confirmButtonText: '确定', type: 'error' })
  }
}

// Whisper 转录
const startWhisperTranscription = async (audioPath: string) => {
  // 检查 Whisper 环境是否就绪
  interface WhisperEnvStatus {
    uv_installed: boolean
    ready: boolean
  }
  
  let whisperEnvStatus: WhisperEnvStatus
  try {
    whisperEnvStatus = await invoke<WhisperEnvStatus>('check_whisper_env_status')
  } catch (e) {
    await ElMessageBox.alert('无法检查 Whisper 环境状态', '错误', { confirmButtonText: '确定', type: 'error' })
    return
  }
  
  if (!whisperEnvStatus.ready) {
    // 首次使用，检查 uv 是否安装
    if (!whisperEnvStatus.uv_installed) {
      await ElMessageBox.alert(
        '首次使用需要安装 uv 包管理器。\n\n安装命令：\nmacOS/Linux: curl -LsSf https://astral.sh/uv/install.sh | sh\nWindows: powershell -c "irm https://astral.sh/uv/install.ps1 | iex"\n\n安装后请重启应用。',
        '需要安装 uv',
        { confirmButtonText: '确定', type: 'warning' }
      )
      return
    }

    // 检测是否使用 GPU：macOS 不支持 CUDA，Windows 才尝试 GPU
    const isMacOS = navigator.platform.toLowerCase().includes('mac')
    const useGpu = !isMacOS // Windows 默认尝试 GPU 版本

    // 自动安装 Whisper 环境
    isTranscribing.value = true
    isCancelled.value = false
    transcriptionProgress.value = 0
    transcriptionMessage.value = `首次使用，正在安装 Whisper ${useGpu ? 'GPU' : 'CPU'} 环境...`
    showTranscriptionDialog.value = true

    // 监听安装进度
    const unlistenInstall = await listen<{ progress: number; current_text: string }>('whisper-progress', (event) => {
      transcriptionProgress.value = event.payload.progress
      transcriptionMessage.value = event.payload.current_text
    })

    try {
      await invoke('install_whisper', { useGpu })
      // 刷新状态
      whisperEnvStatus = await invoke<WhisperEnvStatus>('check_whisper_env_status')
    } catch (error) {
      unlistenInstall()
      isTranscribing.value = false
      showTranscriptionDialog.value = false
      const errorMsg = error instanceof Error ? error.message : String(error)
      await ElMessageBox.alert(`环境安装失败：${errorMsg}`, '安装失败', { confirmButtonText: '确定', type: 'error' })
      return
    }
    unlistenInstall()

    if (isCancelled.value) return
  }

  isTranscribing.value = true
  isCancelled.value = false
  transcriptionProgress.value = 0
  // 首次使用默认 tiny 模型
  const modelToUse = downloadedModels.value.length > 0 ? configStore.whisperModel : 'tiny'
  transcriptionMessage.value = '正在转录音频（首次使用模型时会自动下载）...'
  showTranscriptionDialog.value = true
  
  // 显示 tray 进度
  await invoke('show_tray_progress').catch(console.error)
  
  const entries = await invoke<SubtitleEntry[]>('transcribe_audio_to_subtitles', {
    audioPath,
    modelSize: modelToUse,
    language: configStore.whisperLanguage,
  })
  
  if (isCancelled.value) return
  await finishTranscription(audioPath, entries)
}

// SenseVoice 转录
const startSensevoiceTranscription = async (audioPath: string) => {
  // 检查环境
  sensevoiceEnvStatus.value = await invoke<SenseVoiceEnvStatus>('check_sensevoice_env_status')
  
  if (!sensevoiceEnvStatus.value.ready) {
    // 需要安装环境
    const confirm = await ElMessageBox.confirm(
      'SenseVoice 环境尚未安装，需要下载约 2-3GB 的依赖。是否现在安装？',
      '需要安装环境',
      { confirmButtonText: '安装', cancelButtonText: '取消', type: 'info' }
    ).catch(() => false)
    if (!confirm) return
    
    if (!sensevoiceEnvStatus.value.uv_installed) {
      await ElMessageBox.alert(
        '请先安装 uv 包管理器。\n\n安装命令：\nmacOS/Linux: curl -LsSf https://astral.sh/uv/install.sh | sh\nWindows: powershell -c "irm https://astral.sh/uv/install.ps1 | iex"',
        '需要安装 uv',
        { confirmButtonText: '确定', type: 'warning' }
      )
      return
    }
    
    // 安装环境
    isInstallingSensevoice.value = true
    isTranscribing.value = true
    transcriptionProgress.value = 0
    transcriptionMessage.value = '正在安装 SenseVoice 环境...'
    showTranscriptionDialog.value = true
    
    // 监听安装进度
    const unlistenInstall = await listen<TranscriptionProgress>('sensevoice-progress', (event) => {
      transcriptionProgress.value = event.payload.progress
      transcriptionMessage.value = event.payload.current_text
    })
    
    try {
      await invoke('install_sensevoice')
      sensevoiceEnvStatus.value = await invoke<SenseVoiceEnvStatus>('check_sensevoice_env_status')
    } catch (error) {
      isTranscribing.value = false
      isInstallingSensevoice.value = false
      showTranscriptionDialog.value = false
      unlistenInstall()
      await ElMessageBox.alert(`安装失败：${error instanceof Error ? error.message : '未知错误'}`, '安装失败', { confirmButtonText: '确定', type: 'error' })
      return
    }
    unlistenInstall()
    isInstallingSensevoice.value = false
  }

  isTranscribing.value = true
  isCancelled.value = false
  transcriptionProgress.value = 0
  transcriptionMessage.value = '正在转录音频...'
  showTranscriptionDialog.value = true
  
  // 显示 tray 进度
  await invoke('show_tray_progress').catch(console.error)
  
  const entries = await invoke<SubtitleEntry[]>('transcribe_with_sensevoice_model', {
    audioPath,
    language: configStore.whisperLanguage,
  })
  
  if (isCancelled.value) return
  await finishTranscription(audioPath, entries)
}

// 完成转录，跳转编辑器
const finishTranscription = async (audioPath: string, entries: SubtitleEntry[]) => {
  // 🔥 转录完成后，立即应用智能词典
  if (smartDictionary.totalCount > 0) {
    let replacementCount = 0
    for (const entry of entries) {
      const { result, replacements } = smartDictionary.applyDictionary(entry.text)
      if (replacements.length > 0) {
        entry.text = result
        replacementCount += replacements.length
      }
    }
    if (replacementCount > 0) {
      console.log(`智能词典替换了 ${replacementCount} 处`)
    }
  }

  // 兼容 Windows 和 Unix 路径分隔符
  const fileName = audioPath.split(/[/\\]/).pop() || 'transcription.srt'
  const srtFileName = fileName.replace(/\.[^.]+$/, '.srt')
  // 生成与音频文件同目录的 srt 文件路径
  let srtPath = audioPath.replace(/\.[^.]+$/, '.srt')

  // 检查文件是否已存在
  const fileExists = await invoke<boolean>('check_file_exists', { filePath: srtPath })
  if (fileExists) {
    // 暂时隐藏转录对话框以显示确认框
    showTranscriptionDialog.value = false

    try {
      await ElMessageBox.confirm(
        `文件 "${srtFileName}" 已存在，是否覆盖？`,
        '文件已存在',
        {
          confirmButtonText: '覆盖',
          cancelButtonText: '另存为',
          distinguishCancelAndClose: true,
          type: 'warning',
        }
      )
      // 用户选择覆盖，继续使用原路径
    } catch (action) {
      if (action === 'cancel') {
        // 用户选择另存为
        const { save } = await import('@tauri-apps/plugin-dialog')
        const newPath = await save({
          filters: [{ name: 'SRT 字幕文件', extensions: ['srt'] }],
          defaultPath: srtPath,
        })
        if (newPath) {
          srtPath = newPath
        } else {
          // 用户取消了另存为，不保存文件但继续进入编辑器
          srtPath = ''
        }
      } else {
        // 用户关闭了对话框，不保存文件但继续进入编辑器
        srtPath = ''
      }
    }
  }

  await subtitleStore.loadSRTFile({
    name: srtFileName,
    path: srtPath,
    entries,
    encoding: 'UTF-8',
  })

  // 自动保存字幕文件（如果有路径）
  if (srtPath) {
    try {
      await subtitleStore.saveToFile()
    } catch (error) {
      console.error('自动保存字幕文件失败:', error)
    }
  }

  const fileExtension = audioPath.split('.').pop()?.toLowerCase() || 'mp3'
  await audioStore.loadAudio({ name: fileName, path: audioPath, duration: 0, format: fileExtension })

  // 隐藏 tray 进度
  await invoke('hide_tray_progress').catch(console.error)

  // 关闭对话框，直接跳转编辑器
  showTranscriptionDialog.value = false
  isTranscribing.value = false
  isLoading.value = false
  loadingMessage.value = ''
  router.push('/editor')
}

const cancelTranscription = async () => {
  isCancelled.value = true
  try {
    if (configStore.transcriptionEngine === 'sensevoice') {
      await invoke('cancel_sensevoice_task')
    } else {
      await invoke('cancel_whisper_task')
    }
  } catch (e) {
    console.error('取消转录失败:', e)
  }
  
  // 隐藏 tray 进度
  await invoke('hide_tray_progress').catch(console.error)
  
  showTranscriptionDialog.value = false
  isTranscribing.value = false
  isTransitioningToEditor.value = false
  isInstallingSensevoice.value = false
}

const onSelectModel = (command: string) => {
  if (command === 'sensevoice') {
    configStore.transcriptionEngine = 'sensevoice'
  } else {
    configStore.transcriptionEngine = 'whisper'
    configStore.whisperModel = command
  }
  configStore.saveWhisperSettings()
}

// Whisper 模型描述
const getWhisperModelDesc = (modelName: string): string => {
  const descMap: Record<string, string> = {
    'tiny': '最快速度',
    'base': '速度与精度平衡',
    'small': '较高精度',
    'medium': '高精度',
    'large-v2': '最高精度',
    'large-v3': '最新最高精度',
  }
  return descMap[modelName] || ''
}
</script>

<template>
  <div class="welcome-page" :class="{ 'is-dragging': isDragging }">
    <div class="titlebar" @mousedown.left="onTitlebarMousedown" @dblclick="onTitlebarDoubleClick">
      <span class="titlebar-title">VoSub</span>
    </div>

    <div class="welcome-content">
      <div class="main-section">
        <!-- 品牌区域 -->
        <div class="brand-area">
          <div class="brand-icon">
            <i class="i-mdi-subtitles-outline"></i>
          </div>
          <div class="brand-text">
            <h1 class="brand-title">VoSub</h1>
            <p class="brand-desc">专业的字幕编辑工具，支持音频同步和批量操作</p>
          </div>
        </div>

        <!-- 开始使用 -->
        <div class="get-started">
          <p class="section-title">开始使用</p>
          <div class="action-buttons">
            <button class="primary-btn" :class="{ 'christmas-btn': isChristmasSeason }" :disabled="isLoading" @click="openSRTFile">
              <span v-if="!isLoading">打开字幕文件</span>
              <span v-else>{{ loadingMessage }}</span>
            </button>
            <div class="transcription-btn-group">
              <button class="transcription-btn" :disabled="isLoading" @click="startTranscription">
                <i class="i-mdi-microphone"></i>
                <span>AI 语音转录</span>
                <span class="model-badge">{{ currentModelName }}</span>
              </button>
              <el-dropdown trigger="click" @command="onSelectModel" popper-class="model-selector-dropdown">
                <button class="transcription-dropdown" :disabled="isLoading">
                  <i class="i-mdi-chevron-down"></i>
                </button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <!-- Whisper 模型 -->
                    <el-dropdown-item disabled class="dropdown-header">
                      <span>OpenAI Whisper</span>
                    </el-dropdown-item>
                    <el-dropdown-item
                      v-for="model in downloadedModels"
                      :key="model.name"
                      :command="model.name"
                      class="model-option"
                      :class="{ 'is-active': configStore.transcriptionEngine === 'whisper' && model.name === configStore.whisperModel }"
                    >
                      <span class="model-name">{{ model.name }}</span>
                      <span class="model-desc">{{ getWhisperModelDesc(model.name) }}</span>
                      <i v-if="configStore.transcriptionEngine === 'whisper' && model.name === configStore.whisperModel" class="i-mdi-check model-check"></i>
                    </el-dropdown-item>
                    <!-- 没有下载模型时，显示 tiny 作为默认选项 -->
                    <el-dropdown-item
                      v-if="downloadedModels.length === 0"
                      command="tiny"
                      class="model-option"
                      :class="{ 'is-active': configStore.transcriptionEngine === 'whisper' }"
                    >
                      <span class="model-name">tiny</span>
                      <span class="model-desc">最快速度（首次自动下载）</span>
                      <i v-if="configStore.transcriptionEngine === 'whisper'" class="i-mdi-check model-check"></i>
                    </el-dropdown-item>
                    <!-- SenseVoice (仅在已安装时显示) -->
                    <template v-if="sensevoiceEnvStatus.ready">
                      <el-dropdown-item divided disabled class="dropdown-header">
                        <span>阿里 SenseVoice</span>
                      </el-dropdown-item>
                      <el-dropdown-item
                        command="sensevoice"
                        class="model-option"
                        :class="{ 'is-active': configStore.transcriptionEngine === 'sensevoice' }"
                      >
                        <span class="model-name">Small</span>
                        <span class="model-desc">中文优化</span>
                        <i v-if="configStore.transcriptionEngine === 'sensevoice'" class="i-mdi-check model-check"></i>
                      </el-dropdown-item>
                    </template>
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
              <button class="recent-delete-btn" @click="removeRecentFile(file.path, $event)" title="从列表中移除">
                <i class="i-mdi-close"></i>
              </button>
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

    <!-- 新手引导 -->
    <WelcomeGuide v-if="showWelcomeGuide" @close="closeWelcomeGuide" />

    <!-- 转录进度对话框 -->
    <el-dialog 
      v-model="showTranscriptionDialog" 
      :close-on-click-modal="false" 
      :close-on-press-escape="false" 
      :show-close="false"
      width="420px"
      class="transcription-dialog"
      :class="{ 'is-transcribing': isTranscribing }"
    >
      <div class="transcription-content" :class="{ 'is-transitioning': isTransitioningToEditor }">
        <!-- 关闭按钮 -->
        <button v-if="!isTransitioningToEditor" class="close-btn" @click="cancelTranscription">
          <i class="i-mdi-close"></i>
        </button>
        
        <!-- 过渡到编辑器的动画 -->
        <template v-if="isTransitioningToEditor">
          <div class="transition-animation">
            <div class="success-icon">
              <i class="i-mdi-check-circle"></i>
            </div>
          </div>
          <p class="transition-status">正在进入编辑器...</p>
        </template>
        
        <!-- 转录中的动画 -->
        <template v-else>
          <!-- 动画图标区域 -->
          <div class="transcription-animation">
            <div class="audio-wave">
              <span class="wave-bar"></span>
              <span class="wave-bar"></span>
              <span class="wave-bar"></span>
              <span class="wave-bar"></span>
              <span class="wave-bar"></span>
            </div>
            <div class="pulse-ring"></div>
            <div class="pulse-ring delay-1"></div>
            <div class="pulse-ring delay-2"></div>
          </div>
          
          <!-- 进度信息 -->
          <div class="progress-info">
            <!-- 下载或 SenseVoice 转录时显示真实进度条 -->
            <template v-if="showRealProgress">
              <div class="progress-bar-container">
                <div class="progress-bar-track">
                  <div 
                    class="progress-bar-fill" 
                    :style="{ width: `${Math.round(transcriptionProgress)}%` }"
                  ></div>
                  <div 
                    class="progress-bar-glow" 
                    :style="{ left: `${Math.round(transcriptionProgress)}%` }"
                  ></div>
                </div>
              </div>
              <div class="progress-stats">
                <span class="progress-percentage">{{ Math.round(transcriptionProgress) }}%</span>
                <span class="progress-status">{{ localizedMessage }}</span>
              </div>
            </template>
            <!-- Whisper 转录时只显示状态文字（不显示进度条） -->
            <template v-else>
              <p class="transcription-status">{{ localizedMessage }}</p>
            </template>
          </div>
          
          <!-- 提示信息 -->
          <p class="transcription-hint">
            <i class="i-mdi-information-outline"></i>
            <template v-if="transcriptionMessage.includes('下载') || transcriptionMessage.includes('首次')">
              首次使用需要下载模型，请耐心等待
            </template>
            <template v-else>
              转录时间取决于音频长度和模型大小
            </template>
          </p>
        </template>
      </div>
      
      <template #footer>
        <div v-if="!isTransitioningToEditor" class="dialog-footer">
          <el-button type="default" @click="cancelTranscription">
            取消转录
          </el-button>
        </div>
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

/* 🎄 圣诞按钮样式 */
.primary-btn.christmas-btn {
  background: linear-gradient(135deg, #dc2626, #b91c1c);
  box-shadow: 0 2px 8px rgba(220, 38, 38, 0.25);
}

.primary-btn.christmas-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  box-shadow: 0 4px 12px rgba(220, 38, 38, 0.3);
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

/* 下拉菜单分组标题 */
:deep(.dropdown-header) {
  font-size: 11px !important;
  color: #909399 !important;
  font-weight: 600;
  letter-spacing: 0.3px;
  display: flex !important;
  align-items: center;
  gap: 6px;
  padding: 8px 12px 4px !important;
}



:deep(.el-dropdown-menu__item.is-active) {
  color: #409eff;
  background: rgba(64, 158, 255, 0.08);
}

:deep(.model-option) {
  display: flex !important;
  align-items: center;
  gap: 8px;
  padding: 8px 12px !important;
  min-width: 180px;
}

:deep(.model-option .model-name) {
  font-size: 13px;
  font-weight: 500;
  color: #303133;
}

:deep(.model-option .model-desc) {
  font-size: 11px;
  color: #909399;
  flex: 1;
}

:deep(.model-option .model-check) {
  font-size: 16px;
  color: #409eff;
  margin-left: auto;
}

:deep(.model-option.is-active .model-name) {
  color: #409eff;
}

:deep(.no-model-hint) {
  display: flex !important;
  align-items: center;
  gap: 6px;
  font-size: 12px !important;
  color: #909399 !important;
}

:deep(.no-model-hint i) {
  font-size: 14px;
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

.recent-delete-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  color: #c0c4cc;
  transition: all 0.15s ease;
  flex-shrink: 0;
  margin-left: 4px;
}

.recent-delete-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: #909399;
}

.recent-delete-btn i {
  font-size: 14px;
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
:deep(.transcription-dialog) {
  user-select: none;
  
  .el-dialog__header {
    display: none;
  }
  .el-dialog__body {
    padding: 0;
    user-select: none;
  }
  .el-dialog__footer {
    padding: 0 24px 20px;
    border-top: none;
    user-select: none;
  }
  .el-dialog {
    border-radius: 16px;
    overflow: hidden;
  }
  
  * {
    user-select: none;
  }
}

.transcription-content {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 32px 24px 24px;
  background: linear-gradient(180deg, #f0f7ff 0%, #fff 100%);
  user-select: none;
}

/* 关闭按钮 */
.close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  color: #909399;
  transition: all 0.15s ease;
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: #606266;
}

.close-btn i {
  font-size: 18px;
}

/* 动画区域 */
.transcription-animation {
  position: relative;
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 20px;
}

/* 音频波形动画 */
.audio-wave {
  display: flex;
  align-items: center;
  gap: 4px;
  z-index: 2;
}

.wave-bar {
  width: 4px;
  height: 20px;
  background: linear-gradient(180deg, #409eff 0%, #79bbff 100%);
  border-radius: 2px;
  animation: wave 1.2s ease-in-out infinite;
}

.wave-bar:nth-child(1) { animation-delay: 0s; height: 16px; }
.wave-bar:nth-child(2) { animation-delay: 0.1s; height: 24px; }
.wave-bar:nth-child(3) { animation-delay: 0.2s; height: 32px; }
.wave-bar:nth-child(4) { animation-delay: 0.3s; height: 24px; }
.wave-bar:nth-child(5) { animation-delay: 0.4s; height: 16px; }

@keyframes wave {
  0%, 100% { transform: scaleY(0.5); opacity: 0.6; }
  50% { transform: scaleY(1); opacity: 1; }
}

/* 脉冲环动画 */
.pulse-ring {
  position: absolute;
  width: 60px;
  height: 60px;
  border: 2px solid rgba(64, 158, 255, 0.3);
  border-radius: 50%;
  animation: pulse 2s ease-out infinite;
}

.pulse-ring.delay-1 { animation-delay: 0.6s; }
.pulse-ring.delay-2 { animation-delay: 1.2s; }

@keyframes pulse {
  0% {
    transform: scale(1);
    opacity: 0.6;
  }
  100% {
    transform: scale(2);
    opacity: 0;
  }
}



/* 进度信息 */
.progress-info {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-bar-container {
  width: 100%;
  position: relative;
}

.progress-bar-track {
  width: 100%;
  height: 8px;
  background: #e4e7ed;
  border-radius: 4px;
  overflow: visible;
  position: relative;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #409eff 0%, #79bbff 100%);
  border-radius: 4px;
  transition: width 0.3s ease;
  position: relative;
}



.progress-bar-glow {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 12px;
  height: 12px;
  background: #409eff;
  border-radius: 50%;
  box-shadow: 0 0 12px rgba(64, 158, 255, 0.6);
  transition: left 0.3s ease;
}

.progress-stats {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.progress-percentage {
  font-size: 24px;
  font-weight: 700;
  color: #409eff;
  font-variant-numeric: tabular-nums;
}

/* 转录状态文字 */
.transcription-status {
  font-size: 14px;
  color: #606266;
  text-align: center;
  margin: 0;
}

.progress-status {
  font-size: 13px;
  color: #909399;
  text-align: center;
  flex: 1;
  word-break: break-word;
}

/* 提示信息 */
.transcription-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 20px 0 0;
  padding: 10px 14px;
  background: rgba(64, 158, 255, 0.08);
  border-radius: 8px;
  font-size: 12px;
  color: #606266;
}

.transcription-hint i {
  font-size: 16px;
  color: #409eff;
  flex-shrink: 0;
}

/* 对话框底部 */
.dialog-footer {
  display: flex;
  justify-content: center;
}

.dialog-footer .el-button {
  min-width: 100px;
}

/* 过渡动画 */
.transcription-content.is-transitioning {
  padding: 48px 24px;
}

.transition-animation {
  position: relative;
  width: 56px;
  height: 56px;
  margin-bottom: 20px;
}

.success-icon {
  position: relative;
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #409eff 0%, #79bbff 100%);
  border-radius: 50%;
  animation: popIn 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.success-icon::before {
  content: '';
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(64, 158, 255, 0.3) 0%, rgba(121, 187, 255, 0.1) 100%);
  animation: ringPulse 0.8s ease-out;
}

.success-icon i {
  font-size: 28px;
  color: white;
}

@keyframes popIn {
  0% {
    transform: scale(0);
    opacity: 0;
  }
  50% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}

@keyframes ringPulse {
  0% {
    transform: scale(0.8);
    opacity: 0;
  }
  50% {
    opacity: 1;
  }
  100% {
    transform: scale(1.3);
    opacity: 0;
  }
}

.transition-status {
  font-size: 14px;
  font-weight: 500;
  color: #606266;
  margin: 0;
  animation: fadeSlideIn 0.4s ease 0.15s both;
}

@keyframes fadeSlideIn {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>

<style>
/* 全局样式 - 禁止转录对话框文字选中 */
.transcription-dialog,
.transcription-dialog * {
  user-select: none !important;
  -webkit-user-select: none !important;
}
</style>
