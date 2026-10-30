<script setup lang="ts">
import { FileDown, Download } from "@lucide/vue";
import { toast } from "vue-sonner";

const props = defineProps<{
    size: string;
    mime: string;
    filename?: string;
    content?: string;
}>();

const downloadFile = () => {
    const filename = props.filename || `response-file.${props.mime.split("/")[1] || "bin"}`;
    const blob = new Blob([props.content || ""], { type: props.mime });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
    toast.success(`Downloading ${filename}`);
};
</script>

<template>
    <div class="flex flex-col items-center justify-center p-8 bg-card border rounded-md min-h-[220px]">
        <div class="flex flex-col items-center gap-4 text-center max-w-sm">
            <div class="p-4 bg-primary/10 text-primary rounded-full">
                <FileDown class="h-10 w-10" />
            </div>
            <div class="space-y-1">
                <h3 class="font-bold text-sm text-foreground break-all">
                    {{ filename || "Binary Content File" }}
                </h3>
                <div class="flex items-center justify-center gap-2 text-xs font-mono text-muted-foreground">
                    <span>{{ mime }}</span>
                    <span>•</span>
                    <span>{{ size }}</span>
                </div>
            </div>

            <button
                data-testid="download-btn"
                @click="downloadFile"
                class="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 font-medium rounded-md text-xs transition-colors shadow-xs"
            >
                <Download class="h-4 w-4" /> Download File
            </button>
        </div>
    </div>
</template>
