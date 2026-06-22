<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlaylistStore } from '../stores/playlist'
import { usePlayerStore } from '../stores/player'
import { useAppStore } from '../stores/app'
import { formatTime } from '../utils/format'
import type { Track } from '../types'

const playlistStore = usePlaylistStore()
const playerStore = usePlayerStore()
const appStore = useAppStore()
const ITEM_HEIGHT = 36
const BUFFER_SIZE = 10
const scrollContainer = ref<HTMLElement | null>(null)
const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const selectedIds = ref<Set<number>>(new Set())
const lastClickedIndex = ref(-1)
const menuJustOpened = ref(false)
const dragIndex = ref(-1)
const dropIndex = ref(-1)
const isImporting = ref(false)

const colWidths = ref({ index: 42, format: 140, duration: 70 })
const gridCols = computed(() =>
  `${colWidths.value.index}px minmax(80px,1fr) ${colWidths.value.format}px ${colWidths.value.duration}px`
)

function startResize(e: MouseEvent, col: 'index' | 'format' | 'duration') {
  e.preventDefault()
  e.stopPropagation()
  const startX = e.clientX
  const startWidth = colWidths.value[col]
  const onMove = (ev: MouseEvent) => {
    colWidths.value = { ...colWidths.value, [col]: Math.max(30, startWidth + ev.clientX - startX) }
  }
  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

const filteredTracks = computed(() => {
  const tracks = playlistStore.currentTracks
  const q = appStore.searchQuery.toLowerCase().trim()
  if (!q) return tracks
  return tracks.filter(
    (t) =>
      t.title.toLowerCase().includes(q) ||
      (t.artist || '').toLowerCase().includes(q) ||
      (t.album || '').toLowerCase().includes(q)
  )
})

const otherPlaylists = computed(() =>
  playlistStore.playlists.filter((p) => p.id !== playlistStore.currentPlaylistId)
)

const visibleStart = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / ITEM_HEIGHT) - BUFFER_SIZE)
)
const visibleEnd = computed(() => {
  const h = scrollContainer.value?.clientHeight || 500
  return Math.min(
    filteredTracks.value.length,
    Math.ceil((scrollTop.value + h) / ITEM_HEIGHT) + BUFFER_SIZE
  )
})
const visibleTracks = computed(() =>
  filteredTracks.value.slice(visibleStart.value, visibleEnd.value)
)
const totalHeight = computed(() => filteredTracks.value.length * ITEM_HEIGHT)
const offsetY = computed(() => visibleStart.value * ITEM_HEIGHT)

function onScroll(e: Event) {
  scrollTop.value = (e.target as HTMLElement).scrollTop
}

function scrollToIndex(index: number) {
  if (!scrollContainer.value) return
  const top = index * ITEM_HEIGHT
  const h = scrollContainer.value.clientHeight
  if (top < scrollTop.value) scrollContainer.value.scrollTop = top
  else if (top + ITEM_HEIGHT > scrollTop.value + h)
    scrollContainer.value.scrollTop = top - h + ITEM_HEIGHT
}

function onClickTrack(e: MouseEvent, track: Track, index: number) {
  const gi = visibleStart.value + index
  if (e.ctrlKey || e.metaKey) {
    const s = new Set(selectedIds.value)
    s.has(track.id) ? s.delete(track.id) : s.add(track.id)
    selectedIds.value = s
    lastClickedIndex.value = gi
  } else if (e.shiftKey && lastClickedIndex.value >= 0) {
    const a = Math.min(lastClickedIndex.value, gi)
    const b = Math.max(lastClickedIndex.value, gi)
    const s = new Set(selectedIds.value)
    for (let i = a; i <= b; i++) {
      const t = filteredTracks.value[i]
      if (t) s.add(t.id)
    }
    selectedIds.value = s
  } else {
    selectedIds.value = new Set([track.id])
    lastClickedIndex.value = gi
  }
}

function onDblClick(track: Track) {
  selectedIds.value = new Set([track.id])
  playerStore.playTrack(track)
}

function onDragStart(e: DragEvent, index: number) {
  const gi = visibleStart.value + index
  dragIndex.value = gi
  dropIndex.value = gi
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', String(gi))
  }
}
function onDragOver(e: DragEvent, index: number) {
  e.preventDefault()
  e.stopPropagation()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  dropIndex.value = visibleStart.value + index
}
function onDragEnd() {
  if (dragIndex.value >= 0 && dropIndex.value >= 0 && dragIndex.value !== dropIndex.value) {
    playlistStore.reorderTracks(playlistStore.currentPlaylistId, dragIndex.value, dropIndex.value)
  }
  dragIndex.value = -1
  dropIndex.value = -1
}
function onDrop(e: DragEvent) {
  e.preventDefault()
  e.stopPropagation()
  onDragEnd()
}

