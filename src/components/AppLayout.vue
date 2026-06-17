<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'
import AppHeader from './AppHeader.vue'
import Sidebar from './Sidebar.vue'
import TrackList from './TrackList.vue'
import PlayerBar from './PlayerBar.vue'
import { usePlaylistStore } from '../stores/playlist'
import { useAppStore } from '../stores/app'
import { invoke } from '@tauri-apps/api/core'
import type { Track } from '../types'

const playlistStore = usePlaylistStore()
const appStore = useAppStore()
const isDragOver = ref(false)
const dragPosition = ref({ x: 0, y: 0 })
let unlistenDrop: UnlistenFn | null = null

onMounted(async () => {
  unlistenDrop = await getCurrentWindow().onDragDropEvent(async (event) => {
    if (event.payload.type === 'enter') {
      isDragOver.value = true
      if (event.payload.position) {
        dragPosition.value = { x: event.payload.position.x, y: event.payload.position.y }
      }
    } else if (event.payload.type === 'over') {
      if (event.payload.position) {
        dragPosition.value = { x: event.payload.position.x, y: event.payload.position.y }
      }
    } else if (event.payload.type === 'drop') {
      isDragOver.value = false
      const paths = event.payload.paths as string[]
      if (paths && paths.length > 0) {
        await handleDroppedFiles(paths)
      }
    } else if (event.payload.type === 'leave') {
      isDragOver.value = false
    }
  })
})

onUnmounted(() => { if (unlistenDrop) unlistenDrop() })

/// Determine drop target type using elementFromPoint
function getDropTarget(): { type: 'sidebar' | 'playlist' | 'tracklist'; playlistId?: number } {
  const el = document.elementFromPoint(dragPosition.value.x, dragPosition.value.y)
  if (!el) return { type: 'tracklist' }
  const plItem = el.closest('.playlist-item') as HTMLElement
  if (plItem) {
    const plId = plItem.dataset.playlistId
    if (plId) return { type: 'playlist', playlistId: parseInt(plId) }
  }
  if (el.closest('.sidebar')) return { type: 'sidebar' }
  return { type: 'tracklist' }
}

async function addPathsToPlaylist(paths: string[], playlistId: number): Promise<number> {
  try {
    const result = await invoke<{ added: number; skipped: number; failed: number }>('library_add_files', {
      paths,
      playlistId,
    })
    // Reload songs for this playlist
    const songs = await invoke<Track[]>('playlist_get_songs', { playlistId })
    const pl = playlistStore.playlists.find(p => p.id === playlistId)
    if (pl) pl.tracks = songs
    return result.added + result.skipped
  } catch {
    return 0
  }
}

async function scanFolderToPlaylist(folderPath: string, playlistId: number): Promise<number> {
  try {
    const result = await invoke<{ added: number; skipped: number; failed: number }>('scan_start_folder', {
      path: folderPath,
      playlistId,
    })
    // Reload songs for this playlist
    const songs = await invoke<Track[]>('playlist_get_songs', { playlistId })
    const pl = playlistStore.playlists.find(p => p.id === playlistId)
    if (pl) pl.tracks = songs
    return result.added
  } catch {
    return 0
  }
}

async function handleDroppedFiles(paths: string[]) {
  const target = getDropTarget()
  appStore.scanningMessage = '正在添加歌曲...'
  let totalAdded = 0

  if (target.type === 'sidebar') {
    // Drop on sidebar: folders create new playlists, files add to current
    for (const path of paths) {
      const folderName = path.replace(/\\/g, '/').split('/').pop() || '新播放列表'
      const plId = await playlistStore.createPlaylist(folderName)
      if (plId) {
        totalAdded += await scanFolderToPlaylist(path, plId)
        playlistStore.setCurrentPlaylist(plId)
      }
    }
  } else if (target.type === 'playlist' && target.playlistId) {
    // Drop on specific playlist: add to that playlist
    playlistStore.setCurrentPlaylist(target.playlistId)
    totalAdded += await addPathsToPlaylist(paths, target.playlistId)
  } else {
    // Drop on track list: add to current playlist
    totalAdded += await addPathsToPlaylist(paths, playlistStore.currentPlaylistId)
  }

  appStore.scanningMessage = totalAdded > 0 ? `已添加 ${totalAdded} 首歌曲` : '未发现音乐文件'
  setTimeout(() => { appStore.scanningMessage = '' }, 3000)
  appStore.saveConfig()
}
</script>

<template>
  <div class="app-layout">
    <AppHeader />
    <div class="app-body">
      <Sidebar />
      <TrackList />
    </div>
    <PlayerBar />
    <div v-if="isDragOver" class="global-drop-overlay">
      <div class="drop-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>
        </svg>
        <span>释放以添加音乐文件</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-layout { display: flex; flex-direction: column; height: 100vh; width: 100vw; overflow: hidden; background: var(--bg-primary); color: var(--text-primary); position: relative; }
.app-body { display: flex; flex: 1; overflow: hidden; min-height: 0; }
.global-drop-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 9999; pointer-events: none; }
.drop-content { display: flex; flex-direction: column; align-items: center; gap: 16px; color: var(--accent); font-size: 18px; padding: 40px 60px; border: 3px dashed var(--accent); border-radius: 16px; background: rgba(59,130,246,0.08); }
</style>
