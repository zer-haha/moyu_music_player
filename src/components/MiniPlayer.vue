<script setup lang="ts">
import { computed, ref, watch, nextTick, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { usePlayerStore } from '../stores/player'
import { useAppStore } from '../stores/app'
import { formatTime } from '../utils/format'

const playerStore = usePlayerStore()
const appStore = useAppStore()

const displayTitle = computed(() => {
  const t = playerStore.currentTrack
  if (!t) return '墨鱼听歌'
  return t.artist ? `${t.title} - ${t.artist}` : t.title
})

const titleRef = ref<HTMLElement | null>(null)
const titleText = ref<HTMLElement | null>(null)
const needsMarquee = ref(false)

function checkOverflow() {
  nextTick(() => {
    if (titleRef.value && titleText.value)
      needsMarquee.value = titleText.value.scrollWidth > titleRef.value.clientWidth
  })
}
watch(() => displayTitle.value, () => { needsMarquee.value = false; setTimeout(checkOverflow, 50) })
onMounted(() => { checkOverflow(); window.addEventListener('resize', checkOverflow) })

function startDrag(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.closest('button')) return
  try { getCurrentWindow().startDragging() } catch {}
}

async function hideToTray() {
  try { await getCurrentWindow().close() } catch {}
}

function onProgressMouseDown(e: MouseEvent) {
  e.stopPropagation()
  const bar = e.currentTarget as HTMLElement; const rect = bar.getBoundingClientRect()
  const pct = Math.max(0, Math.min(100, ((e.clientX - rect.left) / rect.width) * 100))
  playerStore.updateSeekPreview((pct / 100) * playerStore.duration)

  const onMove = (ev: MouseEvent) => {
    const p = Math.max(0, Math.min(100, ((ev.clientX - rect.left) / rect.width) * 100))
    playerStore.updateSeekPreview((p / 100) * playerStore.duration)
  }
  const onUp = (ev: MouseEvent) => {
    const finalPct = Math.max(0, Math.min(100, ((ev.clientX - rect.left) / rect.width) * 100))
    playerStore.commitSeek((finalPct / 100) * playerStore.duration)
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
</script>

<template>
  <div class="mini-player" @mousedown="startDrag">
    <div class="mini-info">
      <div ref="titleRef" class="mini-title-wrap">
        <span ref="titleText" class="mini-title" :class="{ marquee: needsMarquee }" :title="displayTitle">
          <span v-if="needsMarquee" class="marquee-inner">
            {{ displayTitle }}<span class="marquee-gap">&nbsp;&nbsp;&nbsp;&nbsp;</span>{{ displayTitle }}
          </span>
          <template v-else>{{ displayTitle }}</template>
        </span>
      </div>
      <span class="mini-time">{{ formatTime(playerStore.currentTime) }} / {{ formatTime(playerStore.duration) }}</span>
    </div>
    <div class="mini-controls" @mousedown.stop>
      <button class="mini-btn" @click="playerStore.playPrev()">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="19 20 9 12 19 4 19 20"/><line x1="5" y1="19" x2="5" y2="5" stroke="currentColor" stroke-width="3"/></svg>
      </button>
      <button class="mini-btn mini-play" @click="playerStore.togglePlay()">
        <svg v-if="!playerStore.isPlaying" width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><polygon points="6 3 20 12 6 21 6 3"/></svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/></svg>
      </button>
      <button class="mini-btn" @click="playerStore.playNext()">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19" stroke="currentColor" stroke-width="3"/></svg>
      </button>
    </div>
    <div class="mini-actions" @mousedown.stop>
      <button class="mini-btn" title="展开" @click="appStore.toggleMiniMode()">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/>
          <line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/>
        </svg>
      </button>
      <button class="mini-btn mini-close" title="关闭" @click="hideToTray">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="6" y1="6" x2="18" y2="18"/><line x1="6" y1="18" x2="18" y2="6"/>
        </svg>
      </button>
    </div>
    <div class="mini-progress" @mousedown.stop="onProgressMouseDown">
      <div class="mini-progress-track"><div class="mini-progress-fill" :style="{ width: playerStore.progress + '%' }"></div></div>
    </div>
  </div>
</template>

<style scoped>
.mini-player { height: 100vh; width: 100vw; display: flex; align-items: center; gap: 6px; padding: 0 8px; background: var(--bg-primary); user-select: none; -webkit-user-select: none; position: relative; }
.mini-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
.mini-title-wrap { overflow: hidden; }
.mini-title { font-size: 12px; color: var(--text-primary); white-space: nowrap; line-height: 1.3; }
.mini-title.marquee { overflow: hidden; text-overflow: unset; }
.marquee-inner { display: inline-block; animation: miniMarquee 8s linear infinite; white-space: nowrap; }
.marquee-gap { display: inline-block; width: 4em; }
@keyframes miniMarquee { 0% { transform: translateX(0); } 100% { transform: translateX(-50%); } }
.mini-time { font-size: 10px; color: var(--text-muted); font-variant-numeric: tabular-nums; }
.mini-controls { display: flex; align-items: center; gap: 0; flex-shrink: 0; }
.mini-btn { display: flex; align-items: center; justify-content: center; width: 28px; height: 28px; border: none; background: transparent; color: var(--text-secondary); border-radius: 50%; cursor: pointer; transition: all 0.15s; }
.mini-btn:hover { color: var(--text-primary); background: var(--bg-hover); }
.mini-play { color: var(--text-primary); }
.mini-close:hover { background: #e81123; color: white; }
.mini-actions { display: flex; align-items: center; gap: 0; flex-shrink: 0; }
.mini-progress { position: absolute; bottom: 0; left: 0; right: 0; height: 3px; display: flex; align-items: center; cursor: pointer; }
.mini-progress:hover { height: 5px; }
.mini-progress-track { width: 100%; height: 100%; background: var(--bg-progress); overflow: hidden; }
.mini-progress-fill { height: 100%; background: var(--accent); transition: width 0.1s; }
</style>