function onKeyDown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
    e.preventDefault()
    selectedIds.value = new Set(filteredTracks.value.map((t) => t.id))
    return
  }
  if (e.key === 'Delete' && selectedIds.value.size > 0) {
    e.preventDefault()
    removeSelectedTracks()
    return
  }
  if (e.key === 'ArrowDown' && filteredTracks.value.length > 0) {
    e.preventDefault()
    const idx =
      lastClickedIndex.value < 0
        ? 0
        : Math.min(lastClickedIndex.value + 1, filteredTracks.value.length - 1)
    const track = filteredTracks.value[idx]
    if (track) {
      selectedIds.value = new Set([track.id])
      lastClickedIndex.value = idx
      scrollToIndex(idx)
    }
    return
  }
  if (e.key === 'ArrowUp' && filteredTracks.value.length > 0) {
    e.preventDefault()
    const idx =
      lastClickedIndex.value < 0
        ? 0
        : Math.max(lastClickedIndex.value - 1, 0)
    const track = filteredTracks.value[idx]
    if (track) {
      selectedIds.value = new Set([track.id])
      lastClickedIndex.value = idx
      scrollToIndex(idx)
    }
    return
  }
  if (e.key === 'Enter' && lastClickedIndex.value >= 0) {
    e.preventDefault()
    const track = filteredTracks.value[lastClickedIndex.value]
    if (track) playerStore.playTrack(track)
    return
  }
  if (e.key === ' ' && document.activeElement === containerRef.value) {
    e.preventDefault()
    playerStore.togglePlay()
  }
}

const contextMenu = ref({ show: false, x: 0, y: 0, track: null as Track | null })
const emptyContextMenu = ref({ show: false, x: 0, y: 0 })

function onContextMenu(e: MouseEvent, track: Track) {
  e.preventDefault()
  e.stopPropagation()
  emptyContextMenu.value.show = false
  if (!selectedIds.value.has(track.id)) selectedIds.value = new Set([track.id])
  contextMenu.value = { show: true, x: e.clientX, y: e.clientY, track }
  menuJustOpened.value = true
  setTimeout(() => { menuJustOpened.value = false }, 200)
}
function onEmptyContextMenu(e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()
  contextMenu.value.show = false
  emptyContextMenu.value = { show: true, x: e.clientX, y: e.clientY }
  menuJustOpened.value = true
  setTimeout(() => { menuJustOpened.value = false }, 200)
}
function hideContextMenu() {
  if (menuJustOpened.value) return
  contextMenu.value.show = false
  emptyContextMenu.value.show = false
}

async function showInFolder() {
  if (contextMenu.value.track) {
    try {
      await invoke('show_in_folder', { filePath: contextMenu.value.track.path })
    } catch {}
  }
  hideContextMenu()
}

async function removeTrack() {
  if (contextMenu.value.track) {
    const id = contextMenu.value.track.id
    playerStore.checkAndStopIfRemoved(id)
    await playlistStore.removeTrack(playlistStore.currentPlaylistId, id)
    selectedIds.value.delete(id)
    appStore.saveConfig()
  }
  hideContextMenu()
}
async function removeSelectedTracks() {
  for (const id of Array.from(selectedIds.value)) {
    playerStore.checkAndStopIfRemoved(id)
    await playlistStore.removeTrack(playlistStore.currentPlaylistId, id)
  }
  selectedIds.value = new Set()
  appStore.saveConfig()
  hideContextMenu()
}
function playFromContext() {
  if (contextMenu.value.track) playerStore.playTrack(contextMenu.value.track)
  hideContextMenu()
}
async function addToPlaylist(targetPlaylistId: number) {
  const tracks =
    selectedIds.value.size > 0
      ? filteredTracks.value.filter((t) => selectedIds.value.has(t.id))
      : contextMenu.value.track
        ? [contextMenu.value.track]
        : []
  for (const track of tracks) {
    await playlistStore.addTrackToPlaylist(targetPlaylistId, track)
  }
  hideContextMenu()
  appStore.saveConfig()
}

async function pickFolderAndImport(asNewList: boolean) {
  hideContextMenu()
  if (isImporting.value) return
  try {
    const folder = await invoke<string | null>('pick_folder')
    if (!folder) return
    isImporting.value = true
    if (asNewList) {
      await playlistStore.importFolderAsNewPlaylist(folder)
    } else {
      await playlistStore.importFolderToCurrent(folder)
    }
    appStore.saveConfig()
  } catch {} finally {
    isImporting.value = false
  }
}

