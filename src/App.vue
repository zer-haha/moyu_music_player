<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import AppLayout from './components/AppLayout.vue'
import MiniPlayer from './components/MiniPlayer.vue'
import { useAppStore } from './stores/app'
import { usePlayerStore } from './stores/player'

const appStore = useAppStore()
const playerStore = usePlayerStore()
let unlistenMedia: (() => void) | null = null

function onGlobalKeyDown(e: KeyboardEvent) {
  const target = e.target as HTMLElement
  const tag = target.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA') return

  if (e.code === 'Space') {
    const el = document.activeElement as HTMLElement | null
    if (el?.classList.contains('track-list-container')) return
    e.preventDefault()
    playerStore.togglePlay()
  }
}

onMounted(async () => {
  playerStore.init()
  await appStore.loadConfig()

  document.addEventListener('keydown', onGlobalKeyDown)

  unlistenMedia = await listen<string>('app://media_key', (event) => {
    const action = event.payload
    if (action === 'toggle') playerStore.togglePlay()
    else if (action === 'next') playerStore.playNext()
    else if (action === 'prev') playerStore.playPrev()
  })
})

onUnmounted(() => {
  document.removeEventListener('keydown', onGlobalKeyDown)
  if (unlistenMedia) unlistenMedia()
  appStore.saveConfig()
  playerStore.destroy()
})
</script>

<template>
  <div id="app-root" :class="['app-root', appStore.theme]" @contextmenu.prevent>
    <MiniPlayer v-if="appStore.isMiniMode" />
    <AppLayout v-else />
  </div>
</template>
