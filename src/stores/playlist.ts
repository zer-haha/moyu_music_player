import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Playlist, Track, AddResultDto } from '../types'

export const usePlaylistStore = defineStore('playlist', {
  state: () => ({
    playlists: [] as Playlist[],
    currentPlaylistId: 0,
  }),

  getters: {
    currentPlaylist(state): Playlist | undefined {
      return state.playlists.find((p) => p.id === state.currentPlaylistId)
    },

    currentTracks(state): Track[] {
      const pl = state.playlists.find((p) => p.id === state.currentPlaylistId)
      return pl?.tracks || []
    },

    allTracks(state): Track[] {
      return state.playlists.flatMap((p) => p.tracks)
    },
  },

  actions: {
    /** 从后端加载所有播放列表和歌曲 */
    async init() {
      try {
        const playlists = await invoke<Playlist[]>('playlist_get_all')
        this.playlists = playlists.map((pl: any) => ({
          ...pl,
          tracks: [],
        }))

        // Load songs for each playlist
        for (const pl of this.playlists) {
          try {
            const songs = await invoke<Track[]>('playlist_get_songs', { playlistId: pl.id })
            pl.tracks = songs
          } catch {
            pl.tracks = []
          }
        }

        // Set current playlist to first one (default playlist)
        if (this.playlists.length > 0 && !this.currentPlaylistId) {
          this.currentPlaylistId = this.playlists[0].id
        }
      } catch (e) {
        console.error('Failed to load playlists:', e)
        // Fallback: ensure at least one playlist exists
        if (this.playlists.length === 0) {
          this.playlists.push({
            id: 1,
            name: '默认播放列表',
            kind: 'default',
            source_folder: null,
            sort_order: 0,
            created_at: 0,
            updated_at: 0,
            tracks: [],
          })
          this.currentPlaylistId = 1
        }
      }
    },

    setCurrentPlaylist(id: number) {
      if (this.playlists.some((p) => p.id === id)) {
        this.currentPlaylistId = id
      }
    },

    /** 创建新播放列表 */
    async createPlaylist(name: string): Promise<number> {
      try {
        const pl = await invoke<Playlist>('playlist_create', { name })
        this.playlists.push({ ...pl, tracks: [] })
        return pl.id
      } catch (e) {
        console.error('Failed to create playlist:', e)
        return 0
      }
    },

    /** 删除播放列表（默认列表不可删除） */
    async removePlaylist(id: number) {
      const pl = this.playlists.find((p) => p.id === id)
      if (!pl || pl.kind === 'default') return

      try {
        await invoke('playlist_delete', { id })
        const idx = this.playlists.findIndex((p) => p.id === id)
        if (idx !== -1) this.playlists.splice(idx, 1)
        if (this.currentPlaylistId === id) {
          this.currentPlaylistId = this.playlists[0]?.id || 0
        }
      } catch (e) {
        console.error('Failed to delete playlist:', e)
      }
    },

    /** 重命名播放列表 */
    async renamePlaylist(id: number, name: string) {
      try {
        const updated = await invoke<Playlist>('playlist_rename', { id, name })
        const pl = this.playlists.find((p) => p.id === id)
        if (pl) pl.name = updated.name
      } catch (e) {
        console.error('Failed to rename playlist:', e)
      }
    },

    /** 清空播放列表 */
    async clearPlaylist(id: number) {
      try {
        await invoke('playlist_clear', { id })
        const pl = this.playlists.find((p) => p.id === id)
        if (pl) pl.tracks = []
      } catch (e) {
        console.error('Failed to clear playlist:', e)
      }
    },

    /** 添加歌曲到指定播放列表（通过文件路径） */
    async addFilesToPlaylist(playlistId: number, filePaths: string[]): Promise<number> {
      try {
        const result = await invoke<AddResultDto>('library_add_files', {
          paths: filePaths,
          playlistId,
        })
        // Reload songs for this playlist
        const songs = await invoke<Track[]>('playlist_get_songs', { playlistId })
        const pl = this.playlists.find((p) => p.id === playlistId)
        if (pl) pl.tracks = songs
        return result.added
      } catch (e) {
        console.error('Failed to add files:', e)
        return 0
      }
    },

    /** 添加歌曲到当前播放列表 */
    async addFilesToCurrent(filePaths: string[]): Promise<number> {
      return this.addFilesToPlaylist(this.currentPlaylistId, filePaths)
    },

    /** 从播放列表中移除歌曲 */
    async removeTrack(playlistId: number, trackId: number) {
      try {
        await invoke('library_remove_song_from_playlist', { playlistId, songId: trackId })
        const pl = this.playlists.find((p) => p.id === playlistId)
        if (pl) {
          const idx = pl.tracks.findIndex((t) => t.id === trackId)
          if (idx !== -1) pl.tracks.splice(idx, 1)
        }
      } catch (e) {
        console.error('Failed to remove track:', e)
      }
    },

    /** 拖动排序播放列表 */
    movePlaylist(fromIndex: number, toIndex: number) {
      const [pl] = this.playlists.splice(fromIndex, 1)
      if (pl) this.playlists.splice(toIndex, 0, pl)
    },

    /** 拖动排序歌曲 */
    async reorderTracks(playlistId: number, fromIndex: number, toIndex: number) {
      const pl = this.playlists.find((p) => p.id === playlistId)
      if (!pl) return
      const [track] = pl.tracks.splice(fromIndex, 1)
      if (track) pl.tracks.splice(toIndex, 0, track)
      // Persist new order to backend
      const songIds = pl.tracks.map((t) => t.id)
      try {
        await invoke('playlist_song_reorder', { playlistId, songIds })
      } catch (e) {
        console.error('Failed to reorder:', e)
      }
    },

    /** 从文件路径查找歌曲 */
    findTrackByPath(path: string): { playlist: Playlist; track: Track } | null {
      for (const pl of this.playlists) {
        const track = pl.tracks.find((t) => t.path === path)
        if (track) return { playlist: pl, track }
      }
      return null
    },

    getPlaylistTrackCount(id: number): number {
      return this.playlists.find((p) => p.id === id)?.tracks.length || 0
    },
  },
})
