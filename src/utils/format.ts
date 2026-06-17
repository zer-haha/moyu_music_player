/** 格式化秒数为 mm:ss */
export function formatTime(seconds: number): string {
  if (!seconds || !isFinite(seconds) || isNaN(seconds)) return '00:00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
}

/** 格式化文件大小 */
export function formatFileSize(bytes: number): string {
  if (!bytes || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  let size = bytes
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024
    i++
  }
  return `${size.toFixed(i > 0 ? 1 : 0)} ${units[i]}`
}

/** 获取播放模式名称 */
export function getPlayModeName(mode: string): string {
  const names: Record<string, string> = {
    'list_loop': '列表循环',
    'single_loop': '单曲循环',
    'sequence': '顺序播放',
    'random': '随机播放',
  }
  return names[mode] || mode
}

/** 获取播放模式图标（SVG path） */
export function getPlayModeIcon(mode: string): string {
  switch (mode) {
    case 'list_loop':
      return 'M17 1l4 4-4 4' // 循环箭头
    case 'single_loop':
      return 'M17 1l4 4-4 4M3 11V9a4 4 0 0 1 4-4h14'
    case 'sequence':
      return 'M5 12h14M12 5l7 7-7 7'
    case 'random':
      return 'M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5'
    default:
      return ''
  }
}

/** 生成简单的唯一 ID */
export function simpleId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substring(2, 8)
}

/** 从文件路径中提取文件名（不含扩展名） */
export function getFileNameWithoutExt(filePath: string): string {
  const parts = filePath.replace(/\\/g, '/').split('/')
  const fileName = parts[parts.length - 1] || ''
  const dotIndex = fileName.lastIndexOf('.')
  return dotIndex > 0 ? fileName.substring(0, dotIndex) : fileName
}

/** 获取文件扩展名 */
export function getFileExtension(filePath: string): string {
  const parts = filePath.split('.')
  return (parts[parts.length - 1] || '').toLowerCase()
}
