<script setup lang="ts">
import { defineAsyncComponent } from "vue";
import { Sheet, SheetContent } from "@/components/ui/sheet";
import { useDialogState } from "@/composables/useDialogState";

const HistoryView = defineAsyncComponent(() => import("@/features/history/views/HistoryView.vue"));

const { isHistoryOverlayOpen } = useDialogState();
</script>

<template>
    <Sheet v-model:open="isHistoryOverlayOpen">
        <SheetContent side="right" class="w-full sm:max-w-2xl md:max-w-3xl lg:max-w-4xl p-0 h-full flex flex-col border-l border-border bg-background z-50">
            <HistoryView />
        </SheetContent>
    </Sheet>
</template>
