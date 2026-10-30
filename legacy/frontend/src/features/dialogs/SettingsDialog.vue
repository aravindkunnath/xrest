<script setup lang="ts">
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import SettingsView from "@/features/settings/views/SettingsView.vue";

defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[750px] max-h-[85vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle>Settings</DialogTitle>
        <DialogDescription>
          Manage your interface preferences and application configuration.
        </DialogDescription>
      </DialogHeader>
      
      <div class="py-4">
        <!-- We use SettingsView here but remove some header padding or titles if needed. -->
        <!-- However, let's keep it simple by embedding it. Let's override any extreme paddings/headings. -->
        <SettingsView class="!p-0 !max-w-none" :embedded="true" />
      </div>
    </DialogContent>
  </Dialog>
</template>
