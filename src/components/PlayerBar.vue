<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted } from 'vue'
import { usePlayerStore } from '../stores/player'
import { useAppStore } from '../stores/app'
import { formatTime, getPlayModeName } from '../utils/format'
import type { PlayMode } from '../types'

const playerStore = usePlayerStore()
const appStore = useAppStore()
const isDraggingProgress = ref(false)
const isDraggingVolume = ref(false)
const progressPercent = ref(0)

function calcProgress(e: MouseEvent, bar: HTMLElement): number {
  const rect = bar.getBoundingClientRect()
  return Math.max(0, Math.min(100, ((e.clientX - rect.left) / rect.width) * 100))
}

function onProgressMouseDown(e: MouseEvent) {
  isDraggingProgress.value = true
  const bar = e.currentTarget as HTMLElement
  const pct = calcProgress(e, bar)
  progressPercent.value = pct
  playerStore.updateSeekPreview((pct / 100) * playerStore.duration)

  const onMove = (ev: MouseEvent) => {
    const b = document.querySelector('.progress-bar') as HTMLElement; if (!b) return
    const p = calcProgress(ev, b)
    progressPercent.value = p
    playerStore.updateSeekPreview((p / 100) * playerStore.duration)
  }
  const onUp = () => {
    isDraggingProgress.value = false
    const finalPct = progressPercent.value
    playerStore.commitSeek((finalPct / 100) * playerStore.duration)
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
function onVolumeClick(e: MouseEvent) {
  const bar = e.currentTarget as HTMLElement; const rect = bar.getBoundingClientRect()
  playerStore.setVolume(Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width)))
}
function onVolumeMouseDown(e: MouseEvent) {
  isDraggingVolume.value = true; onVolumeClick(e)
  const onMove = (ev: MouseEvent) => {
    const bar = document.querySelector('.volume-bar') as HTMLElement; if (!bar) return
    const rect = bar.getBoundingClientRect()
    playerStore.setVolume(Math.max(0, Math.min(1, (ev.clientX - rect.left) / rect.width)))
  }
  const onUp = () => { isDraggingVolume.value = false; document.removeEventListener('mousemove', onMove); document.removeEventListener('mouseup', onUp) }
  document.addEventListener('mousemove', onMove); document.addEventListener('mouseup', onUp)
}

const displayTitle = computed(() => playerStore.currentTrack ? playerStore.currentTrack.title : '未播放')
const displayArtist = computed(() => playerStore.currentTrack?.artist || '')
const displayInfo = computed(() => displayArtist.value ? `${displayTitle.value} - ${displayArtist.value}` : displayTitle.value)

const titleContainer = ref<HTMLElement | null>(null)
const titleText = ref<HTMLElement | null>(null)
const needsMarquee = ref(false)
function checkOverflow() {
  nextTick(() => {
    if (titleContainer.value && titleText.value)
      needsMarquee.value = titleText.value.scrollWidth > titleContainer.value.clientWidth
  })
}
watch(() => displayInfo.value, () => { needsMarquee.value = false; setTimeout(checkOverflow, 50) })
onMounted(() => { checkOverflow(); window.addEventListener('resize', checkOverflow) })

function playModeIcon(mode: PlayMode): string {
  switch (mode) {
    case 'list_loop': return 'M17 2l4 4-4 4M3 11V9a4 4 0 0 1 4-4h14M7 22l-4-4 4-4M21 13v2a4 4 0 0 1-4 4H3'
    case 'single_loop': return 'M17 2l4 4-4 4M3 11V9a4 4 0 0 1 4-4h14M7 22l-4-4 4-4M21 13v2a4 4 0 0 1-4 4H3M12 7v8M10 13h4'
    case 'sequence': return 'M5 12h14M12 5l7 7-7 7'
    case 'random': return 'M16 3h5v5M4 20L21 3M21 16v5h-5M15 15l6 6M4 4l5 5'
    default: return ''
  }
}
const volumePercent = computed(() => Math.round(playerStore.volume * 100))
const showVolumeIcon = computed(() => {
  if (playerStore.muted || playerStore.volume === 0) return 'muted'
  return playerStore.volume < 0.5 ? 'low' : 'high'
})
</script>

