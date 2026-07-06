<script setup lang="ts">
import type { DialogOverlayProps } from "reka-ui";
import type { HTMLAttributes } from "vue";
import { reactiveOmit } from "@vueuse/core";
import { DialogOverlay } from "reka-ui";
import { cn } from "@/lib/utils";
import { Window } from "@wailsio/runtime";

const props = defineProps<
  DialogOverlayProps & { class?: HTMLAttributes["class"] }
>();

const delegatedProps = reactiveOmit(props, "class");

const handleDoubleClick = async () => {
  // @ts-ignore
  if (window.wails) {
    if (await Window.IsMaximised()) {
      await Window.UnMaximise();
    } else {
      await Window.Maximise();
    }
  }
};
</script>

<template>
  <DialogOverlay data-slot="dialog-overlay" v-bind="delegatedProps" :class="cn(
    'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/80',
    props.class,
  )
    ">
    <!-- Draggable region for macOS traffic lights -->
    <div class="absolute top-0 left-0 right-0 h-10 drag-region" @dblclick="handleDoubleClick" />
    <slot />
  </DialogOverlay>
</template>

<style scoped>
.drag-region {
  cursor: default;
  -webkit-user-select: none;
  user-select: none;
  --wails-draggable: drag;
}
</style>
