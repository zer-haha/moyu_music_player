import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize, LogicalPosition } from '@tauri-apps/api/dpi'
import { usePlaylistStore } from './playlist'
import { usePlayerStore } from './player'
import type { Theme, PlaybackStateDto } from '../types'

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
  }),

  actions: {
    async loadConfig() {
      const playlistStore = usePlaylistStore()
      const playerStore = usePlayerStore()

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

        await playlistStore.init()

        if (pbState.current_playlist_id) {
          playlistStore.setCurrentPlaylist(pbState.current_playlist_id)
        }

        if (pbState.current_song_id) {
          const allTracks = playlistStore.allTracks
          const track = allTracks.find((t) => t.id === pbState.current_song_id)
          if (track) {
            playerStore.currentTrack = track
          }
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
