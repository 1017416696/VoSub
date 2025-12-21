# Tech Stack & Build System

## Frontend

- **Vue 3** with TypeScript (Composition API, `<script setup>`)
- **Vite 7** (build tool, HMR)
- **Pinia** (state management)
- **Vue Router** (2 routes: `/` welcome, `/editor` main)
- **Tailwind CSS 4** + **Element Plus** (UI)
- **Canvas** (audio waveform rendering)
- **Howler.js** (audio playback)

## Backend (Rust)

- **Tauri 2** (desktop framework)
- **Tokio** (async runtime)
- **Symphonia** (audio codecs)
- **Serde** (JSON serialization)

## Development Tools

- **pnpm** (package manager)
- **Vitest** (unit testing)
- **ESLint** + **Prettier** (code quality)
- **TypeScript** (strict mode)

## Common Commands

```bash
# Development (frontend + backend)
pnpm tauri dev

# Run tests
pnpm test

# Type checking
pnpm type-check

# Build production
pnpm tauri build

# Check Rust code
pnpm check

# Version bump
pnpm bump X.Y.Z
```

## Code Style

### TypeScript/Vue
- No semicolons
- Single quotes
- 2-space indentation
- 120 char line width
- Trailing commas (ES5 style)

### Rust
- Standard rustfmt
- LTO enabled in release
- Optimize for speed (`opt-level = 3`)

## Auto-Import

Vue, Vue Router, and Pinia APIs are auto-imported. No need to manually import `ref`, `computed`, `watch`, `useRouter`, etc.

## Key Dependencies

| Package | Purpose |
|---------|---------|
| @tauri-apps/api | Tauri IPC |
| @tauri-apps/plugin-dialog | Native file dialogs |
| element-plus | UI components |
| Canvas API | Waveform rendering |
| howler | Audio playback |
| pinia | State management |
