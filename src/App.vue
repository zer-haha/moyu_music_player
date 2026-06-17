<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import AppLayout from './components/AppLayout.vue'
import MiniPlayer from './components/MiniPlayer.vue'
import { useAppStore } from './stores/app'
import { usePlayerStore } from './stores/player'

const appStore = useAppStore()
const playerStore = usePlayerStore()
const showCloseDialog = ref(false)
let unlistenClose: (() => void) | null = null

onMounted(async () => {
  playerStore.init()
  await appStore.loadConfig()

  // Use Tauri frontend API to intercept close
  unlistenClose = await getCurrentWindow().onCloseRequested((event) => {
    event.preventDefault()
    showCloseDialog.value = true
  })
})

onUnmounted(() => {
  playerStore.destroy()
  if (unlistenClose) unlistenClose()
})

async function onQuit() {
  showCloseDialog.value = false
  await appStore.saveConfig()
  await invoke('app_quit')
}

async function onMinimizeToTray() {
  showCloseDialog.value = false
  try { await getCurrentWindow().hide() } catch {}
}

function onCancelClose() {
  showCloseDialog.value = false
}
</script>

<template>
  <div id="app-root" :class="['app-root', appStore.theme]" @contextmenu.prevent>
    <MiniPlayer v-if="appStore.isMiniMode" />
    <AppLayout v-else />
    <Teleport to="body">
      <div v-if="showCloseDialog" class="close-dialog-overlay" @click="onCancelClose">
        <div class="close-dialog" @click.stop>
          <div class="close-dialog-title">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            <span>退出确认</span>
          </div>
          <p class="close-dialog-msg">请选择操作：</p>
          <div class="close-dialog-btns">
            <button class="cd-btn cd-quit" @click="onQuit">退出程序</button>
            <button class="cd-btn cd-tray" @click="onMinimizeToTray">最小化到托盘</button>
            <button class="cd-btn cd-cancel" @click="onCancelClose">取消</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style>
.close-dialog-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.45); display: flex;
  align-items: center; justify-content: center; z-index: 99999;
}
.close-dialog {
  background: var(--bg-menu, #2a2a2a); border: 1px solid var(--border-color, #3a3a3a);
  border-radius: 10px; padding: 20px 24px; min-width: 300px; max-width: 360px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.5); color: var(--text-primary, #e6e6e6);
}
.close-dialog-title {
  display: flex; align-items: center; gap: 8px; font-size: 15px; font-weight: 600;
  margin-bottom: 12px; color: var(--text-primary, #e6e6e6);
}
.close-dialog-title svg { color: var(--accent, #4da3ff); }
.close-dialog-msg { font-size: 13px; color: var(--text-secondary, #a8a8a8); margin: 0 0 18px; }
.close-dialog-btns { display: flex; flex-direction: column; gap: 8px; }
.cd-btn {
  padding: 10px 0; border: 1px solid var(--border-color, #3a3a3a); border-radius: 6px;
  font-size: 13px; cursor: pointer; transition: all 0.15s; background: transparent;
  color: var(--text-primary, #e6e6e6); text-align: center;
}
.cd-btn:hover { background: var(--bg-hover, rgba(255,255,255,0.06)); }
.cd-quit { border-color: #e81123; color: #ff6b6b; }
.cd-quit:hover { background: rgba(232,17,35,0.15); color: #ff4444; }
.cd-tray { border-color: var(--accent, #4da3ff); color: var(--accent, #4da3ff); }
.cd-tray:hover { background: rgba(77,163,255,0.12); }
</style>