async function onEmptyPickFolder() {
  if (isImporting.value) return
  isImporting.value = true
  try {
    const folder = await invoke<string | null>('pick_folder')
    if (folder) {
      await playlistStore.importFolderAsNewPlaylist(folder)
      appStore.saveConfig()
    }
  } catch {} finally {
    isImporting.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', hideContextMenu)
  containerRef.value?.focus()
})
onUnmounted(() => {
  document.removeEventListener('click', hideContextMenu)
})

function isCurrentTrack(t: Track): boolean {
  return playerStore.currentTrack?.id === t.id
}
function isSelected(t: Track): boolean {
  return selectedIds.value.has(t.id)
}
function trackDuration(t: Track): number {
  return t.duration_ms / 1000
}
function isTrackMissing(t: Track): boolean {
  return t.missing
}
</script>

<template>
  <main
    ref="containerRef"
    class="track-list-container"
    tabindex="0"
    @keydown="onKeyDown"
    @click="containerRef?.focus()"
    :style="{ '--grid-cols': gridCols }"
  >
    <div class="track-header">
      <span class="col-index">
        #
        <div class="resize-handle" @mousedown="startResize($event, 'index')"></div>
      </span>
      <span class="col-title">歌曲</span>
      <span class="col-format">
        格式
        <div class="resize-handle" @mousedown="startResize($event, 'format')"></div>
      </span>
      <span class="col-duration">
        时长
        <div class="resize-handle" @mousedown="startResize($event, 'duration')"></div>
      </span>
    </div>
    <div
      v-if="filteredTracks.length > 0"
      ref="scrollContainer"
      class="track-scroll"
      @scroll="onScroll"
      @contextmenu.prevent="onEmptyContextMenu"
    >
      <div :style="{ height: totalHeight + 'px', position: 'relative' }">
        <div :style="{ transform: `translateY(${offsetY}px)` }">
          <div
            v-for="(track, i) in visibleTracks"
            :key="track.id"
            class="track-row"
            :class="{
              playing: isCurrentTrack(track),
              selected: isSelected(track),
              'not-exists': isTrackMissing(track),
              'drag-above':
                dropIndex === visibleStart + i &&
                dragIndex >= 0 &&
                dropIndex !== visibleStart + i,
              'drag-below':
                dropIndex === visibleStart + i + 1 &&
                dragIndex >= 0 &&
                dropIndex !== visibleStart + i,
              'drag-self': dragIndex === visibleStart + i,
            }"
            draggable="true"
            @dragstart="onDragStart($event, i)"
            @dragover="onDragOver($event, i)"
            @dragend="onDragEnd"
            @drop="onDrop"
            @click="onClickTrack($event, track, i)"
            @dblclick="onDblClick(track)"
            @contextmenu="onContextMenu($event, track)"
          >
            <span class="col-index">
              <span
                v-if="isCurrentTrack(track) && playerStore.isPlaying"
                class="playing-indicator"
              >
                <span></span><span></span><span></span>
              </span>
              <span v-else>{{ visibleStart + i + 1 }}</span>
            </span>
            <span class="col-title" :title="track.path">
              {{ track.title }}
              <span v-if="isTrackMissing(track)" class="missing-tag">文件不存在</span>
            </span>
            <span class="col-format">{{ (track.format || track.extension || '—').toUpperCase() }}</span>
            <span class="col-duration">{{ formatTime(trackDuration(track)) }}</span>
          </div>
        </div>
      </div>
    </div>
    <div v-else class="empty-state" @contextmenu.prevent="onEmptyContextMenu">
      <svg
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1"
        opacity="0.3"
      >
        <path d="M9 18V5l12-2v13" />
        <circle cx="6" cy="18" r="3" />
        <circle cx="18" cy="16" r="3" />
      </svg>
      <p v-if="appStore.searchQuery">没有找到匹配的歌曲</p>
      <template v-else>
        <p>还没有歌曲</p>
        <button class="empty-btn" :disabled="isImporting" @click="onEmptyPickFolder">
          {{ isImporting ? '正在添加...' : '选择文件夹，开始听歌' }}
        </button>
        <p class="empty-hint">或者把文件夹拖到这个窗口里</p>
      </template>
    </div>
    <div
      v-if="contextMenu.show"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="ctx-item" @click="playFromContext">播放</div>
      <div class="ctx-item" @click="showInFolder">在资源管理器中显示</div>
      <template v-if="otherPlaylists.length > 0">
        <div class="ctx-divider"></div>
        <div class="ctx-label">添加到列表</div>
        <div
          v-for="pl in otherPlaylists"
          :key="pl.id"
          class="ctx-item"
          @click="addToPlaylist(pl.id)"
        >
          {{ pl.name }}
        </div>
      </template>
      <div class="ctx-divider"></div>
      <div
        v-if="selectedIds.size > 1"
        class="ctx-item danger"
        @click="removeSelectedTracks"
      >
        移除选中 ({{ selectedIds.size }})
      </div>
      <div v-else class="ctx-item danger" @click="removeTrack">从列表中移除</div>
    </div>
    <div
      v-if="emptyContextMenu.show"
      class="context-menu"
      :style="{ left: emptyContextMenu.x + 'px', top: emptyContextMenu.y + 'px' }"
      @click.stop
    >
      <div class="ctx-item" @click="pickFolderAndImport(true)">选择文件夹（新建列表）</div>
      <div class="ctx-item" @click="pickFolderAndImport(false)">选择文件夹（加入当前列表）</div>
    </div>
  </main>
