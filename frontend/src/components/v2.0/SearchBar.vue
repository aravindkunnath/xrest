<script setup lang="ts">
import { RiSearch2Line } from '@remixicon/vue'
import { Input } from '@/components/ui/input'

const modelValue = defineModel<string>({ default: '' })
</script>

<template>
  <div class="flex-1 max-w-sm mx-4 relative group">
    <div class="absolute inset-y-0 left-2.5 flex items-center pointer-events-none text-muted-foreground group-focus-within:text-foreground transition-colors duration-200">
      <RiSearch2Line class="h-3.5 w-3.5" />
    </div>
    <Input 
      v-model="modelValue"
      type="text" 
      placeholder="Search requests..." 
      class="h-7.5 pl-8 pr-10 bg-muted/20 text-sm text-foreground placeholder:text-muted-foreground focus:bg-background"
    />
    <div class="absolute inset-y-0 right-2.5 flex items-center pointer-events-none">
      <kbd class="hidden sm:inline-flex h-4 items-center gap-0.5 rounded border border-border bg-muted px-1 font-mono text-[11px] font-medium text-muted-foreground shadow-3xs">
        <span>⌘</span><span>K</span>
      </kbd>
    </div>
  </div>
</template>
