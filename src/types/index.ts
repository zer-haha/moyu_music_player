/** 歌曲记录（对应后端 SongDto） */
export interface Track {
  id: number
  path: string
  path_key: string
  file_name: string
  title: string
  artist: string | null
  album: string | null
  album_artist: string | null
  genre: string | null
  track_number: number | null
  disc_number: number | null
  duration_ms: number
  format: string | null
  extension: string | null
  size_bytes: number
  modified_time: number
  added_at: number
  updated_at: number
  last_played_at: number | null
  play_count: number
  playable: boolean
  missing: boolean
  last_error: string | null
}

/** 播放列表（对应后端 PlaylistDto） */
export interface Playlist {
  id: number
  name: string
  kind: string
  source_folder: string | null
  sort_order: number
  created_at: number
  updated_at: number
  tracks: Track[]
}

/** 播放状态（对应后端 AudioStateDto） */
export interface AudioStateDto {
  current_song_id: number | null
  path: string | null
  status: 'idle' | 'loading' | 'playing' | 'paused' | 'stopped' | 'ended' | 'error'
  position_ms: number
  duration_ms: number
  volume: number
  muted: boolean
  play_mode: PlayMode
  error: string | null
}

/** 播放模式 */
export type PlayMode = 'sequence' | 'list_loop' | 'single_loop' | 'random'

/** 主题 */
export type Theme = 'dark' | 'light'

/** 错误结构 */
export interface AppErrorDto {
  code: string
  message: string
  detail: string | null
  recoverable: boolean
}

/** 扫描结果 */
export interface AddResultDto {
  added: number
  skipped: number
  failed: number
}

/** 播放状态持久化 */
export interface PlaybackStateDto {
  current_playlist_id: number | null
  current_song_id: number | null
  position_ms: number
  volume: number
  muted: boolean
  play_mode: string
}
