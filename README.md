# VoSub

<p align="center">
  <img src="./public/icon-concept-6-small-v.png" alt="VoSub Logo" width="128" height="128">
</p>

<p align="center">
  <strong>专业的 SRT 字幕编辑器</strong><br>
  支持 AI 语音转录、字幕校正、导出多种字幕格式
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#截图预览">截图预览</a> •
  <a href="#安装">安装</a> •
  <a href="#开发">开发</a> •
  <a href="#技术栈">技术栈</a>
</p>

---

## 功能特性

### 🎬 字幕编辑
- 实时编辑 SRT 字幕文件
- 时间轴冲突检测（自动标记重叠字幕）
- 多标签页编辑，同时处理多个文件
- 双击 `.srt` 文件直接打开

### 🎵 音频同步
- 音频波形可视化（Canvas 原生渲染）
- 支持 MP3、WAV、AAC、FLAC、OGG 等格式
- 音频与字幕时间轴联动

### 🤖 AI 语音转录
- **OpenAI Whisper** - 多语言支持，多种模型可选（tiny/base/small/medium/large）
- **阿里 SenseVoice** - 中文优化，识别准确率高
- **FireRedASR** - 字幕校正，提升识别准确度
- 本地词典 - 自定义替换规则，自动纠正转录结果

### 📝 批量文本处理
- HTML 标签清理
- 标点符号规范化
- 大小写转换
- CJK 文字间距调整

### 📤 多格式导出
- SRT（SubRip）
- VTT（WebVTT）
- TXT（纯文本）
- Markdown
- FCPXML（Final Cut Pro）

## 截图预览

![VoSub Screenshot](./public/screenshot.png)

## 安装

### 下载安装包

前往 [Releases](https://github.com/your-username/vosub/releases) 页面下载最新版本：

- **macOS**: `.dmg` 安装包
- **Windows**: `.exe` 安装程序

### 系统要求

- macOS 10.15+ (Safari 16+)
- Windows 10+ (Chrome 107+)

## 开发

### 环境准备

1. 安装 [Tauri 开发环境](https://tauri.app/start/prerequisites/)
2. 安装 [pnpm](https://pnpm.io/installation)
3. 克隆仓库并安装依赖：

```bash
git clone https://github.com/your-username/vosub.git
cd vosub
pnpm install
```

### 常用命令

```bash
# 启动开发服务器（前端 + 后端）
pnpm tauri dev

# 运行测试
pnpm test

# 类型检查
pnpm type-check

# 构建生产版本
pnpm tauri build

# 检查 Rust 代码
pnpm check
```

### 项目结构

```
src/                    # 前端 (Vue 3 + TypeScript)
├── views/              # 页面组件
├── components/         # UI 组件
├── stores/             # Pinia 状态管理
├── types/              # TypeScript 类型定义
└── utils/              # 工具函数

src-tauri/              # 后端 (Rust)
├── src/
│   ├── lib.rs          # Tauri 命令处理
│   ├── srt_parser.rs   # SRT 解析/导出
│   ├── waveform_generator.rs  # 音频波形生成
│   ├── whisper_python_transcriber.rs  # Whisper 转录
│   ├── sensevoice_transcriber.rs      # SenseVoice 转录
│   └── firered_corrector.rs           # FireRedASR 校正
└── Cargo.toml          # Rust 依赖配置
```

## 技术栈

### 前端
- **Vue 3** + TypeScript (Composition API)
- **Vite 7** (构建工具)
- **Pinia** (状态管理)
- **Tailwind CSS 4** + **Element Plus** (UI)
- **Canvas** (音频波形渲染)
- **Howler.js** (音频播放)

### 后端
- **Tauri 2** (桌面框架)
- **Rust** + **Tokio** (异步运行时)
- **Symphonia** (音频解码)
