<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { usePlaylistStore } from '../stores/playlist'
import { useAppStore } from '../stores/app'
import type { Playlist } from '../types'

const playlistStore = usePlaylistStore()
const appStore = useAppStore()
const isRenaming = ref(false)
const renameId = ref(0)
const renameValue = ref('')
const showNewInput = ref(false)
const newPlaylistName = ref('')
const contextMenu = ref({ show: false, x: 0, y: 0, playlist: null as Playlist | null })

// Mouse-based drag state
const dragState = ref({
  active: false,
  startIndex: -1,
  targetIndex: -1,
  startY: 0,
  moved: false,
})
const listRef = ref<HTMLElement | null>(null)

function selectPlaylist(id: number) {
  playlistStore.setCurrentPlaylist(id)
  appStore.setSearchQuery('')
}
function startRename(pl: Playlist) {
  if (pl.kind === 'default') return
  isRenaming.value = true; renameId.value = pl.id; renameValue.value = pl.name
}
async function confirmRename() {
  if (renameValue.value.trim()) {
    await playlistStore.renamePlaylist(renameId.value, renameValue.value.trim())
    appStore.saveConfig()
  }
  isRenaming.value = false; renameId.value = 0
}
function cancelRename() { isRenaming.value = false; renameId.value = 0 }
function startNewPlaylist() { showNewInput.value = true; newPlaylistName.value = '' }
async function confirmNewPlaylist() {
  const name = newPlaylistName.value.trim()
  if (name) { await playlistStore.createPlaylist(name); appStore.saveConfig() }
  showNewInput.value = false; newPlaylistName.value = ''
}
async function deletePlaylist(id: number) { await playlistStore.removePlaylist(id); appStore.saveConfig() }
async function clearPlaylist(id: number) { await playlistStore.clearPlaylist(id); appStore.saveConfig() }

function onPlaylistContextMenu(e: MouseEvent, pl: Playlist) {
  e.preventDefault(); e.stopPropagation()
  contextMenu.value = { show: true, x: e.clientX, y: e.clientY, playlist: pl }
}
function onSidebarContextMenu(e: MouseEvent) {
  e.preventDefault()
  contextMenu.value = { show: true, x: e.clientX, y: e.clientY, playlist: null }
}
function hideContextMenu() { contextMenu.value.show = false }
function ctxNewPlaylist() { hideContextMenu(); startNewPlaylist() }
function ctxRename() { if (contextMenu.value.playlist) startRename(contextMenu.value.playlist); hideContextMenu() }
function ctxDelete() { if (contextMenu.value.playlist && contextMenu.value.playlist.kind !== 'default') deletePlaylist(contextMenu.value.playlist.id); hideContextMenu() }
function ctxClear() { if (contextMenu.value.playlist) clearPlaylist(contextMenu.value.playlist.id); hideContextMenu() }

// ---- Mouse-based drag reorder ----
function onPlMouseDown(e: MouseEvent, index: number) {
  if (e.button !== 0) return
  const pl = playlistStore.playlists[index]
  if (!pl || pl.kind === 'default') return
  // Don't start drag on input/button clicks
  const target = e.target as HTMLElement
  if (target.closest('input, button')) return

  dragState.value = {
    active: false,
    startIndex: index,
    targetIndex: index,
    startY: e.clientY,
    moved: false,
  }
  document.addEventListener('mousemove', onPlMouseMove)
  document.addEventListener('mouseup', onPlMouseUp)
}

function onPlMouseMove(e: MouseEvent) {
  const ds = dragState.value
  if (ds.startIndex < 0) return

  if (!ds.active) {
    if (Math.abs(e.clientY - ds.startY) > 5) {
      ds.active = true
      ds.moved = true
    } else {
      return
    }
  }

  // Find which item the mouse is over
  const items = listRef.value?.querySelectorAll('.playlist-item')
  if (!items) return

  let targetIdx = ds.startIndex
  for (let i = 0; i < items.length; i++) {
    const el = items[i] as HTMLElement
    // Skip the new-input div
    if (el.classList.contains('new-input')) continue
    const rect = el.getBoundingClientRect()
    if (e.clientY >= rect.top && e.clientY <= rect.bottom) {
      targetIdx = i
      break
    }
    if (e.clientY < rect.top) {
      targetIdx = i
      break
    }
    if (i === items.length - 1 && e.clientY > rect.bottom) {
      targetIdx = i
    }
  }

  // Don't allow dropping on default playlist (index 0)
  const targetPl = playlistStore.playlists[targetIdx]
  if (targetPl && targetPl.kind === 'default') {
    targetIdx = 1
  }
  if (targetIdx < 1) targetIdx = 1

  ds.targetIndex = targetIdx
}

function onPlMouseUp() {
  const ds = dragState.value
  document.removeEventListener('mousemove', onPlMouseMove)
  document.removeEventListener('mouseup', onPlMouseUp)

  if (ds.active && ds.startIndex >= 0 && ds.targetIndex >= 0 && ds.startIndex !== ds.targetIndex) {
    playlistStore.movePlaylist(ds.startIndex, ds.targetIndex)
  }

  dragState.value = {
    active: false,
    startIndex: -1,
    targetIndex: -1,
    startY: 0,
    moved: false,
  }
}

onMounted(() => { document.addEventListener('click', hideContextMenu) })
onUnmounted(() => {
  document.removeEventListener('click', hideContextMenu)
  document.removeEventListener('mousemove', onPlMouseMove)
  document.removeEventListener('mouseup', onPlMouseUp)
})
</script>

