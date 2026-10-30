<script setup lang="ts">
import { Copy, Play, X } from "@lucide/vue";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
} from "@/components/ui/accordion";
import JsonHighlighter from "@/components/editor/JsonHighlighter.vue";
import ImageViewer from "@/features/response/components/ImageViewer.vue";
import HtmlPreviewer from "@/features/response/components/HtmlPreviewer.vue";
import MediaViewer from "@/features/response/components/MediaViewer.vue";
import TableViewer from "@/features/response/components/TableViewer.vue";
import MultipartViewer from "@/features/response/components/MultipartViewer.vue";
import BinaryViewer from "@/features/response/components/BinaryViewer.vue";

import { resolveContentType } from "@/core/utils/contentTypeResolver";
import { toast } from "vue-sonner";
import { computed } from "vue";

const props = defineProps<{
    response: {
        activeTab: string;
        status: number;
        statusText: string;
        time: string;
        size: string;
        type: string;
        body: string;
        error: string;
        headers: any[];
        requestHeaders: any[];
    };
    url: string;
    variables: Record<string, string>;
    environmentName: string;
}>();

const response = computed(() => props.response);

const hasResponse = computed(() => {
    const r = response.value;
    if (!r) return false;
    return Boolean(
        (r.status && r.status > 0) ||
        r.error ||
        (r.body && r.body.length > 0) ||
        (r.headers && r.headers.length > 0)
    );
});

const activeTabValue = computed({
    get: () => response.value?.activeTab || "body",
    set: (val: string) => {
        if (response.value) {
            response.value.activeTab = val;
        }
    },
});

const contentTypeHeader = computed(() => {
    const h = response.value?.headers?.find(
        (header: any) => header.name?.toLowerCase() === "content-type",
    );
    return h?.value || "";
});

const resolvedContentType = computed(() => resolveContentType(contentTypeHeader.value));

const contentDispositionHeader = computed(() => {
    const h = response.value?.headers?.find(
        (header: any) => header.name?.toLowerCase() === "content-disposition",
    );
    return h?.value || "";
});

const extractedFilename = computed(() => {
    const cd = contentDispositionHeader.value;
    if (!cd) return undefined;
    const match = cd.match(/filename=["']?([^"';]+)["']?/i);
    return match ? match[1] : undefined;
});

const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast.success("Copied to clipboard", {
        duration: 2000,
    });
};
</script>