</template>

<style scoped>
.track-list-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  position: relative;
  background: var(--bg-primary);
  outline: none;
}
.track-header {
  display: grid;
  grid-template-columns: var(--grid-cols, 42px 1fr 140px 70px);
  padding: 6px 16px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  user-select: none;
}
.track-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}
.track-row {
  display: grid;
  grid-template-columns: var(--grid-cols, 42px 1fr 140px 70px);
  padding: 0 16px;
  height: 36px;
  align-items: center;
  font-size: 13px;
  cursor: default;
  transition: background 0.1s;
  user-select: none;
  position: relative;
}
.track-row:hover {
  background: var(--bg-hover);
}
.track-row.selected {
  background: var(--accent-dim);
}
.track-row.playing {
  background: var(--bg-active);
  overflow: hidden;
}
.track-row.playing::after {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(59, 130, 246, 0.08), transparent);
  animation: sweepHL 3s ease-in-out infinite;
  pointer-events: none;
}
@keyframes sweepHL {
  0% {
    left: -100%;
  }
  50% {
    left: 100%;
  }
  100% {
    left: 100%;
  }
}
.track-row.playing .col-title {
  color: var(--accent);
}
.track-row.not-exists {
  opacity: 0.45;
}
.track-row.drag-self {
  opacity: 0.3;
}
.track-row.drag-above {
  box-shadow: inset 0 2px 0 var(--accent);
}
.track-row.drag-below {
  box-shadow: inset 0 -2px 0 var(--accent);
}
.track-row[draggable='true'] {
  cursor: grab;
}
.track-row[draggable='true']:active {
  cursor: grabbing;
}
.col-index {
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
  position: relative;
}
.col-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding-right: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
}
.col-format {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
  padding-right: 8px;
  position: relative;
}
.col-duration {
  color: var(--text-muted);
  font-size: 12px;
  text-align: right;
  font-variant-numeric: tabular-nums;
  position: relative;
}
.resize-handle {
  position: absolute;
  right: -2px;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  z-index: 10;
}
.resize-handle:hover {
  background: var(--accent);
}
.missing-tag {
  font-size: 10px;
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
  flex-shrink: 0;
}
.playing-indicator {
  display: inline-flex;
  align-items: flex-end;
  gap: 2px;
  height: 14px;
}
.playing-indicator span {
  display: block;
  width: 2px;
  background: var(--accent);
  border-radius: 1px;
  animation: soundBar 0.8s ease-in-out infinite alternate;
}
.playing-indicator span:nth-child(1) {
  height: 6px;
  animation-delay: 0s;
}
.playing-indicator span:nth-child(2) {
  height: 10px;
  animation-delay: 0.2s;
}
.playing-indicator span:nth-child(3) {
  height: 4px;
  animation-delay: 0.4s;
}
@keyframes soundBar {
  0% {
    transform: scaleY(0.4);
  }
  100% {
    transform: scaleY(1);
  }
}
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  color: var(--text-muted);
  font-size: 14px;
}
.empty-btn {
  padding: 12px 28px;
  font-size: 15px;
  font-weight: 600;
  color: white;
  background: var(--accent);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: opacity 0.15s;
}
.empty-btn:hover:not(:disabled) {
  opacity: 0.9;
}
.empty-btn:disabled {
  opacity: 0.6;
  cursor: wait;
}
.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
  opacity: 0.7;
}
.context-menu {
  position: fixed;
  background: var(--bg-menu);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 4px;
  min-width: 160px;
  max-height: 320px;
  overflow-y: auto;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 1000;
}
.ctx-label {
  padding: 4px 12px 2px;
  font-size: 11px;
  color: var(--text-muted);
}
.ctx-item {
  padding: 6px 12px;
  font-size: 13px;
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.1s;
}
.ctx-item:hover {
  background: var(--bg-hover);
}
.ctx-item.danger:hover {
  background: rgba(239, 68, 68, 0.12);
  color: #ef4444;
}
.ctx-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 8px;
}
</style>
