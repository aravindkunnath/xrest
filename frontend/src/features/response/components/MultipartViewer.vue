<script setup lang="ts">
import { computed } from "vue";
import { Layers } from "@lucide/vue";

const props = defineProps<{
    content: string;
    boundary?: string;
}>();

// interface MultipartPart {
//     headers: Record<string, string>;
//     body: string;
// }

const parsedParts = computed(() => {
    if (!props.content) return [];
    let boundary = props.boundary;
    if (!boundary) {
        const firstLine = props.content.split(/\r?\n/)[0];
        if (firstLine && firstLine.startsWith("--")) {
            boundary = firstLine.substring(2).trim();
        }
    }

    if (!boundary) return [];

    const delimiter = `--${boundary}`;
    const rawParts = props.content.split(delimiter).filter((p) => p.trim() && p.trim() !== "--");

    return rawParts.map((raw) => {
        const lines = raw.split(/\r?\n/);
        const headers: Record<string, string> = {};
        let bodyStartIndex = 0;

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i].trim();
            if (!line) {
                bodyStartIndex = i + 1;
                break;
            }
            const colonIdx = line.indexOf(":");
            if (colonIdx > 0) {
                headers[line.substring(0, colonIdx).trim().toLowerCase()] = line.substring(colonIdx + 1).trim();
            }
        }

        const body = lines.slice(bodyStartIndex).join("\n").trim();
        return { headers, body };
    });
});
</script>

<template>
    <div class="flex flex-col h-full bg-background border rounded-md overflow-hidden">
        <div class="flex items-center gap-2 px-3 py-1.5 border-b bg-muted/40 text-xs">
            <Layers class="h-3.5 w-3.5 text-muted-foreground" />
            <span class="font-medium">Multipart Payload Parts</span>
            <span class="text-muted-foreground">({{ parsedParts.length }} parts)</span>
        </div>

        <div class="flex-1 overflow-auto p-4 space-y-3">
            <div
                v-for="(part, idx) in parsedParts"
                :key="idx"
                data-testid="multipart-part"
                class="border rounded-md p-3 bg-card shadow-xs space-y-2"
            >
                <div class="flex items-center justify-between text-xs font-mono text-primary font-semibold border-b pb-1">
                    <span>Part #{{ idx + 1 }}</span>
                    <span>{{ part.headers["content-type"] || "text/plain" }}</span>
                </div>
                <div v-if="Object.keys(part.headers).length" class="text-xs space-y-1 bg-muted/30 p-2 rounded">
                    <div v-for="(v, k) in part.headers" :key="k" class="font-mono text-[11px] text-muted-foreground">
                        <span class="font-semibold">{{ k }}:</span> {{ v }}
                    </div>
                </div>
                <pre class="font-mono text-xs p-2 bg-muted/50 rounded overflow-auto max-h-[150px]">{{ part.body }}</pre>
            </div>
        </div>
    </div>
</template>