<template>
    <div class="h-full flex flex-col bg-background">
        <template v-if="hasResponse">
            <!-- Tabs Header Bar -->
            <Tabs v-model="activeTabValue" class="flex-1 flex flex-col overflow-hidden">
                <div class="flex items-center justify-between border-b px-4 shrink-0 h-9">
                    <TabsList class="justify-start h-9 bg-transparent p-0 border-b-0 rounded-none shrink-0">
                        <TabsTrigger
                            value="body"
                            class="h-9 text-xs rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
                        >
                            Body
                        </TabsTrigger>
                        <TabsTrigger
                            value="headers"
                            class="h-9 text-xs rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent"
                        >
                            Headers
                        </TabsTrigger>
                    </TabsList>

                    <!-- Status Metadata Badges -->
                    <div class="flex gap-3 text-xs items-center">
                        <span
                            :class="[
                                response.status >= 200 && response.status < 300
                                    ? 'text-green-600 dark:text-green-500'
                                    : 'text-destructive',
                                'font-bold font-mono',
                            ]"
                        >
                            {{
                                response.status
                                    ? (response.statusText.startsWith(String(response.status))
                                        ? response.statusText
                                        : `${response.status} ${response.statusText}`)
                                    : response.statusText
                            }}
                        </span>
                        <span class="text-[10px] h-4 px-1.5 rounded inline-flex items-center font-mono font-normal bg-muted text-muted-foreground border">
                            {{ response.time }}
                        </span>
                        <span class="text-[10px] h-4 px-1.5 rounded inline-flex items-center font-mono font-normal bg-muted text-muted-foreground border">
                            {{ response.size }}
                        </span>
                    </div>
                </div>

                <!-- Response Content Body -->
                <div class="flex-1 overflow-auto p-4">
                    <!-- Error Display -->
                    <div
                        v-if="response.error"
                        class="mb-4 p-4 border border-destructive/20 bg-destructive/5 rounded-md animate-in fade-in slide-in-from-top-2 duration-300"
                    >
                        <div class="flex items-start gap-3">
                            <X class="h-4 w-4 text-destructive mt-0.5 shrink-0" />
                            <div class="space-y-1">
                                <h4 class="font-bold text-destructive text-xs">
                                    Request Failed
                                </h4>
                                <p class="text-destructive/80 text-xs break-all">
                                    {{ response.error }}
                                </p>
                            </div>
                        </div>
                    </div>

                    <!-- Response Body Tab -->
                    <div
                        v-if="response.activeTab === 'body'"
                        class="h-full flex flex-col gap-3 animate-in fade-in duration-300"
                    >
                        <div class="flex items-center justify-between">
                            <div class="flex items-center gap-2">
                                <span class="text-xs font-semibold text-foreground/80 tracking-tight">
                                    Content-Type:
                                </span>
                                <span class="text-xs font-mono text-foreground bg-muted px-1.5 py-0.5 rounded border">
                                    {{ contentTypeHeader || "unknown" }}
                                </span>
                            </div>
                            <button
                                @click="copyToClipboard(response.body)"
                                class="flex items-center gap-1.5 px-2 py-1 hover:bg-muted rounded text-xs text-muted-foreground hover:text-foreground transition-colors"
                            >
                                <Copy class="h-3.5 w-3.5" /> Copy
                            </button>
                        </div>

                        <!-- Dynamic Sub-viewers based on Content-Type -->
                        <ImageViewer
                            v-if="resolvedContentType.category === 'image'"
                            :src="response.body"
                            :mime="resolvedContentType.mime"
                            :raw-code="response.body"
                            class="flex-1"
                        />

                        <HtmlPreviewer
                            v-else-if="resolvedContentType.category === 'html'"
                            :html="response.body"
                            class="flex-1"
                        />

                        <MediaViewer
                            v-else-if="resolvedContentType.category === 'audio' || resolvedContentType.category === 'video'"
                            :src="response.body"
                            :mime="resolvedContentType.mime"
                            class="flex-1"
                        />

                        <TableViewer
                            v-else-if="resolvedContentType.category === 'csv'"
                            :content="response.body"
                            class="flex-1"
                        />

                        <MultipartViewer
                            v-else-if="resolvedContentType.category === 'multipart'"
                            :content="response.body"
                            :boundary="resolvedContentType.boundary"
                            class="flex-1"
                        />

                        <BinaryViewer
                            v-else-if="resolvedContentType.category === 'binary' || resolvedContentType.category === 'pdf'"
                            :size="response.size || '0 B'"
                            :mime="resolvedContentType.mime"
                            :filename="extractedFilename"
                            :content="response.body"
                            class="flex-1"
                        />

                        <JsonHighlighter
                            v-else
                            :code="response.body"
                            class="flex-1 min-h-[100px]"
                        />
                    </div>

                    <!-- Response Headers Tab -->
                    <div
                        v-if="response.activeTab === 'headers'"
                        class="animate-in fade-in duration-300 pb-4"
                    >
                        <Accordion
                            type="multiple"
                            class="w-full"
                            :default-value="['response-headers']"
                        >
                            <AccordionItem
                                value="response-headers"
                                class="border rounded-md px-3 mb-3 bg-card shadow-xs"
                            >
                                <AccordionTrigger class="py-2 hover:no-underline">
                                    <span class="text-xs font-bold uppercase text-primary tracking-wider"
                                        >Response Headers</span
                                    >
                                </AccordionTrigger>
                                <AccordionContent>
                                    <div class="space-y-1 py-1 text-xs">
                                        <div
                                            v-for="h in response.headers"
                                            :key="h.name"
                                            class="grid grid-cols-[160px_1fr] gap-4 border-b border-dashed border-muted/50 pb-1.5 pt-1.5 last:border-0"
                                        >
                                            <span
                                                class="font-mono font-medium text-muted-foreground"
                                                >{{ h.name }}</span
                                            >
                                            <span class="font-mono text-foreground truncate break-all">{{ h.value }}</span>
                                        </div>
                                    </div>
                                </AccordionContent>
                            </AccordionItem>
                        </Accordion>
                    </div>
                </div>
            </Tabs>
        </template>
        <template v-else>
            <div class="h-full flex flex-col items-center justify-center text-muted-foreground gap-3 bg-muted/5 p-6">
                <div class="p-4 rounded-full bg-muted/20 border border-dashed border-muted">
                    <Play class="h-8 w-8 opacity-20" />
                </div>
                <p class="font-medium tracking-tight text-sm">
                    Send a request to see the response
                </p>
            </div>
        </template>
    </div>
</template>
