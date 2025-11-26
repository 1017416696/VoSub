import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { EditorConfig, KeyBinding } from '@/types/subtitle'

export const useConfigStore = defineStore('config', () => {
  // 编辑器配置
  const config = ref<EditorConfig>({
    autoSave: true,
    autoscroll: true,
    showWaveform: true,
    showKeyboardHints: true,
    theme: 'light',
    language: 'zh-CN',
  })

  // 快捷键绑定
  const keyBindings = ref<KeyBinding[]>([
    { key: 'Space', description: '播放/暂停', action: 'toggle-play' },
    { key: 'ArrowLeft', description: '上一条字幕', action: 'prev-subtitle' },
    { key: 'ArrowRight', description: '下一条字幕', action: 'next-subtitle' },
    { key: 'ArrowUp', description: '增加音量', action: 'volume-up' },
    { key: 'ArrowDown', description: '减少音量', action: 'volume-down' },
    { key: 'Enter', description: '编辑字幕', action: 'edit-subtitle' },
    { key: 'Escape', description: '退出编辑', action: 'exit-edit' },
    { key: 'Delete', description: '删除字幕', action: 'delete-subtitle' },
    { key: 'Tab', description: '保存并下一条', action: 'save-and-next' },
    { key: 'Shift+Tab', description: '保存并上一条', action: 'save-and-prev' },
    { key: 'Ctrl+S', description: '保存文件', action: 'save-file' },
    { key: 'Ctrl+O', description: '打开文件', action: 'open-file' },
    { key: 'Ctrl+F', description: '查找', action: 'find' },
    { key: 'Ctrl+N', description: '新增字幕', action: 'new-subtitle' },
    { key: 'Ctrl+Z', description: '撤销', action: 'undo' },
    { key: 'Ctrl+Shift+Z', description: '重做', action: 'redo' },
  ])

  // 更新配置
  const updateConfig = (partial: Partial<EditorConfig>) => {
    config.value = { ...config.value, ...partial }
    saveConfig()
  }

  // 保存配置到本地
  const saveConfig = () => {
    localStorage.setItem('srt-editor-config', JSON.stringify(config.value))
  }

  // 加载配置
  const loadConfig = () => {
    const saved = localStorage.getItem('srt-editor-config')
    if (saved) {
      try {
        const parsed = JSON.parse(saved)
        config.value = { ...config.value, ...parsed }
      } catch (error) {
        console.error('Failed to load config:', error)
      }
    }
  }

  // 初始化时加载配置
  loadConfig()

  return {
    config,
    keyBindings,
    updateConfig,
    saveConfig,
    loadConfig,
  }
})
