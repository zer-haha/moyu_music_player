import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { Playlist, Track, AddResultDto } from '../types'

const MAX_CACHED_PLAYLISTS = 5

export const usePlaylistStore = defineStore('playlist', {
  state: () => ({
    playlists: [] as Playlist[],
    currentPlaylistId: 0,
    /** 已加载过歌曲的列表 ID */
    _loadedPlaylistIds: new Set<number>(),
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
    /** 从后端加载播放列表（懒加载歌曲） */
    async init(restorePlaylistId?: number | null) {
      try {
        const playlists = await invoke<Playlist[]>('playlist_get_all')
        this.playlists = playlists.map((pl: Playlist) => ({
          ...pl,
          tracks: [],
        }))
        this._loadedPlaylistIds = new Set()

        if (this.playlists.length === 0) {
          this.playlists.push({
            id: 1,
            name: '默认播放列表',
            kind: 'default',
            source_folder: null,
            sort_order: 0,
            created_at: 0,
            updated_at: 0,
            track_count: 0,
            tracks: [],
          })
          this.currentPlaylistId = 1
          return
        }

        const targetId =
          restorePlaylistId && this.playlists.some((p) => p.id === restorePlaylistId)
            ? restorePlaylistId
            : this.playlists[0].id

        this.currentPlaylistId = targetId
        await this.loadPlaylistSongs(targetId)
      } catch (e) {
        console.error('Failed to load playlists:', e)
        if (this.playlists.length === 0) {
          this.playlists.push({
            id: 1,
            name: '默认播放列表',
            kind: 'default',
            source_folder: null,
            sort_order: 0,
            created_at: 0,
            updated_at: 0,
            track_count: 0,
            tracks: [],
          })
          this.currentPlaylistId = 1
        }
      }
    },

    /** 加载指定列表的歌曲 */
    async loadPlaylistSongs(playlistId: number) {
      const pl = this.playlists.find((p) => p.id === playlistId)
      if (!pl) return

      if (this._loadedPlaylistIds.has(playlistId)) return

      try {
        const songs = await invoke<Track[]>('playlist_get_songs', { playlistId })
        pl.tracks = songs
        pl.track_count = songs.length
        this._loadedPlaylistIds.add(playlistId)
        this.evictOldCaches(playlistId)
      } catch {
        pl.tracks = []
      }
    },

    /** 缓存超过上限时释放非当前列表 */
    evictOldCaches(keepId: number) {
      if (this._loadedPlaylistIds.size <= MAX_CACHED_PLAYLISTS) return
      for (const id of Array.from(this._loadedPlaylistIds)) {
        if (id !== keepId && this._loadedPlaylistIds.size > MAX_CACHED_PLAYLISTS) {
          const pl = this.playlists.find((p) => p.id === id)
          if (pl) pl.tracks = []
          this._loadedPlaylistIds.delete(id)
        }
      }
    },

    async setCurrentPlaylist(id: number) {
      if (!this.playlists.some((p) => p.id === id)) return
      this.currentPlaylistId = id
      await this.loadPlaylistSongs(id)
    },

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

    async removePlaylist(id: number) {
      const pl = this.playlists.find((p) => p.id === id)
      if (!pl || pl.kind === 'default') return

      try {
        await invoke('playlist_delete', { id })
        const idx = this.playlists.findIndex((p) => p.id === id)
        if (idx !== -1) this.playlists.splice(idx, 1)
        this._loadedPlaylistIds.delete(id)
        if (this.currentPlaylistId === id) {
          this.currentPlaylistId = this.playlists[0]?.id || 0
          if (this.currentPlaylistId) await this.loadPlaylistSongs(this.currentPlaylistId)
        }
      } catch (e) {
        console.error('Failed to delete playlist:', e)
      }
    },

    async renamePlaylist(id: number, name: string) {
      try {
        const updated = await invoke<Playlist>('playlist_rename', { id, name })
        const pl = this.playlists.find((p) => p.id === id)
        if (pl) pl.name = updated.name
      } catch (e) {
        console.error('Failed to rename playlist:', e)
      }
    },

    async clearPlaylist(id: number) {
      try {
        await invoke('playlist_clear', { id })
        const pl = this.playlists.find((p) => p.id === id)
        if (pl) {
          pl.tracks = []
          pl.track_count = 0
        }
      } catch (e) {
        console.error('Failed to clear playlist:', e)
      }
    },

    async addFilesToPlaylist(playlistId: number, filePaths: string[]): Promise<number> {
      try {
        const result = await invoke<AddResultDto>('library_add_files', {
          paths: filePaths,
          playlistId,
        })
        await this.reloadPlaylistSongs(playlistId)
        return result.added
      } catch (e) {
        console.error('Failed to add files:', e)
        return 0
      }
    },

    async addFilesToCurrent(filePaths: string[]): Promise<number> {
      return this.addFilesToPlaylist(this.currentPlaylistId, filePaths)
    },

    async reloadPlaylistSongs(playlistId: number) {
      this._loadedPlaylistIds.delete(playlistId)
      await this.loadPlaylistSongs(playlistId)
    },

    async removeTrack(playlistId: number, trackId: number) {
      try {
        await invoke('library_remove_song_from_playlist', { playlistId, songId: trackId })
        const pl = this.playlists.find((p) => p.id === playlistId)
        if (pl) {
          const idx = pl.tracks.findIndex((t) => t.id === trackId)
          if (idx !== -1) pl.tracks.splice(idx, 1)
          pl.track_count = Math.max(0, (pl.track_count || pl.tracks.length) - 1)
        }
      } catch (e) {
        console.error('Failed to remove track:', e)
      }
    },

    async addTrackToPlaylist(playlistId: number, track: Track) {
      const pl = this.playlists.find((p) => p.id === playlistId)
      if (!pl) return
      if (pl.tracks.some((t) => t.id === track.id)) return
      try {
        await invoke('library_add_files', { paths: [track.path], playlistId })
        await this.reloadPlaylistSongs(playlistId)
      } catch (e) {
        console.error('Failed to add track to playlist:', e)
      }
    },

    async movePlaylist(fromIndex: number, toIndex: number) {
      const [pl] = this.playlists.splice(fromIndex, 1)
      if (pl) this.playlists.splice(toIndex, 0, pl)
      const ids = this.playlists.map((p) => p.id)
      try {
        await invoke('playlist_reorder', { playlistIds: ids })
      } catch (e) {
        console.error('Failed to reorder playlists:', e)
      }
    },

    async reorderTracks(playlistId: number, fromIndex: number, toIndex: number) {
      const pl = this.playlists.find((p) => p.id === playlistId)
      if (!pl) return
      const [track] = pl.tracks.splice(fromIndex, 1)
      if (track) pl.tracks.splice(toIndex, 0, track)
      const songIds = pl.tracks.map((t) => t.id)
      try {
        await invoke('playlist_song_reorder', { playlistId, songIds })
      } catch (e) {
        console.error('Failed to reorder:', e)
      }
    },

    findTrackByPath(path: string): { playlist: Playlist; track: Track } | null {
      for (const pl of this.playlists) {
        const track = pl.tracks.find((t) => t.path === path)
        if (track) return { playlist: pl, track }
      }
      return null
    },

    /** 检查当前列表文件是否存在，更新 missing 标记 */
    async checkCurrentTracksExist() {
      const pl = this.playlists.find((p) => p.id === this.currentPlaylistId)
      if (!pl || pl.tracks.length === 0) return

      const paths = pl.tracks.map((t) => t.path)
      try {
        const results = await invoke<[string, boolean][]>('check_files_exist', { filePaths: paths })
        const missingSet = new Set(
          results.filter(([, exists]) => !exists).map(([path]) => path)
        )
        for (const track of pl.tracks) {
          track.missing = missingSet.has(track.path)
        }
      } catch {}
    },

    getPlaylistTrackCount(id: number): number {
      return this.playlists.find((p) => p.id === id)?.tracks.length || 0
    },

    /** 选择文件夹并创建同名播放列表 */
    async importFolderAsNewPlaylist(folderPath: string): Promise<number> {
      const folderName =
        folderPath.replace(/\\/g, '/').split('/').filter(Boolean).pop() || '新播放列表'
      const plId = await this.createPlaylist(folderName)
      if (!plId) return 0

      try {
        const result = await invoke<{ added: number }>('scan_start_folder', {
          path: folderPath,
          playlistId: plId,
        })
        await this.reloadPlaylistSongs(plId)
        await this.setCurrentPlaylist(plId)
        return result.added
      } catch (e) {
        console.error('Failed to scan folder:', e)
        return 0
      }
    },

    /** 扫描文件夹到当前列表 */
    async importFolderToCurrent(folderPath: string): Promise<number> {
      try {
        const result = await invoke<{ added: number }>('scan_start_folder', {
          path: folderPath,
          playlistId: this.currentPlaylistId,
        })
        await this.reloadPlaylistSongs(this.currentPlaylistId)
        return result.added
      } catch (e) {
        console.error('Failed to scan folder:', e)
        return 0
      }
    },
  },
})
