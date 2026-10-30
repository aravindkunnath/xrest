<script setup lang="ts">
import { ref } from "vue";
import { Copy, Eye, Code } from "@lucide/vue";
import { toast } from "vue-sonner";

const props = defineProps<{
    html: string;
}>();

const mode = ref<"preview" | "raw">("preview");

const copyHtml = () => {
    navigator.clipboard.writeText(props.html);
    toast.success("HTML copied to clipboard");
};
</script>

<template>
    <div class="flex flex-col h-full bg-background border rounded-md overflow-hidden">
        <div class="flex items-center justify-between px-3 py-1.5 border-b bg-muted/40 text-xs">
            <div class="flex items-center gap-1 bg-muted p-0.5 rounded border">
                <button
                    data-testid="tab-preview"
                    @click="mode = 'preview'"
                    :class="[
                        'px-2 py-0.5 rounded transition-colors text-xs font-medium',
                        mode === 'preview' ? 'bg-background text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground',
                    ]"
                >
                    <span class="flex items-center gap-1"><Eye class="h-3 w-3" /> Preview</span>
                </button>
                <button
                    data-testid="tab-raw"
                    @click="mode = 'raw'"
                    :class="[
                        'px-2 py-0.5 rounded transition-colors text-xs font-medium',
                        mode === 'raw' ? 'bg-background text-foreground shadow-xs' : 'text-muted-foreground hover:text-foreground',
                    ]"
                >
                    <span class="flex items-center gap-1"><Code class="h-3 w-3" /> Raw Source</span>
                </button>
            </div>

            <button
                @click="copyHtml"
                class="flex items-center gap-1 px-2 py-1 hover:bg-muted rounded text-xs text-muted-foreground hover:text-foreground"
            >
                <Copy class="h-3.5 w-3.5" /> Copy
            </button>
        </div>

        <div class="flex-1 overflow-auto p-2 bg-card">
            <iframe
                v-if="mode === 'preview'"
                :srcdoc="html"
                sandbox="allow-scripts"
                class="w-full h-full min-h-[300px] border-0 rounded bg-white"
            />
            <pre
                v-else
                class="font-mono text-xs p-4 bg-muted/50 rounded overflow-auto h-full whitespace-pre-wrap break-all"
            >{{ html }}</pre>
        </div>
    </div>
</template>
