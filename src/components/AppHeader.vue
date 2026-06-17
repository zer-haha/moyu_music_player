<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useAppStore } from '../stores/app'
import { usePlaylistStore } from '../stores/playlist'
import { usePlayerStore } from '../stores/player'

const appStore = useAppStore()
const playlistStore = usePlaylistStore()
const playerStore = usePlayerStore()
const searchInput = ref('')

function onSearch(e: Event) {
  const val = (e.target as HTMLInputElement).value
  searchInput.value = val
  appStore.setSearchQuery(val)
}

async function minimizeWindow() {
  try {
    await getCurrentWindow().minimize()
  } catch {}
}

async function closeWindow() {
  try {
    await getCurrentWindow().close()
  } catch {}
}

function startDrag(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.closest('input, button, a, select, textarea')) return
  try {
    getCurrentWindow().startDragging()
  } catch {}
}
</script>

<template>
  <header class="app-header" @mousedown="startDrag">
    <div class="header-left">
      <div class="app-icon">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
      </div>
      <span class="app-title">墨鱼听歌</span>
      <span v-if="appStore.scanningMessage" class="scan-msg">{{ appStore.scanningMessage }}</span>
    </div>

    <div class="header-center">
      <div class="search-box">
        <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="M21 21l-4.35-4.35"/>
        </svg>
        <input
          type="text"
          placeholder="搜索歌曲名、歌手、专辑..."
          :value="appStore.searchQuery"
          @input="onSearch"
        />
      </div>
    </div>

    <div class="header-right">
      <div class="window-controls">
        <button class="win-btn mini-btn" title="迷你模式" @click="appStore.toggleMiniMode()">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <rect x="2" y="6" width="20" height="12" rx="2"/>
          </svg>
        </button>
        <button class="win-btn" title="置顶" :class="{ active: appStore.isAlwaysOnTop }" @click="appStore.toggleAlwaysOnTop()">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M12 2L12 16"/>
            <path d="M8 6l4-4 4 4"/>
            <line x1="4" y1="20" x2="20" y2="20"/>
          </svg>
        </button>
        <button class="win-btn" @click="minimizeWindow">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>
        <button class="win-btn win-close" @click="closeWindow">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="6" y1="6" x2="18" y2="18"/>
            <line x1="6" y1="18" x2="18" y2="6"/>
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  height: 38px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  background: var(--bg-header);
  border-bottom: 1px solid var(--border-color);
  user-select: none;
  -webkit-user-select: none;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 140px;
}

.app-icon {
  display: flex;
  align-items: center;
  color: var(--accent);
}

.app-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 1px;
}

.scan-msg {
  font-size: 11px;
  color: var(--accent);
  animation: fadeIn 0.3s ease;
}

.header-center {
  flex: 1;
  display: flex;
  justify-content: center;
  padding: 0 16px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-input);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 4px 12px;
  max-width: 320px;
  width: 100%;
  transition: border-color 0.2s;
}

.search-box:focus-within {
  border-color: var(--accent);
}

.search-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.search-box input {
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  width: 100%;
  line-height: 1.5;
}

.search-box input::placeholder {
  color: var(--text-muted);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 2px;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.win-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.win-btn.active {
  color: var(--accent);
}

.win-btn.mini-btn {
  width: 36px;
  height: 30px;
}

.win-close:hover {
  background: #e81123;
  color: white;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
