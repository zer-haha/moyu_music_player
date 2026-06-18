import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize, LogicalPosition } from '@tauri-apps/api/dpi'
import { usePlaylistStore } from './playlist'
import { usePlayerStore } from './player'
import type { Theme, PlaybackStateDto } from '../types'

export interface ScanProgress {
  active: boolean
  current: number
  total: number
  message: string
  taskId: string | null
}

export const useAppStore = defineStore('app', {
  state: () => ({
    theme: 'dark' as Theme,
    searchQuery: '',
    isMiniMode: false,
    isAlwaysOnTop: false,
    sidebarCollapsed: false,
    isDragging: false,
    dragTargetPlaylistId: '',
    scanningMessage: '',
    scanProgress: {
      active: false,
      current: 0,
      total: 0,
      message: '',
      taskId: null,
    } as ScanProgress,
    _scanUnlisten: null as (() => void) | null,
  }),

  actions: {
    async setupScanListener() {
      if (this._scanUnlisten) return

      const unlistenProgress = await listen<{
        task_id: string
        scanned_files: number
        matched_audio_files: number
        added: number
        skipped_duplicate: number
        failed: number
        current_path?: string
      }>('scan://progress', (event) => {
        const p = event.payload
        this.scanProgress = {
          active: true,
          current: p.matched_audio_files || p.scanned_files,
          total: Math.max(p.matched_audio_files || p.scanned_files, 1),
          message: `正在添加音乐 ${p.added + p.skipped_duplicate}/${p.matched_audio_files || '...'}`,
          taskId: p.task_id,
        }
      })

      const unlistenFinished = await listen<{
        task_id: string
        added: number
        skipped_duplicate: number
        failed: number
      }>('scan://finished', (event) => {
        const p = event.payload
        this.scanProgress.active = false
        this.scanningMessage = `已添加 ${p.added} 首${p.skipped_duplicate > 0 ? `，跳过 ${p.skipped_duplicate} 首重复` : ''}`
        setTimeout(() => { this.scanningMessage = '' }, 4000)
      })

      const unlistenStarted = await listen<{ task_id: string }>('scan://started', (event) => {
        this.scanProgress = {
          active: true,
          current: 0,
          total: 0,
          message: '正在扫描文件夹...',
          taskId: event.payload.task_id,
        }
      })

      const unlistenCancelled = await listen('scan://cancelled', () => {
        this.scanProgress.active = false
        this.scanningMessage = '已取消添加'
        setTimeout(() => { this.scanningMessage = '' }, 2000)
      })

      this._scanUnlisten = () => {
        unlistenProgress()
        unlistenFinished()
        unlistenStarted()
        unlistenCancelled()
      }
    },

    async cancelScan() {
      if (this.scanProgress.taskId) {
        try {
          await invoke('scan_cancel', { taskId: this.scanProgress.taskId })
        } catch {}
      }
    },

    async loadConfig() {
      const playlistStore = usePlaylistStore()
      const playerStore = usePlayerStore()

      await this.setupScanListener()

      try {
        const settings = await invoke<Record<string, string>>('settings_get')
        if (settings.theme === 'light' || settings.theme === 'dark') {
          this.theme = settings.theme
        }

        const pbState = await invoke<PlaybackStateDto>('settings_get_playback_state')
        if (pbState.volume !== undefined) {
          playerStore.volume = pbState.volume
        }
        if (pbState.play_mode) {
          playerStore.playMode = pbState.play_mode as any
        }

        await playlistStore.init(pbState.current_playlist_id)

        let restoreTrack = null
        if (pbState.current_song_id) {
          restoreTrack = playlistStore.currentTracks.find(
            (t) => t.id === pbState.current_song_id
          ) || null
        }

        const ww = settings.window_width ? parseInt(settings.window_width) : null
        const wh = settings.window_height ? parseInt(settings.window_height) : null
        const wx = settings.window_x ? parseInt(settings.window_x) : null
        const wy = settings.window_y ? parseInt(settings.window_y) : null

        if (ww && wh) {
          try {
            const win = getCurrentWindow()
            await win.setSize(new LogicalSize(ww, wh))
            if (wx != null && wy != null) {
              await win.setPosition(new LogicalPosition(wx, wy))
            }
          } catch {}
        }

        // 恢复上次播放位置（不自动播放）
        if (restoreTrack) {
          await playerStore.restoreSession(restoreTrack, pbState.position_ms || 0)
        }

        // 后台检查当前列表缺失文件
        playlistStore.checkCurrentTracksExist()
      } catch (e) {
        console.warn('加载配置失败:', e)
        await playlistStore.init()
      }
    },

    async saveConfig() {
      const playlistStore = usePlaylistStore()
      const playerStore = usePlayerStore()

      let windowWidth = 960
      let windowHeight = 640
      let windowX: number | null = null
      let windowY: number | null = null

      try {
        const win = getCurrentWindow()
        const size = await win.outerSize()
        windowWidth = size.width
        windowHeight = size.height
        const pos = await win.outerPosition()
        windowX = pos.x
        windowY = pos.y
      } catch {}

      const settings: Record<string, string> = {
        theme: this.theme,
        window_width: String(windowWidth),
        window_height: String(windowHeight),
      }
      if (windowX != null) settings.window_x = String(windowX)
      if (windowY != null) settings.window_y = String(windowY)

      try {
        await invoke('settings_update', { settings })
      } catch (e) {
        console.warn('保存设置失败:', e)
      }

      try {
        await invoke('settings_save_playback_state', {
          playbackState: {
            current_playlist_id: playlistStore.currentPlaylistId || null,
            current_song_id: playerStore.currentTrack?.id || null,
            position_ms: Math.round(playerStore.currentTime * 1000),
            volume: playerStore.volume,
            muted: playerStore.muted,
            play_mode: playerStore.playMode,
          } as PlaybackStateDto,
        })
      } catch (e) {
        console.warn('保存播放状态失败:', e)
      }
    },

    toggleTheme() {
      this.theme = this.theme === 'dark' ? 'light' : 'dark'
      this.saveConfig()
    },

    setTheme(theme: Theme) {
      this.theme = theme
      this.saveConfig()
    },

    async hideToTray() {
      try {
        await getCurrentWindow().hide()
      } catch {}
    },

    toggleMiniMode() {
      this.isMiniMode = !this.isMiniMode
      try {
        const win = getCurrentWindow()
        if (this.isMiniMode) {
          win.setSize(new LogicalSize(380, 80))
          win.setMinSize(new LogicalSize(300, 60))
          win.setMaximizable(false)
        } else {
          win.setSize(new LogicalSize(960, 640))
          win.setMinSize(new LogicalSize(640, 420))
          win.setMaximizable(true)
        }
      } catch {}
    },

    async toggleAlwaysOnTop() {
      this.isAlwaysOnTop = !this.isAlwaysOnTop
      try {
        const win = getCurrentWindow()
        await win.setAlwaysOnTop(this.isAlwaysOnTop)
      } catch {}
    },

    setSearchQuery(query: string) {
      this.searchQuery = query
    },
  },
})
