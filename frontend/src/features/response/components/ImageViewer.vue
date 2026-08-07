<script setup lang="ts">
import { Code, Copy, Grid, RotateCcw, ZoomIn, ZoomOut } from "@lucide/vue";
import { computed, ref } from "vue";
import { toast } from "vue-sonner";

const props = defineProps<{
    src: string;
    mime: string;
    rawCode?: string;
}>();

const zoom = ref(100);
const bgMode = ref<"grid" | "dark" | "light">("grid");
const showSvgCode = ref(false);

const zoomIn = () => {
    if (zoom.value < 400) zoom.value += 25;
};

const zoomOut = () => {
    if (zoom.value > 25) zoom.value -= 25;
};

const resetZoom = () => {
    zoom.value = 100;
};

const toggleBg = () => {
    if (bgMode.value === "grid") bgMode.value = "dark";
    else if (bgMode.value === "dark") bgMode.value = "light";
    else bgMode.value = "grid";
};

const isSvg = computed(() => props.mime === "image/svg+xml" || props.src.includes("svg+xml"));

const formattedSrc = computed(() => {
    if (!props.src) return "";
    if (props.src.startsWith("http://") || props.src.startsWith("https://") || props.src.startsWith("data:") || props.src.startsWith("blob:")) {
        return props.src;
    }
    // If raw base64 or raw string without data: prefix
    return `data:${props.mime};base64,${props.src}`;
});

const copyImage = () => {
    navigator.clipboard.writeText(props.src);
    toast.success("Image URL/Data copied");
};

</script>

<template>
    <div class="flex flex-col h-full bg-background border rounded-md overflow-hidden">
        <!-- Toolbar -->
        <div class="flex items-center justify-between px-3 py-1.5 border-b bg-muted/40 text-xs">
            <div class="flex items-center gap-2">
                <span class="font-mono text-muted-foreground">{{ mime }}</span>
                <span class="text-muted-foreground">•</span>
                <span class="font-mono font-medium">{{ zoom }}%</span>
            </div>

            <div class="flex items-center gap-1">
                <button
                    data-testid="zoom-out"
                    @click="zoomOut"
                    class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                    title="Zoom Out"
                >
                    <ZoomOut class="h-3.5 w-3.5" />
                </button>
                <button
                    data-testid="zoom-in"
                    @click="zoomIn"
                    class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                    title="Zoom In"
                >
                    <ZoomIn class="h-3.5 w-3.5" />
                </button>
                <button
                    data-testid="zoom-reset"
                    @click="resetZoom"
                    class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                    title="Reset Zoom"
                >
                    <RotateCcw class="h-3.5 w-3.5" />
                </button>

                <div class="h-3 w-[1px] bg-border mx-1" />

                <button
                    data-testid="bg-toggle"
                    @click="toggleBg"
                    class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                    title="Toggle Canvas Background"
                >
                    <Grid class="h-3.5 w-3.5" />
                </button>

                <button
                    v-if="isSvg"
                    data-testid="svg-code-toggle"
                    @click="showSvgCode = !showSvgCode"
                    class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                    title="Toggle SVG Code"
                >
                    <Code class="h-3.5 w-3.5" />
                </button>

                <div class="h-3 w-[1px] bg-border mx-1" />

                <button
                    @click="copyImage"
                    class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground"
                    title="Copy Link/Data"
                >
                    <Copy class="h-3.5 w-3.5" />
                </button>
            </div>
        </div>

        <!-- Canvas / Display Area -->
        <div
            data-testid="image-canvas"
            :class="[
                'flex-1 flex items-center justify-center p-4 overflow-auto min-h-[250px]',
                bgMode === 'grid' ? 'bg-[radial-gradient(#e5e7eb_1px,transparent_1px)] dark:bg-[radial-gradient(#1f2937_1px,transparent_1px)] [background-size:16px_16px]' : '',
                bgMode === 'dark' ? 'bg-slate-950 bg-dark text-white' : '',
                bgMode === 'light' ? 'bg-white text-black' : '',
            ]"
        >
            <div v-if="showSvgCode" class="w-full h-full">
                <pre class="font-mono text-xs p-4 bg-muted/50 rounded overflow-auto h-full">{{ rawCode || src }}</pre>
            </div>
            <img
                v-else
                :src="formattedSrc"
                :alt="mime"
                :style="{ transform: `scale(${zoom / 100})`, transition: 'transform 0.15s ease-out' }"
                class="max-h-full object-contain rounded shadow-xs"
            />
        </div>
    </div>
</template>
