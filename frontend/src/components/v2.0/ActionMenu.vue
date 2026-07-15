<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { 
  RiAddLine, 
  RiFolderAddLine, 
  RiFileAddLine, 
  RiUploadCloud2Line,
  RiSettings4Line
} from '@remixicon/vue'
import { Button } from '@/components/ui/button'
import { useDialogState } from '@/composables/useDialogState'
import { useRouter } from 'vue-router'
import { useTabsStore } from '@/stores/tabs'

const isDropdownOpen = ref(false)
const menuRef = ref<HTMLElement | null>(null)
const router = useRouter()
const dialogState = useDialogState()
const tabsStore = useTabsStore()

const toggleDropdown = () => {
  isDropdownOpen.value = !isDropdownOpen.value
}

const handleClickOutside = (event: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    isDropdownOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

const actions = [
  { id: 'new-request', label: 'New Request', desc: 'Create a new API request', icon: RiFileAddLine, shortcut: 'N' },
  { id: 'new-collection', label: 'New Collection', desc: 'Group requests together', icon: RiFolderAddLine, shortcut: 'G' },
  { id: 'import', label: 'Import Curl / OpenAPI', desc: 'Import from file or text', icon: RiUploadCloud2Line, shortcut: 'I' },
  { id: 'settings', label: 'Settings', desc: 'App configuration', icon: RiSettings4Line, shortcut: ',' },
]

const handleAction = (actionId: string) => {
  console.log(`Action triggered: ${actionId}`)
  isDropdownOpen.value = false

  if (actionId === 'new-request') {
    tabsStore.addTab()
    router.push('/services')
  } else if (actionId === 'new-collection') {
    dialogState.openCollectionDialog()
    router.push('/services')
  } else if (actionId === 'import') {
    dialogState.openSwaggerDialog()
    router.push('/services')
  } else if (actionId === 'settings') {
    router.push('/settings')
  }
}
</script>


<template>
  <div ref="menuRef" class="relative">
    <Button 
      variant="outline"
      size="icon-sm"
      @click="toggleDropdown"
      class="h-7.5 w-7.5 focus-visible:ring-1.5 focus-visible:ring-ring/50 cursor-pointer"
      aria-label="Add new item"
      :aria-expanded="isDropdownOpen"
    >
      <RiAddLine class="h-4.5 w-4.5 transition-transform duration-200" :class="{ 'rotate-45': isDropdownOpen }" />
    </Button>

    <!-- Dropdown Panel -->
    <Transition
      enter-active-class="transition duration-100 ease-out"
      enter-from-class="transform scale-95 opacity-0 -translate-y-1"
      enter-to-class="transform scale-100 opacity-100 translate-y-0"
      leave-active-class="transition duration-75 ease-in"
      leave-from-class="transform scale-100 opacity-100 translate-y-0"
      leave-to-class="transform scale-95 opacity-0 -translate-y-1"
    >
      <div 
        v-if="isDropdownOpen"
        class="absolute right-0 mt-1.5 w-64 origin-top-right rounded border border-border bg-popover p-1 shadow-md focus:outline-none"
        role="menu"
      >
        <div class="px-2 py-1 text-xs font-bold uppercase tracking-wider text-muted-foreground border-b border-border/50 mb-1">
          Quick Actions
        </div>
        <button
          v-for="action in actions"
          :key="action.id"
          @click="handleAction(action.id)"
          class="w-full flex items-start gap-2.5 rounded-sm px-2 py-1.5 text-left hover:bg-accent hover:text-accent-foreground transition-colors duration-150 group/item cursor-pointer"
          role="menuitem"
        >
          <component :is="action.icon" class="h-3.5 w-3.5 text-muted-foreground group-hover/item:text-foreground mt-0.5" />
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-foreground leading-none">{{ action.label }}</span>
              <kbd class="hidden group-hover/item:inline-flex h-3.5 items-center rounded border border-border bg-muted px-1 font-mono text-[10px] font-medium text-muted-foreground leading-none">
                {{ action.shortcut }}
              </kbd>
            </div>
            <p class="text-xs text-muted-foreground truncate mt-0.5">{{ action.desc }}</p>
          </div>
        </button>
      </div>
    </Transition>
  </div>
</template>