<template>
  <footer class="player-bar">
    <div class="player-info">
      <div class="now-playing-icon">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>
        </svg>
      </div>
      <div ref="titleContainer" class="track-info">
        <span ref="titleText" class="track-title" :class="{ marquee: needsMarquee }" :title="displayInfo">
          <span v-if="needsMarquee" class="marquee-inner">
            {{ displayInfo }}<span class="marquee-gap">&nbsp;&nbsp;&nbsp;&nbsp;</span>{{ displayInfo }}
          </span>
          <template v-else>{{ displayTitle }}</template>
        </span>
        <span v-if="!needsMarquee && displayArtist" class="track-artist">{{ displayArtist }}</span>
      </div>
      <span v-if="playerStore.error" class="player-error">{{ playerStore.error }}</span>
    </div>
    <div class="player-controls-wrapper">
      <div class="player-controls">
        <button class="ctrl-btn" :title="'播放模式: ' + getPlayModeName(playerStore.playMode)" @click="playerStore.cyclePlayMode()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path :d="playModeIcon(playerStore.playMode)"/></svg>
        </button>
        <button class="ctrl-btn" title="上一首" @click="playerStore.playPrev()">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><polygon points="19 20 9 12 19 4 19 20"/><line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" stroke-width="2.5"/></svg>
        </button>
        <button class="ctrl-btn play-btn" title="播放/暂停" @click="playerStore.togglePlay()">
          <svg v-if="!playerStore.isPlaying" width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><polygon points="6 3 20 12 6 21 6 3"/></svg>
          <svg v-else width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/></svg>
        </button>
        <button class="ctrl-btn" title="下一首" @click="playerStore.playNext()">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19" stroke="currentColor" stroke-width="2.5"/></svg>
        </button>
      </div>
      <div class="progress-section">
        <span class="time">{{ formatTime(playerStore.currentTime) }}</span>
        <div class="progress-bar" @mousedown="onProgressMouseDown">
          <div class="progress-track"><div class="progress-fill" :style="{ width: (isDraggingProgress ? progressPercent : playerStore.progress) + '%' }"><div class="progress-thumb"></div></div></div>
        </div>
        <span class="time">{{ formatTime(playerStore.duration) }}</span>
      </div>
    </div>
    <div class="player-volume">
      <button class="ctrl-btn" @click="playerStore.toggleMute()" :title="playerStore.muted ? '取消静音' : '静音'">
        <svg v-if="showVolumeIcon === 'muted'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>
        <svg v-else-if="showVolumeIcon === 'low'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
      </button>
      <div class="volume-bar" @mousedown="onVolumeMouseDown"><div class="volume-track"><div class="volume-fill" :style="{ width: (playerStore.muted ? 0 : playerStore.volume * 100) + '%' }"></div></div></div>
      <span class="vol-text">{{ playerStore.muted ? 0 : volumePercent }}%</span>
    </div>
  </footer>
</template>

<style scoped>
.player-bar { height: 80px; display: flex; align-items: center; padding: 0 16px; gap: 16px; background: var(--bg-player); border-top: 1px solid var(--border-color); flex-shrink: 0; }
.player-info { width: 220px; min-width: 120px; display: flex; align-items: center; gap: 10px; }
.now-playing-icon { flex-shrink: 0; color: var(--accent); opacity: 0.7; }
.track-info { display: flex; flex-direction: column; gap: 2px; min-width: 0; overflow: hidden; }
.track-title { font-size: 13px; font-weight: 500; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.track-title.marquee { overflow: hidden; text-overflow: unset; }
.marquee-inner { display: inline-block; animation: marqueeScroll 8s linear infinite; white-space: nowrap; }
.marquee-gap { display: inline-block; width: 4em; }
@keyframes marqueeScroll { 0% { transform: translateX(0); } 100% { transform: translateX(-50%); } }
.track-artist { font-size: 11px; color: var(--text-secondary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.player-error { font-size: 11px; color: #ef4444; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.player-controls-wrapper { flex: 1; display: flex; flex-direction: column; align-items: center; gap: 4px; }
.player-controls { display: flex; align-items: center; gap: 6px; }
.ctrl-btn { display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; border: none; background: transparent; color: var(--text-secondary); border-radius: 50%; cursor: pointer; transition: all 0.15s; }
.ctrl-btn:hover { color: var(--text-primary); background: var(--bg-hover); }
.ctrl-btn.play-btn { width: 38px; height: 38px; color: var(--text-primary); background: var(--bg-hover); }
.ctrl-btn.play-btn:hover { background: var(--accent); color: white; transform: scale(1.05); }
.progress-section { display: flex; align-items: center; gap: 8px; width: 100%; max-width: 520px; }
.time { font-size: 11px; color: var(--text-muted); font-variant-numeric: tabular-nums; min-width: 36px; text-align: center; }
.progress-bar { flex: 1; height: 16px; display: flex; align-items: center; cursor: pointer; }
.progress-track { width: 100%; height: 4px; background: var(--bg-progress); border-radius: 2px; overflow: visible; position: relative; }
.progress-fill { height: 100%; background: var(--accent); border-radius: 2px; position: relative; transition: width 0.1s linear; }
.progress-thumb { position: absolute; right: -5px; top: 50%; transform: translateY(-50%); width: 10px; height: 10px; background: var(--accent); border-radius: 50%; opacity: 0; transition: opacity 0.15s; }
.progress-bar:hover .progress-thumb { opacity: 1; }
.player-volume { display: flex; align-items: center; gap: 6px; width: 140px; min-width: 100px; }
.volume-bar { flex: 1; height: 16px; display: flex; align-items: center; cursor: pointer; }
.volume-track { width: 100%; height: 4px; background: var(--bg-progress); border-radius: 2px; overflow: hidden; }
.volume-fill { height: 100%; background: var(--text-secondary); border-radius: 2px; transition: width 0.1s; }
.volume-bar:hover .volume-fill { background: var(--accent); }
.vol-text { font-size: 11px; color: var(--text-muted); min-width: 30px; text-align: right; font-variant-numeric: tabular-nums; }
</style>