<template>
  <aside class="sidebar" @contextmenu.prevent="onSidebarContextMenu">
    <div class="sidebar-title">
      <span>播放列表</span>
      <button class="add-btn" title="新建播放列表" @click="startNewPlaylist">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </button>
    </div>
    <div ref="listRef" class="playlist-list">
      <div v-if="showNewInput" class="playlist-item new-input">
        <input v-model="newPlaylistName" placeholder="输入列表名称"
          @keyup.enter="confirmNewPlaylist" @keyup.escape="showNewInput = false"
          @blur="confirmNewPlaylist" autofocus />
      </div>
      <div v-for="(pl, idx) in playlistStore.playlists" :key="pl.id" class="playlist-item"
        :data-playlist-id="pl.id"
        :class="{
          active: pl.id === playlistStore.currentPlaylistId,
          'drag-active': dragState.active && dragState.startIndex === idx,
          'drag-target': dragState.active && dragState.targetIndex === idx && dragState.startIndex !== idx,
          'draggable': pl.kind !== 'default',
        }"
        @mousedown="onPlMouseDown($event, idx)"
        @click="selectPlaylist(pl.id)" @contextmenu="onPlaylistContextMenu($event, pl)">
        <div class="pl-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>
          </svg>
        </div>
        <input v-if="isRenaming && renameId === pl.id" v-model="renameValue" class="rename-input"
          @keyup.enter="confirmRename" @keyup.escape="cancelRename" @blur="confirmRename" @click.stop autofocus />
        <template v-else>
          <span class="pl-name" :title="pl.name">{{ pl.name }}</span>
          <span class="pl-count">{{ pl.tracks.length }}</span>
          <svg v-if="pl.kind !== 'default'" class="drag-handle" width="10" height="14" viewBox="0 0 10 14" fill="currentColor" opacity="0.3">
            <circle cx="2" cy="2" r="1.2"/><circle cx="8" cy="2" r="1.2"/>
            <circle cx="2" cy="7" r="1.2"/><circle cx="8" cy="7" r="1.2"/>
            <circle cx="2" cy="12" r="1.2"/><circle cx="8" cy="12" r="1.2"/>
          </svg>
        </template>
      </div>
    </div>
    <div v-if="contextMenu.show" class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }" @click.stop>
      <template v-if="contextMenu.playlist">
        <div class="ctx-item" @click="ctxNewPlaylist">新建播放列表</div>
        <div v-if="contextMenu.playlist.kind !== 'default'" class="ctx-item" @click="ctxRename">重命名</div>
        <div class="ctx-item" @click="ctxClear">清空列表</div>
        <div v-if="contextMenu.playlist.kind !== 'default'" class="ctx-divider"></div>
        <div v-if="contextMenu.playlist.kind !== 'default'" class="ctx-item danger" @click="ctxDelete">删除列表</div>
      </template>
      <template v-else>
        <div class="ctx-item" @click="ctxNewPlaylist">新建播放列表</div>
      </template>
    </div>
  </aside>
</template>

<style scoped>
.sidebar { width: 200px; min-width: 160px; max-width: 280px; background: var(--bg-sidebar);
  border-right: 1px solid var(--border-color); display: flex; flex-direction: column; position: relative; flex-shrink: 0; }
.sidebar-title { display: flex; align-items: center; justify-content: space-between;
  padding: 10px 12px 6px; font-size: 11px; font-weight: 600; color: var(--text-muted);
  text-transform: uppercase; letter-spacing: 1px; }
.add-btn { display: flex; align-items: center; justify-content: center; width: 22px; height: 22px;
  border: none; background: transparent; color: var(--text-muted); border-radius: 4px; cursor: pointer; transition: all 0.15s; }
.add-btn:hover { background: var(--bg-hover); color: var(--accent); }
.playlist-list { flex: 1; overflow-y: auto; padding: 4px 6px; }
.playlist-item { display: flex; align-items: center; gap: 8px; padding: 7px 8px; border-radius: 6px;
  font-size: 13px; color: var(--text-secondary); transition: all 0.15s; position: relative; }
.playlist-item:hover { background: var(--bg-hover); color: var(--text-primary); }
.playlist-item.active { background: var(--bg-active); color: var(--text-primary); }
.playlist-item.active .pl-icon { color: var(--accent); }
.playlist-item.draggable { cursor: default; }
.playlist-item.drag-active { opacity: 0.4; }
.playlist-item.drag-target { box-shadow: inset 0 2px 0 var(--accent); background: rgba(77,163,255,0.08); }
.pl-icon { flex-shrink: 0; display: flex; align-items: center; }
.pl-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.pl-count { font-size: 11px; color: var(--text-muted); min-width: 18px; text-align: right; }
.drag-handle { flex-shrink: 0; cursor: grab; }
.drag-handle:active { cursor: grabbing; }
.rename-input, .new-input input { flex: 1; border: 1px solid var(--accent); background: var(--bg-input);
  color: var(--text-primary); font-size: 13px; padding: 2px 6px; border-radius: 4px; outline: none; min-width: 0; }
.new-input { padding: 5px 8px; }
.context-menu { position: fixed; background: var(--bg-menu); border: 1px solid var(--border-color);
  border-radius: 6px; padding: 4px; min-width: 140px; box-shadow: 0 4px 16px rgba(0,0,0,0.3); z-index: 1000; }
.ctx-item { padding: 6px 12px; font-size: 13px; color: var(--text-primary); border-radius: 4px;
  cursor: pointer; transition: background 0.1s; }
.ctx-item:hover { background: var(--bg-hover); }
.ctx-item.danger:hover { background: rgba(239,68,68,0.12); color: #ef4444; }
.ctx-divider { height: 1px; background: var(--border-color); margin: 4px 8px; }
</style>
