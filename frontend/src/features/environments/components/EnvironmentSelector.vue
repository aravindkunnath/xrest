<script setup lang="ts">
import { computed } from 'vue'
import { useServicesStore } from "@/stores/services"
import { useTabsStore } from "@/stores/tabs"
import { Globe, ShieldAlert } from '@lucide/vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

defineOptions({
  name: 'EnvironmentSelector'
})

const servicesStore = useServicesStore()
const tabsStore = useTabsStore()

const activeTab = computed(() => {
  return tabsStore.tabs.find((t) => t.id === tabsStore.activeTab)
})

const activeService = computed(() => {
  if (!servicesStore.services.length) return null
  if (activeTab.value?.serviceId) {
    const matched = servicesStore.services.find((s) => s.id === activeTab.value?.serviceId)
    if (matched) return matched
  }
  return servicesStore.services[0]
})

const environments = computed(() => {
  return activeService.value?.environments || []
})

const currentEnvName = computed(() => {
  if (!activeService.value) return 'No Service'
  if (activeService.value.selectedEnvironment) {
    return activeService.value.selectedEnvironment
  }
  return activeService.value.environments[0]?.name || 'DEFAULT'
})

const isCurrentEnvUnsafe = computed(() => {
  if (!activeService.value) return false
  const env = activeService.value.environments.find((e) => e.name === currentEnvName.value)
  return env?.isUnsafe ?? false
})

const handleSelectEnv = (envName: unknown) => {
  if (activeService.value && typeof envName === 'string') {
    servicesStore.setSelectedEnvironment(activeService.value.id, envName)
  }
}
</script>

<template>
  <div
    class="environment-selector flex items-center"
    :class="{ 'is-unsafe': isCurrentEnvUnsafe }"
  >
    <Select
      v-if="environments.length > 0"
      :model-value="currentEnvName"
      @update:model-value="handleSelectEnv"
    >
      <SelectTrigger
        class="h-7.5 px-2 text-xs font-medium border-muted-foreground/20 hover:bg-accent hover:text-accent-foreground flex items-center gap-1.5 rounded-md transition-colors"
        :class="
          isCurrentEnvUnsafe
            ? 'border-destructive/60 bg-destructive/15 text-destructive font-semibold hover:bg-destructive/20'
            : ''
        "
      >
        <SelectValue placeholder="Select Environment">
          <div class="flex items-center gap-1.5 truncate max-w-[180px]">
            <ShieldAlert v-if="isCurrentEnvUnsafe" class="h-3.5 w-3.5 text-destructive shrink-0" />
            <Globe v-else class="h-3.5 w-3.5 text-muted-foreground shrink-0" />
            <span class="truncate font-semibold">{{ currentEnvName }}</span>
          </div>
        </SelectValue>
      </SelectTrigger>
      <SelectContent class="z-[1100]">
        <SelectItem
          v-for="env in environments"
          :key="env.name"
          :value="env.name"
          class="text-xs flex items-center justify-between gap-2 cursor-pointer"
        >
          <div class="flex items-center gap-1.5">
            <ShieldAlert v-if="env.isUnsafe" class="h-3 w-3 text-destructive shrink-0" />
            <span :class="env.isUnsafe ? 'text-destructive font-medium' : ''">{{ env.name }}</span>
          </div>
        </SelectItem>
      </SelectContent>
    </Select>
    <div
      v-else
      class="h-7.5 px-2 flex items-center gap-1 text-xs text-muted-foreground border border-dashed rounded-md"
    >
      <Globe class="h-3.5 w-3.5" />
      <span>No Env</span>
    </div>
  </div>
</template>
