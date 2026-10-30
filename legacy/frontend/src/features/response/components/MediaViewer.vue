<script setup lang="ts">
import { computed } from "vue";
import { Music, Film } from "@lucide/vue";

const props = defineProps<{
    src: string;
    mime: string;
}>();

const isAudio = computed(() => props.mime.startsWith("audio/"));
</script>

<template>
    <div class="flex flex-col items-center justify-center p-6 bg-card border rounded-md min-h-[200px]">
        <div v-if="isAudio" class="flex flex-col items-center gap-4 w-full max-w-md">
            <div class="p-4 bg-primary/10 text-primary rounded-full">
                <Music class="h-8 w-8" />
            </div>
            <span class="text-xs font-mono text-muted-foreground">{{ mime }}</span>
            <audio controls class="w-full">
                <source :src="src" :type="mime" />
                Your browser does not support audio playback.
            </audio>
        </div>

        <div v-else class="flex flex-col items-center gap-4 w-full max-w-xl">
            <div class="flex items-center gap-2 text-xs font-mono text-muted-foreground">
                <Film class="h-4 w-4" /> {{ mime }}
            </div>
            <video controls class="w-full max-h-[400px] rounded border bg-black shadow-xs">
                <source :src="src" :type="mime" />
                Your browser does not support video playback.
            </video>
        </div>
    </div>
</template>
