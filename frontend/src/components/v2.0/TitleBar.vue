<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Window } from "@wailsio/runtime"
import SearchBar from './SearchBar.vue'
import ActionMenu from './ActionMenu.vue'

const searchQuery = ref('')
const isMaximized = ref(false)

const toggleMaximize = async () => {
  try {
    await Window.ToggleMaximise()
    isMaximized.value = await Window.IsMaximised()
  } catch (e) {
    console.error(e)
  }
}

const handleDoubleClick = () => {
  toggleMaximize()
}

onMounted(async () => {
  try {
    isMaximized.value = await Window.IsMaximised()
  } catch (e) {
    console.error(e)
  }
})
</script>

<template>
  <header 
    class="titlebar"
    @dblclick="handleDoubleClick"
  >
    <!-- Background drag region sits behind interactive elements -->
    <div class="titlebar-drag-region"></div>

    <!-- Left: Brand Logo (with mac-traffic-lights spacer) -->
    <div class="titlebar-section left-section">
      <div class="mac-traffic-lights-spacer"></div>
    </div>

    <!-- Center: SearchBar + ActionMenu -->
    <div class="titlebar-section center-section flex-1 max-w-lg justify-center gap-1.5">
      <SearchBar v-model="searchQuery" class="!mr-0" />
      <div class="h-4 w-px bg-border flex-shrink-0"></div>
      <ActionMenu class="!ml-2" />
    </div>

    <!-- Right: Spacer to balance mac traffic lights -->
    <div class="titlebar-section right-section">
      <div class="right-spacer"></div>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 44px;
  background: var(--background);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  z-index: 1000;
  user-select: none;
  -webkit-base-select: none;
  -webkit-user-select: none;
}

/* Background drag region */
.titlebar-drag-region {
  --wails-draggable: drag;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1;
}

.titlebar-section {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
}

.left-section {
  gap: 12px;
}

.right-section {
  gap: 12px;
}

.mac-traffic-lights-spacer {
  width: 75px;
  height: 100%;
  flex-shrink: 0;
}

.right-spacer {
  width: 75px;
  height: 100%;
  flex-shrink: 0;
}
</style>
