<script setup lang="ts">
import { Button } from "@/components/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
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
import { RiArrowRightSLine, RiFolderLine, RiSettings4Line } from '@remixicon/vue'
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
const { openSettingsDialog, openServiceDialog } = useDialogState()

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

const currentServiceId = computed(() => {
  if (servicesStore.selectedServiceId) return servicesStore.selectedServiceId
  return servicesStore.services[0]?.id || ''
})

const selectedServiceName = computed(() => {
  if (!servicesStore.services?.length) return ''
  return servicesStore.services.find((s) => s.id === currentServiceId.value)?.name || servicesStore.services[0].name
})

const handleSelectService = (value: unknown) => {
  if (typeof value !== 'string' || !value) return
  if (value === '__new__') {
    openServiceDialog()
    return
  }
  servicesStore.setSelectedService(value)
}

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

      <!-- 1. Service Selector (replaces workspace selector) -->
      <Select :model-value="currentServiceId" @update:model-value="handleSelectService">
        <SelectTrigger
          class="h-6 px-2 text-sm font-semibold border-0 focus-visible:ring-0 hover:bg-accent hover:text-accent-foreground flex items-center gap-1.5 rounded-md transition-colors text-foreground"
        >
          <SelectValue placeholder="Select Service">
            <div class="flex items-center gap-1.5 truncate max-w-[200px]">
              <RiFolderLine class="h-3.5 w-3.5 text-muted-foreground shrink-0" />
              <span class="truncate font-semibold">{{ selectedServiceName || 'No Service' }}</span>
            </div>
          </SelectValue>
        </SelectTrigger>
        <SelectContent class="z-[1100]">
          <SelectItem
            v-for="service in servicesStore.services"
            :key="service.id"
            :value="service.id"
            class="text-sm cursor-pointer"
          >
            {{ service.name }}
          </SelectItem>
          <SelectSeparator />
          <SelectItem value="__new__" class="text-sm text-primary font-medium cursor-pointer">
            + New Service
          </SelectItem>
        </SelectContent>
      </Select>

      <!-- 2. Breadcrumb Separator -->
      <RiArrowRightSLine class="h-4 w-4 text-muted-foreground shrink-0" />

      <!-- 3. Environment Selector -->
      <EnvironmentSelector />
    </div>

    <!-- Center Display -->
    <div class="center-section titlebar-section">
      <span class="truncate text-sm font-medium text-foreground select-none">
        XRest<span v-if="selectedServiceName" class="text-muted-foreground"> · {{ selectedServiceName }}</span>
      </span>
    </div>

    <!-- Right Utility & Status Section -->
    <div class="titlebar-section right-section flex-1 gap-2 justify-end">
      <!-- 4. Quick Search -->
      <SearchBar v-model="searchQuery" class="mr-0!" />

      <!-- 5. Settings Icon -->
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="icon-sm"
              class="h-6 w-6 cursor-pointer text-muted-foreground hover:text-foreground focus-visible:ring-0"
              @click="goToSettings"
            >
              <RiSettings4Line class="h-4 w-4" />
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

      <!-- Mirrors the mac traffic-light spacer so the center label stays truly centered -->
      <div v-if="!isMac" class="right-spacer"></div>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 40px;
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

/* Absolutely centered active view indicator */
.center-section {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  max-width: 40%;
  min-width: 0;
  pointer-events: none;
}

.center-section span {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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