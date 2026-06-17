<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import AppLayout from './components/AppLayout.vue'
import MiniPlayer from './components/MiniPlayer.vue'
import { useAppStore } from './stores/app'
import { usePlayerStore } from './stores/player'

const appStore = useAppStore()
const playerStore = usePlayerStore()
let unlistenClose: (() => void) | null = null

onMounted(async () => {
  playerStore.init()
  await appStore.loadConfig()

  // Save config before closing, then let the window close normally
  unlistenClose = await getCurrentWindow().onCloseRequested(async (event) => {
    await appStore.saveConfig()
    // Don't prevent default — window closes and app exits
  })
})

onUnmounted(() => {
  playerStore.destroy()
  if (unlistenClose) unlistenClose()
})
</script>

<template>
  <div id="app-root" :class="['app-root', appStore.theme]" @contextmenu.prevent>
    <MiniPlayer v-if="appStore.isMiniMode" />
    <AppLayout v-else />
  </div>
</template>
