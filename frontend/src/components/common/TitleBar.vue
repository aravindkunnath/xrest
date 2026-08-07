<script setup lang="ts">
import { Button } from "@/components/ui/button"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"
import { useDialogState } from "@/composables/useDialogState"
import EnvironmentSelector from "@/features/environments/components/EnvironmentSelector.vue"
import { useServicesStore } from "@/stores/services"
import { useTabsStore } from "@/stores/tabs"
import { RiSettings4Line } from '@remixicon/vue'
import { System, Window } from "@wailsio/runtime"
import { computed, onMounted, ref } from 'vue'
import SearchBar from './SearchBar.vue'

const checkIsMac = (): boolean => {
  if (typeof window === 'undefined') return false
  try {
    if (typeof System !== 'undefined' && System && typeof System.IsMac === 'function' && System.IsMac()) return true
  } catch {}
  const wailsOS = (window as any)._wails?.environment?.OS
  if (wailsOS) return wailsOS === 'darwin' || wailsOS === 'macOS' || wailsOS === 'mac'
  const nav = window.navigator || {}
  const ua = nav.userAgent || ''
  const platform = (nav as any).userAgentData?.platform || nav.platform || ''
  return /Mac|Macintosh|MacIntel|MacPPC/i.test(ua) || /Mac/i.test(platform)
}

const searchQuery = ref('')
const isMaximized = ref(false)
const isMac = ref(checkIsMac())
const { openSettingsDialog } = useDialogState()

const servicesStore = useServicesStore()
const tabsStore = useTabsStore()

const activeTab = computed(() => tabsStore.tabs.find((t) => t.id === tabsStore.activeTab))
const activeService = computed(() => {
  if (!servicesStore.services?.length) return null
  if (activeTab.value?.serviceId) {
    return servicesStore.services.find((s) => s.id === activeTab.value?.serviceId) || servicesStore.services[0]
  }
  return servicesStore.services[0]
})

const isUnsafeEnv = computed(() => {
  if (!activeService.value) return false
  const envName = activeService.value.selectedEnvironment || activeService.value.environments[0]?.name
  const env = activeService.value.environments.find((e) => e.name === envName)
  return env?.isUnsafe ?? false
})

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

const goToSettings = () => {
  openSettingsDialog()
}

onMounted(async () => {
  try {
    isMaximized.value = await Window.IsMaximised()
  } catch (e) {
    console.error(e)
  }

  try {
    const env = await System.Environment()
    if (env?.OS) {
      isMac.value = env.OS === 'darwin' || env.OS === 'macOS' || env.OS === 'mac'
    }
  } catch {
    // Keep initial synchronous result
  }
})
</script>

<template>
  <header
    class="titlebar transition-colors duration-200"
    :class="{ 'is-unsafe-titlebar border-b-destructive/60 bg-destructive/10 border-t-2 border-t-destructive': isUnsafeEnv }"
    @dblclick="handleDoubleClick"
  >
    <!-- Background drag region sits behind interactive elements -->
    <div class="titlebar-drag-region"></div>

    <!-- Left Aligned Controls Section (avoids mac camera cutout in center) -->
    <div class="titlebar-section left-section flex-1 gap-2">
      <div v-if="isMac" class="mac-traffic-lights-spacer"></div>

      <!-- 1. XRest Brand -->
      <span class="font-bold text-xs tracking-tight text-foreground select-none px-1">
        XRest
      </span>

      <!-- Divider -->
      <div class="h-4 w-px bg-border shrink-0 my-auto"></div>

      <!-- 2. SearchBar -->
      <SearchBar v-model="searchQuery" class="!mr-0" />

      <!-- Divider -->
      <div class="h-4 w-px bg-border shrink-0 my-auto"></div>

      <!-- 3. Environment Selector -->
      <EnvironmentSelector />

      <!-- Divider -->
      <div class="h-4 w-px bg-border shrink-0 my-auto"></div>

      <!-- 4. Settings Icon -->
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="icon-lg"
              class="h-7.5 w-7.5 cursor-pointer text-muted-foreground hover:text-foreground"
              @click="goToSettings"
            >
              <RiSettings4Line class="h-4.5 w-4.5" />
            </Button>
          </TooltipTrigger>
          <TooltipContent
            side="bottom"
            class="z-50 border px-2.5 py-1 text-sm rounded shadow-md"
          >
            Settings
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
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
