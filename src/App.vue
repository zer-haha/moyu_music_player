<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import AppLayout from './components/AppLayout.vue'
import MiniPlayer from './components/MiniPlayer.vue'
import { useAppStore } from './stores/app'
import { usePlayerStore } from './stores/player'

const appStore = useAppStore()
const playerStore = usePlayerStore()

onMounted(async () => {
  playerStore.init()
  await appStore.loadConfig()
})

onUnmounted(() => {
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
