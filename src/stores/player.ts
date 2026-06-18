import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { usePlaylistStore } from './playlist'
import type { PlayMode, Track, AudioStateDto } from '../types'

export const usePlayerStore = defineStore('player', {
  state: () => ({
    currentTrack: null as Track | null,
    isPlaying: false,
    currentTime: 0,
    duration: 0,
    volume: 0.8,
    muted: false,
    playMode: 'list_loop' as PlayMode,
    error: '',
    isLoading: false,
    isSeeking: false,
    seekTargetTime: 0,
    _unlisten: [] as (() => void)[],
    /** 随机模式：当前轮次剩余曲目索引 */
    _shuffleQueue: [] as number[],
  }),

  getters: {
    progress(state): number {
      if (!state.duration || state.duration <= 0) return 0
      return (state.currentTime / state.duration) * 100
    },
  },

  actions: {
    /** 初始化：注册后端事件监听 */
    async init() {
      const unlistenState = await listen<AudioStateDto>(
        'audio://state',
        (event) => {
          const p = event.payload
          if (!this.isSeeking) {
            this.currentTime = p.position_ms / 1000
          }
          if (p.duration_ms > 0) {
            this.duration = p.duration_ms / 1000
          }
          this.isPlaying = p.status === 'playing'
          this.volume = p.volume
          this.muted = p.muted
          if (p.play_mode) {
            this.playMode = p.play_mode
          }
          if (p.error) {
            this.error = p.error
          }
        }
      )

      const unlistenEnded = await listen<{ song_id: number; path: string; reason: string }>(
        'audio://ended',
        () => {
          this.isPlaying = false
          this.playNext()
        }
      )

      const unlistenErr = await listen<{ song_id: number; path: string; code: string; message: string }>(
        'audio://error',
        (event) => {
          this.showSkipError(event.payload.message || '播放失败')
        }
      )

      const unlistenRecovered = await listen<{ song_id: number; path: string }>(
        'audio://device_recovered',
        () => {
          this.error = ''
          this.isLoading = false
        }
      )

      this._unlisten.push(unlistenState, unlistenEnded, unlistenErr, unlistenRecovered)
      try { await invoke('audio_set_volume', { volume: this.volume }) } catch {}
    },

    showSkipError(msg: string) {
      this.error = msg.includes('跳过') ? msg : `${msg}，已跳过`
      this.isPlaying = false
      this.isLoading = false
      setTimeout(() => {
        if (this.error) {
          this.error = ''
          this.playNext()
        }
      }, 2500)
    },

    /** 恢复上次会话：加载曲目与进度，保持暂停 */
    async restoreSession(track: Track, positionMs: number) {
      if (track.missing || !track.playable) return

      this.currentTrack = track
      this.isLoading = true
      this.error = ''

      try {
        const state = await invoke<AudioStateDto>('audio_play', {
          songId: track.id,
          path: track.path,
        })
        this.duration = state.duration_ms / 1000
        await invoke('audio_pause')
        this.isPlaying = false

        if (positionMs > 0) {
          const seconds = positionMs / 1000
          await invoke('audio_seek', { seconds })
          this.currentTime = seconds
        } else {
          this.currentTime = 0
        }
      } catch {
        // 恢复失败时仅保留选中状态
        this.currentTime = positionMs / 1000
        this.isPlaying = false
      } finally {
        this.isLoading = false
      }
    },

    /** 播放指定歌曲 */
    async playTrack(track: Track) {
      if (track.missing || !track.playable) {
        this.showSkipError(track.missing ? '文件不存在' : '无法播放此文件')
        return
      }

      this.error = ''
      this.currentTrack = track
      this.isLoading = true
      this.currentTime = 0
      this._shuffleQueue = []

      try {
        const state = await invoke<AudioStateDto>('audio_play', {
          songId: track.id,
          path: track.path,
        })
        this.duration = state.duration_ms / 1000
        this.isPlaying = state.status === 'playing'
        this.isLoading = false
      } catch (e: any) {
        this.showSkipError(e?.message || String(e) || '播放失败')
      }
    },

    /** 停止播放 */
    async stopPlayback() {
      try { await invoke('audio_stop') } catch {}
      this.isPlaying = false
      this.isLoading = false
      this.currentTime = 0
      this.duration = 0
      this.currentTrack = null
    },

    checkAndStopIfRemoved(trackId: number) {
      if (this.currentTrack && this.currentTrack.id === trackId) this.stopPlayback()
    },

    async togglePlay() {
      if (!this.currentTrack) {
        const tracks = usePlaylistStore().currentTracks
        if (tracks.length > 0) this.playTrack(tracks[0])
        return
      }
      try { await invoke('audio_toggle_pause') } catch {}
    },

    /** 构建随机播放队列（一轮内不重复） */
    buildShuffleQueue(tracks: Track[], currentIndex: number) {
      const indices = tracks.map((_, i) => i).filter((i) => i !== currentIndex)
      for (let i = indices.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1))
        ;[indices[i], indices[j]] = [indices[j], indices[i]]
      }
      this._shuffleQueue = indices
    },

    playNext() {
      const tracks = usePlaylistStore().currentTracks
      if (tracks.length === 0) return
      if (!this.currentTrack) { this.playTrack(tracks[0]); return }
      const ci = tracks.findIndex((t) => t.id === this.currentTrack!.id)
      switch (this.playMode) {
        case 'sequence':
          if (ci < tracks.length - 1) this.playTrack(tracks[ci + 1])
          break
        case 'single_loop':
          this.currentTime = 0
          this.playTrack(this.currentTrack)
          break
        case 'list_loop':
          this.playTrack(tracks[(ci + 1) % tracks.length])
          break
        case 'random': {
          if (tracks.length === 1) { this.playTrack(this.currentTrack); break }
          if (this._shuffleQueue.length === 0) {
            this.buildShuffleQueue(tracks, ci)
          }
          const nextIdx = this._shuffleQueue.shift()!
          this.playTrack(tracks[nextIdx])
          break
        }
      }
    },

    playPrev() {
      const tracks = usePlaylistStore().currentTracks
      if (tracks.length === 0 || !this.currentTrack) return

      // 播放超过 3 秒则回到本曲开头
      if (this.currentTime > 3) {
        this.currentTime = 0
        this.commitSeek(0)
        return
      }

      const ci = tracks.findIndex((t) => t.id === this.currentTrack!.id)
      this.playTrack(tracks[ci <= 0 ? tracks.length - 1 : ci - 1])
    },

    updateSeekPreview(time: number) {
      this.isSeeking = true
      this.seekTargetTime = time
      this.currentTime = time
    },

    async commitSeek(time: number) {
      try {
        await invoke('audio_seek', { seconds: time })
        this.currentTime = time
      } catch {}
      this.isSeeking = false
    },

    async seek(time: number) {
      try { await invoke('audio_seek', { seconds: time }) } catch {}
    },

    seekByPercent(p: number) { this.seek((p / 100) * this.duration) },

    async setVolume(vol: number) {
      this.volume = Math.max(0, Math.min(1, vol))
      try { await invoke('audio_set_volume', { volume: this.volume }) } catch {}
      if (this.volume > 0) this.muted = false
    },

    async toggleMute() {
      this.muted = !this.muted
      try { await invoke('audio_set_volume', { volume: this.muted ? 0 : this.volume }) } catch {}
    },

    setPlayMode(mode: PlayMode) {
      this.playMode = mode
      this._shuffleQueue = []
    },

    cyclePlayMode() {
      const modes: PlayMode[] = ['list_loop', 'single_loop', 'sequence', 'random']
      this.playMode = modes[(modes.indexOf(this.playMode) + 1) % modes.length]
      this._shuffleQueue = []
      invoke('audio_set_play_mode', { mode: this.playMode }).catch(() => {})
    },

    destroy() {
      this._unlisten.forEach((fn) => fn())
      this._unlisten = []
    },
  },
})
